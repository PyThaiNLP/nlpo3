// SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

/**
 * Character-based trie for dictionary prefix lookup.
 *
 * The trie nodes branch on `char` values (Rust's native Unicode scalar),
 * so the structure works directly on UTF-8 text without any custom encoding.
 * Words are encoded implicitly in the trie paths; no separate word list is
 * kept, which reduces memory use compared with maintaining a parallel
 * `HashSet<String>`.
 *
 * For basic information on tries, see:
 *   https://en.wikipedia.org/wiki/Trie
 */
use crate::char_string::CharString;
use crate::tokenizer::dict_backend::DictBackend;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

#[derive(Debug, Clone)]
struct TrieNode {
    children: HashMap<char, Self>,
    end: bool,
}

impl Default for TrieNode {
    fn default() -> Self {
        Self::new()
    }
}

impl TrieNode {
    pub fn new() -> Self {
        Self {
            children: HashMap::default(),
            end: false,
        }
    }

    fn find_child(&self, ch: char) -> Option<&Self> {
        self.children.get(&ch)
    }

    fn find_mut_child(&mut self, ch: char) -> Option<&mut Self> {
        self.children.get_mut(&ch)
    }

    fn remove_child(&mut self, ch: char) {
        self.children.remove(&ch);
    }

    fn set_not_end(&mut self) {
        self.end = false;
    }

    fn add_word(&mut self, chars: &[char]) {
        if chars.is_empty() {
            self.end = true;
            return;
        }
        self.children
            .entry(chars[0])
            .or_default()
            .add_word(&chars[1..]);
    }

    fn remove_word(&mut self, chars: &[char]) {
        if chars.is_empty() {
            return;
        }
        let ch = chars[0];
        if let Some(child) = self.find_mut_child(ch) {
            if chars.len() == 1 {
                child.set_not_end();
            } else {
                child.remove_word(&chars[1..]);
            }
            if !child.end && child.children.is_empty() {
                self.remove_child(ch);
            }
        }
    }

    /// Return `true` if the exact sequence `chars` is a complete word.
    fn contains_word(&self, chars: &[char]) -> bool {
        if chars.is_empty() {
            return self.end;
        }
        match self.find_child(chars[0]) {
            Some(child) => child.contains_word(&chars[1..]),
            None => false,
        }
    }

    /// Append all complete words reachable from this node to `result`.
    ///
    /// `buf` accumulates the character path from the root to the current node.
    fn collect_words(&self, buf: &mut String, result: &mut Vec<String>) {
        if self.end {
            result.push(buf.clone());
        }
        // Iterate in sorted order for deterministic output.
        let mut pairs: Vec<(&char, &TrieNode)> = self.children.iter().collect();
        pairs.sort_by_key(|(ch, _)| *ch);
        for (ch, child) in pairs {
            buf.push(*ch);
            child.collect_words(buf, result);
            buf.pop();
        }
    }
}

/// Character-based trie storing a set of words.
///
/// Words are decoded to Unicode scalar values (`char`) on insertion, and the
/// trie branches on those `char` values.  This allows the trie to work
/// directly with Rust's standard UTF-8 string types without any intermediate
/// encoding: each `&str` is decoded once on the way in, and lookups compare
/// decoded `char` values.
///
/// Words are stored exclusively in the trie structure; there is no parallel
/// `HashSet` for membership tests.  This halves the per-word memory overhead
/// compared with keeping a separate word list.
#[derive(Debug, Clone)]
pub struct TrieChar {
    word_count: usize,
    root: TrieNode,
}

impl TrieChar {
    pub fn new(words: &[CharString]) -> Self {
        let mut instance = Self {
            word_count: 0,
            root: TrieNode::new(),
        };
        for word in words {
            instance.add(word);
        }
        instance
    }

    pub fn add(&mut self, word: &CharString) {
        let stripped = word.trim();
        if !stripped.is_empty() {
            let key = stripped.as_str();
            let chars: Vec<char> = key.chars().collect();
            let is_new = !self.root.contains_word(&chars);
            self.root.add_word(&chars);
            if is_new {
                self.word_count += 1;
            }
        }
    }

    pub fn remove(&mut self, word: &CharString) {
        let stripped = word.trim();
        if !stripped.is_empty() {
            let key = stripped.as_str();
            let chars: Vec<char> = key.chars().collect();
            if self.root.contains_word(&chars) {
                self.root.remove_word(&chars);
                self.word_count -= 1;
            }
        }
    }

    #[allow(dead_code)]
    pub fn contain(&self, word: &CharString) -> bool {
        let stripped = word.trim();
        if stripped.is_empty() {
            return false;
        }
        let chars: Vec<char> = stripped.as_str().chars().collect();
        self.root.contains_word(&chars)
    }

    /// Iterate over all words stored in the trie in lexicographic order.
    ///
    /// This eagerly collects all words into a `Vec` before returning the
    /// iterator; use sparingly on the hot path.
    #[allow(dead_code)]
    pub fn iterate(&self) -> impl Iterator<Item = String> {
        let mut buf = String::new();
        let mut result = Vec::new();
        self.root.collect_words(&mut buf, &mut result);
        result.into_iter()
    }

    #[allow(dead_code)]
    pub fn amount_of_words(&self) -> usize {
        self.word_count
    }

    /// Return character lengths of all dictionary entries that are prefixes
    /// of `prefix`.
    ///
    /// For example, if the dictionary contains "กข" and "กขค" and `prefix`
    /// starts with "กขคงจ", the result is `[2, 3]`.
    pub fn prefix_ref(prefix: &CharString, dict_trie: &Self) -> Vec<usize> {
        let mut result: Vec<usize> = Vec::new();
        let mut current_node = Some(&dict_trie.root);
        let n = prefix.chars_len();

        for i in 0..n {
            let ch = prefix.get_char_at(i);
            if let Some(node) = current_node
                && let Some(child) = node.find_child(ch)
            {
                if child.end {
                    result.push(i + 1); // word of length i+1 chars
                }
                current_node = Some(child);
            } else {
                break;
            }
        }
        result
    }
}

// ---------------------------------------------------------------------------
// DictBackend implementation
// ---------------------------------------------------------------------------

impl DictBackend for TrieChar {
    fn prefix_lengths_of(&self, prefix: &CharString) -> Vec<usize> {
        TrieChar::prefix_ref(prefix, self)
    }

    fn add_word(&mut self, word: &CharString) {
        self.add(word);
    }

    fn remove_word(&mut self, word: &CharString) {
        self.remove(word);
    }
}

// ---------------------------------------------------------------------------
// TrieCharLegacy — original implementation with a parallel HashSet<String>
// ---------------------------------------------------------------------------

/// Character-based trie with a parallel word set (legacy implementation).
///
/// This is the original `TrieChar` that existed before the memory-optimized
/// version.  It keeps a `HashSet<String>` alongside the trie structure,
/// which gives O(1) membership tests and O(1) word-count queries at the cost
/// of roughly 12 % higher memory use compared with [`TrieChar`].
///
/// All three backends implement [`DictBackend`], so switching is one type
/// parameter:
///
/// ```rust
/// use nlpo3::char_string::CharString;
/// use nlpo3::tokenizer::trie_char::{TrieChar, TrieCharLegacy};
/// use nlpo3::tokenizer::dict_backend::DictBackend;
///
/// // Both implement DictBackend; NewmmTokenizer accepts either.
/// let words = vec![CharString::new("กา"), CharString::new("กาแฟ")];
/// let _tc: Box<dyn DictBackend> = Box::new(TrieChar::new(&words));
/// let _tl: Box<dyn DictBackend> = Box::new(TrieCharLegacy::new(&words));
/// ```
///
/// # When to prefer `TrieCharLegacy`
///
/// - When `contain()` is called frequently on the hot path and O(1) hash
///   lookup matters more than saving ~12 % memory.
/// - When iterating words without caring about lexicographic order (the
///   iteration order of `HashSet` is unspecified).
#[derive(Debug, Clone)]
pub struct TrieCharLegacy {
    words: HashSet<String>,
    root: TrieNode,
}

impl TrieCharLegacy {
    pub fn new(words: &[CharString]) -> Self {
        let mut instance = Self {
            words: HashSet::default(),
            root: TrieNode::new(),
        };
        for word in words {
            instance.add(word);
        }
        instance
    }

    pub fn add(&mut self, word: &CharString) {
        let stripped = word.trim();
        if !stripped.is_empty() {
            let key = stripped.as_str().to_string();
            if self.words.insert(key) {
                let chars: Vec<char> = stripped.as_str().chars().collect();
                self.root.add_word(&chars);
            }
        }
    }

    pub fn remove(&mut self, word: &CharString) {
        let stripped = word.trim();
        if !stripped.is_empty() {
            let key = stripped.as_str();
            if self.words.remove(key) {
                let chars: Vec<char> = key.chars().collect();
                self.root.remove_word(&chars);
            }
        }
    }

    /// O(1) membership test via the internal `HashSet`.
    #[allow(dead_code)]
    pub fn contain(&self, word: &CharString) -> bool {
        let stripped = word.trim();
        if stripped.is_empty() {
            return false;
        }
        self.words.contains(stripped.as_str())
    }

    /// Iterate over all words stored in the dictionary.
    ///
    /// Iteration order is unspecified (hash set order).
    #[allow(dead_code)]
    pub fn iterate(&self) -> impl Iterator<Item = &String> {
        self.words.iter()
    }

    #[allow(dead_code)]
    pub fn amount_of_words(&self) -> usize {
        self.words.len()
    }

    /// Return character lengths of all dictionary entries that are prefixes
    /// of `prefix`.  Identical to [`TrieChar::prefix_ref`].
    pub fn prefix_ref(prefix: &CharString, dict_trie: &Self) -> Vec<usize> {
        let mut result: Vec<usize> = Vec::new();
        let mut current_node = Some(&dict_trie.root);
        let n = prefix.chars_len();

        for i in 0..n {
            let ch = prefix.get_char_at(i);
            if let Some(node) = current_node
                && let Some(child) = node.find_child(ch)
            {
                if child.end {
                    result.push(i + 1);
                }
                current_node = Some(child);
            } else {
                break;
            }
        }
        result
    }
}

// ---------------------------------------------------------------------------
// DictBackend implementation for TrieCharLegacy
// ---------------------------------------------------------------------------

impl DictBackend for TrieCharLegacy {
    fn prefix_lengths_of(&self, prefix: &CharString) -> Vec<usize> {
        TrieCharLegacy::prefix_ref(prefix, self)
    }

    fn add_word(&mut self, word: &CharString) {
        self.add(word);
    }

    fn remove_word(&mut self, word: &CharString) {
        self.remove(word);
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_remove_word() {
        let mut trie = TrieChar::new(&[CharString::new("ศาล")]);
        assert_eq!(trie.amount_of_words(), 1);
        trie.add(&CharString::new("ศาล"));
        assert_eq!(trie.amount_of_words(), 1);
        trie.add(&CharString::new("  ศาล "));
        assert_eq!(trie.amount_of_words(), 1);
        trie.add(&CharString::new("ศาลา"));
        assert_eq!(trie.amount_of_words(), 2);
        trie.remove(&CharString::new("ศาลา"));
        assert_eq!(trie.amount_of_words(), 1);
        trie.remove(&CharString::new("ลา"));
        assert_eq!(trie.amount_of_words(), 1);
        trie.remove(&CharString::new("ศาล"));
        assert_eq!(trie.amount_of_words(), 0);
        trie.remove(&CharString::new(""));
        assert_eq!(trie.amount_of_words(), 0);
    }

    #[test]
    fn test_prefix_ref() {
        let words = vec![
            CharString::new("ก"),
            CharString::new("กข"),
            CharString::new("กขค"),
            CharString::new("คง"),
        ];
        let trie = TrieChar::new(&words);
        let input = CharString::new("กขคงจ");
        let lengths = TrieChar::prefix_ref(&input, &trie);
        assert!(lengths.contains(&1));
        assert!(lengths.contains(&2));
        assert!(lengths.contains(&3));
        assert_eq!(lengths.len(), 3);
    }

    #[test]
    fn test_legacy_add_and_remove_word() {
        let mut trie = TrieCharLegacy::new(&[CharString::new("ศาล")]);
        assert_eq!(trie.amount_of_words(), 1);
        trie.add(&CharString::new("ศาล"));
        assert_eq!(trie.amount_of_words(), 1);
        trie.add(&CharString::new("  ศาล "));
        assert_eq!(trie.amount_of_words(), 1);
        trie.add(&CharString::new("ศาลา"));
        assert_eq!(trie.amount_of_words(), 2);
        trie.remove(&CharString::new("ศาลา"));
        assert_eq!(trie.amount_of_words(), 1);
        trie.remove(&CharString::new("ลา"));
        assert_eq!(trie.amount_of_words(), 1);
        trie.remove(&CharString::new("ศาล"));
        assert_eq!(trie.amount_of_words(), 0);
        trie.remove(&CharString::new(""));
        assert_eq!(trie.amount_of_words(), 0);
    }

    #[test]
    fn test_legacy_contain() {
        let trie = TrieCharLegacy::new(&[CharString::new("ก"), CharString::new("กข")]);
        assert!(trie.contain(&CharString::new("ก")));
        assert!(trie.contain(&CharString::new("กข")));
        assert!(!trie.contain(&CharString::new("ข")));
        assert!(!trie.contain(&CharString::new("")));
    }

    #[test]
    fn test_legacy_prefix_ref() {
        let words = vec![
            CharString::new("ก"),
            CharString::new("กข"),
            CharString::new("กขค"),
            CharString::new("คง"),
        ];
        let trie = TrieCharLegacy::new(&words);
        let input = CharString::new("กขคงจ");
        let lengths = TrieCharLegacy::prefix_ref(&input, &trie);
        assert!(lengths.contains(&1));
        assert!(lengths.contains(&2));
        assert!(lengths.contains(&3));
        assert_eq!(lengths.len(), 3);
    }

    #[test]
    fn test_legacy_and_trie_prefix_ref_agree() {
        // TrieChar and TrieCharLegacy must produce identical prefix_ref results.
        let words = vec![
            CharString::new("กา"),
            CharString::new("กาแฟ"),
            CharString::new("กาแฟร้อน"),
        ];
        let tc = TrieChar::new(&words);
        let tl = TrieCharLegacy::new(&words);
        let input = CharString::new("กาแฟร้อนมาก");
        let mut tc_result = TrieChar::prefix_ref(&input, &tc);
        let mut tl_result = TrieCharLegacy::prefix_ref(&input, &tl);
        tc_result.sort_unstable();
        tl_result.sort_unstable();
        assert_eq!(tc_result, tl_result);
    }
}
