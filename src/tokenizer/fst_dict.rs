// SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

//! Memory-efficient dictionary backed by a Finite State Transducer (FST).
//!
//! The `FstDict` stores the base word set as an `fst::Set`, which
//! minimizes the deterministic automaton and typically uses only a few bytes
//! per entry (compared with tens of bytes per trie node in the `TrieChar`
//! implementation).
//!
//! Because `fst::Set` is immutable, dynamic add/remove operations are
//! handled with two small delta `HashSet`s:
//!   - `additions`: words added after construction.
//!   - `removals`: words in the base set that have been removed.
//!
//! The lookup API mirrors `TrieChar::prefix_ref`, returning the character
//! lengths of all dictionary entries that form a prefix of the query text.
//!
//! # Construction
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use nlpo3::tokenizer::fst_dict::FstDict;
//!
//! let dict = FstDict::from_words(["กา", "กาแฟ", "แฟ"].iter().copied())?;
//! let lengths = dict.prefix_lengths("กาแฟดี");
//! assert!(lengths.contains(&2)); // "กา" (2 chars)
//! assert!(lengths.contains(&4)); // "กาแฟ" (4 chars)
//! # Ok(())
//! # }
//! ```

use fst::{Automaton, IntoStreamer, Set, Streamer};
use rustc_hash::FxHashSet as HashSet;
use std::{error::Error, fmt};

use crate::{char_string::CharString, tokenizer::dict_backend::DictBackend};

// ---------------------------------------------------------------------------
// Custom automaton: accepts all keys that are prefixes of a query string
// ---------------------------------------------------------------------------

/// An FST automaton that accepts any key `k` such that `query.starts_with(k)`.
struct PrefixOf<'a> {
    query: &'a [u8],
}

impl<'a> PrefixOf<'a> {
    fn new(query: &'a str) -> Self {
        Self {
            query: query.as_bytes(),
        }
    }
}

/// A dead (non-matching, non-extendable) state.
const DEAD: usize = usize::MAX;

impl<'a> Automaton for PrefixOf<'a> {
    /// State = number of query bytes consumed so far, or `DEAD`.
    type State = usize;

    fn start(&self) -> usize {
        0
    }

    /// A key matches if we consumed at least one byte and haven't gone off the
    /// rails (`state != DEAD`). `is_match` is called only when a key in the
    /// FST is fully consumed.
    fn is_match(&self, &state: &usize) -> bool {
        state != DEAD
    }

    /// We can still find matches as long as there are more query bytes to
    /// advance through.
    fn can_match(&self, &state: &usize) -> bool {
        state != DEAD && state <= self.query.len()
    }

    /// Advance the automaton by one byte of a key being searched in the FST.
    fn accept(&self, &state: &usize, byte: u8) -> usize {
        if state == DEAD || state >= self.query.len() {
            DEAD
        } else if self.query[state] == byte {
            state + 1
        } else {
            DEAD
        }
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct FstDictError(String);

impl fmt::Display for FstDictError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for FstDictError {}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// A memory-efficient dictionary backed by `fst::Set`.
///
/// Words are UTF-8 strings stored in a minimized finite state automaton.
/// Dynamic additions and removals are tracked in small delta sets.
#[derive(Clone)]
pub struct FstDict {
    /// Immutable base set, sorted lexicographically.
    base: Set<Vec<u8>>,
    /// Words added dynamically after construction.
    additions: HashSet<String>,
    /// Words in `base` that have been removed.
    removals: HashSet<String>,
}

impl fmt::Debug for FstDict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FstDict")
            .field("fst_bytes", &self.base.as_fst().as_bytes().len())
            .field("additions", &self.additions.len())
            .field("removals", &self.removals.len())
            .finish()
    }
}

impl FstDict {
    /// Build a dictionary from an iterator of word strings.
    ///
    /// The words are sorted before being inserted into the FST (required by
    /// `fst::SetBuilder`). Whitespace is trimmed from each word; empty words
    /// are skipped.
    pub fn from_words<'a, I>(words: I) -> Result<Self, Box<dyn Error>>
    where
        I: Iterator<Item = &'a str>,
    {
        let mut sorted: Vec<String> = words
            .map(|w| w.trim().to_string())
            .filter(|w| !w.is_empty())
            .collect();
        sorted.sort_unstable();
        sorted.dedup();

        let mut builder = fst::SetBuilder::memory();
        for word in &sorted {
            builder
                .insert(word.as_bytes())
                .map_err(|e| FstDictError(format!("FST insert error: {}", e)))?;
        }
        let set = builder.into_set();

        Ok(Self {
            base: set,
            additions: HashSet::default(),
            removals: HashSet::default(),
        })
    }

    /// Add a word to the dictionary.
    pub fn add(&mut self, word: &str) {
        let w = word.trim();
        if w.is_empty() {
            return;
        }
        let key = w.to_string();
        // If the word was previously removed, just un-remove it.
        if self.removals.contains(&key) {
            self.removals.remove(&key);
        } else if !self.base.contains(key.as_bytes()) {
            self.additions.insert(key);
        }
    }

    /// Remove a word from the dictionary.
    pub fn remove(&mut self, word: &str) {
        let w = word.trim();
        if w.is_empty() {
            return;
        }
        let key = w.to_string();
        if self.base.contains(key.as_bytes()) {
            self.removals.insert(key);
        } else {
            self.additions.remove(&key);
        }
    }

    /// Return `true` if `word` is currently in the dictionary.
    pub fn contains(&self, word: &str) -> bool {
        let key = word.trim();
        if self.removals.contains(key) {
            return false;
        }
        if self.additions.contains(key) {
            return true;
        }
        self.base.contains(key.as_bytes())
    }

    /// Return the character lengths of all dictionary entries that are
    /// prefixes of `text`.
    ///
    /// For example, if the dictionary contains "กา" (2 chars) and "กาแฟ"
    /// (4 chars) and `text` starts with "กาแฟดี", the result contains
    /// `[2, 4]`.
    ///
    /// The results are deduplicated and do not include a guaranteed order.
    pub fn prefix_lengths(&self, text: &str) -> Vec<usize> {
        // Use a HashSet to collect lengths so duplicates are impossible even
        // when the same character length appears from both the base FST and
        // the delta additions set (e.g., two different words of the same
        // length are both prefixes of `text`).
        let mut seen = rustc_hash::FxHashSet::<usize>::default();
        let mut result: Vec<usize> = Vec::new();

        // --- base FST search ---
        let automaton = PrefixOf::new(text);
        let mut stream = self.base.search(&automaton).into_stream();
        while let Some(key) = stream.next() {
            // SAFETY: All keys in the FST were inserted as valid UTF-8
            // strings in `from_words`. The FST stores raw bytes verbatim
            // and never modifies them, so every key is guaranteed to be
            // valid UTF-8. Using the unchecked variant avoids a redundant
            // byte-scan on every iteration of this hot lookup loop.
            let word = unsafe { std::str::from_utf8_unchecked(key) };
            if !self.removals.contains(word) {
                let len = word.chars().count();
                if seen.insert(len) {
                    result.push(len);
                }
            }
        }

        // --- delta additions ---
        // Walk through all character-boundary prefixes of text, checking additions.
        let mut byte_pos = 0;
        let mut char_pos = 0;
        for ch in text.chars() {
            byte_pos += ch.len_utf8();
            char_pos += 1;
            let prefix = &text[..byte_pos];
            if self.additions.contains(prefix) && seen.insert(char_pos) {
                result.push(char_pos);
            }
        }

        result
    }

    /// Return the number of bytes used by the immutable FST base set.
    ///
    /// This reflects the in-memory size of the compact automaton and can be
    /// compared against the equivalent `HashMap`/trie memory to illustrate
    /// the memory savings of the FST approach.
    pub fn fst_size_bytes(&self) -> usize {
        self.base.as_fst().as_bytes().len()
    }

    /// Return the total number of entries (base + additions − removals).
    pub fn len(&self) -> usize {
        let mut stream = self.base.stream();
        let mut base_count: usize = 0;
        while stream.next().is_some() {
            base_count += 1;
        }
        base_count.saturating_sub(self.removals.len()) + self.additions.len()
    }

    /// Return `true` if the dictionary has no entries.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ---------------------------------------------------------------------------
// DictBackend implementation
// ---------------------------------------------------------------------------

impl DictBackend for FstDict {
    fn prefix_lengths_of(&self, prefix: &CharString) -> Vec<usize> {
        self.prefix_lengths(prefix.as_str())
    }

    fn add_word(&mut self, word: &CharString) {
        self.add(word.as_str());
    }

    fn remove_word(&mut self, word: &CharString) {
        self.remove(word.as_str());
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dict(words: &[&str]) -> FstDict {
        FstDict::from_words(words.iter().copied()).unwrap()
    }

    #[test]
    fn test_prefix_lengths_basic() {
        let dict = make_dict(&["กา", "กาแฟ", "แฟ", "ดี"]);
        let lengths = dict.prefix_lengths("กาแฟดี");
        assert!(lengths.contains(&2), "กา (2 chars)");
        assert!(lengths.contains(&4), "กาแฟ (4 chars)");
        assert_eq!(lengths.len(), 2);
    }

    #[test]
    fn test_contains() {
        let dict = make_dict(&["สวัสดี", "สวัส"]);
        assert!(dict.contains("สวัสดี"));
        assert!(dict.contains("สวัส"));
        assert!(!dict.contains("ดี"));
    }

    #[test]
    fn test_add_and_remove() {
        let mut dict = make_dict(&["กา"]);
        assert!(!dict.contains("ข้าว"));
        dict.add("ข้าว");
        assert!(dict.contains("ข้าว"));
        dict.remove("ข้าว");
        assert!(!dict.contains("ข้าว"));

        assert!(dict.contains("กา"));
        dict.remove("กา");
        assert!(!dict.contains("กา"));
        // Re-add the removed base word.
        dict.add("กา");
        assert!(dict.contains("กา"));
    }

    #[test]
    fn test_prefix_with_additions() {
        let mut dict = make_dict(&["กา"]);
        dict.add("กาแฟ");
        let lengths = dict.prefix_lengths("กาแฟดี");
        assert!(lengths.contains(&2));
        assert!(lengths.contains(&4));
    }

    #[test]
    fn test_empty_input() {
        let dict = make_dict(&["ก"]);
        assert!(dict.prefix_lengths("").is_empty());
    }
}
