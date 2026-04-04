// SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

/**
 * Dictionary-based maximal matching word segmentation, constrained with
 * Thai Character Cluster (TCC) boundaries.
 *
 * The code is based on the notebooks created by Korakot Chaovavanich,
 * with heuristic graph size limit added to avoid exponential wait time.
 *
 * :See Also:
 *  * <https://github.com/PyThaiNLP/pythainlp/blob/dev/pythainlp/tokenize/newmm.py>
 *
 * Original Rust implementation: Thanathip Suntorntip
 * Rewrite using native Rust Unicode types: PyThaiNLP Project
 */
use std::{collections::VecDeque, error::Error, fmt::Display, path::PathBuf, sync::Arc};

use super::{
    dict_backend::DictBackend,
    dict_reader::{DictSource, create_dict_fst, create_dict_trie},
    parallel_helper,
    parallel_options::ParallelOptions,
    tcc::tcc_tokenizer,
    tokenizer_trait::Tokenizer,
    trie_char::TrieChar,
};
use crate::char_string::{CharString, rfind_space_char_index};

use anyhow::Result as AnyResult;
use binary_heap_plus::{BinaryHeap, MinComparator};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

const MAX_GRAPH_SIZE: usize = 50;
const USE_MULTITHREAD_THRESHOLD: usize = 10000;

// Window size to check break points for safe mode.
const TEXT_SCAN_POINT: usize = 120;
const TEXT_SCAN_LEFT: usize = 20;
const TEXT_SCAN_RIGHT: usize = 20;
const TEXT_SCAN_BEGIN: usize = TEXT_SCAN_POINT - TEXT_SCAN_LEFT;
const TEXT_SCAN_END: usize = TEXT_SCAN_POINT + TEXT_SCAN_RIGHT;

type CharacterIndex = usize;

/// Native Unicode patterns for non-Thai text segments.
const NON_THAI_READABLE_PATTERN: &[&str; 5] = &[
    r"^[-a-zA-Z]+",
    r"^[0-9]+([,\.][0-9]+)*",
    r"^[๐-๙]+([,\.][๐-๙]+)*",
    r"^[ \t]+",
    r"^\r?\n",
];

lazy_static! {
    static ref NON_THAI_PATTERN: Regex = Regex::new(&NON_THAI_READABLE_PATTERN.join("|"))
        .expect("newmm: NON_THAI_PATTERN must be a valid static regex");
    static ref THAI_TWOCHARS_PATTERN: Regex = Regex::new(r"^[ก-ฮ]{0,2}$")
        .expect("newmm: THAI_TWOCHARS_PATTERN must be a valid static regex");
}

#[derive(Clone, Debug)]
struct BFSSearchError {
    graph: HashMap<CharacterIndex, Vec<CharacterIndex>>,
    start: CharacterIndex,
    goal: CharacterIndex,
}

impl BFSSearchError {
    pub fn new(
        graph: &HashMap<CharacterIndex, Vec<CharacterIndex>>,
        start: CharacterIndex,
        goal: CharacterIndex,
    ) -> Self {
        Self {
            graph: graph.clone(),
            start,
            goal,
        }
    }
}

impl Display for BFSSearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cannot find goal position {} with start position {} with graph {:?}",
            self.goal, self.start, self.graph
        )
    }
}

impl Error for BFSSearchError {}

/// Dictionary-based maximal-matching tokenizer (CharString + TrieChar).
///
/// Uses `CharString` for native UTF-8 text and `TrieChar` for fast dictionary
/// prefix lookup.  This is the **default and fastest** combination for
/// end-to-end tokenization.
///
/// The dictionary is stored behind an [`Arc`], so **cloning a tokenizer is
/// O(1)** — no dictionary data is copied.  Multiple instances created from the
/// same word list share the dictionary in memory automatically.  Because all
/// `segment` calls take `&self`, the same instance (or a cheaply cloned copy)
/// is safe to call from multiple threads concurrently.
///
/// To use a more memory-efficient dictionary backend, see [`NewmmFstTokenizer`].
///
/// All dictionary-based tokenizers implement the shared [`Tokenizer`] trait,
/// so switching between them requires only a single line change:
///
/// ```no_run
/// use nlpo3::tokenizer::newmm::{NewmmTokenizer, NewmmFstTokenizer};
/// use nlpo3::tokenizer::tokenizer_trait::Tokenizer;
///
/// // Maximum speed (CharString + TrieChar):
/// let _tok: Box<dyn Tokenizer> = Box::new(NewmmTokenizer::new("words_th.txt").unwrap());
///
/// // Compact memory (CharString + FstDict):
/// let _tok: Box<dyn Tokenizer> = Box::new(NewmmFstTokenizer::new("words_th.txt").unwrap());
/// ```
#[derive(Debug)]
pub struct NewmmTokenizer<D: DictBackend = TrieChar> {
    dict: Arc<D>,
}

/// Memory-efficient dictionary-based tokenizer (CharString + FstDict).
///
/// Uses the same `CharString` representation and the same maximal-matching
/// algorithm as [`NewmmTokenizer`], but stores the dictionary in a minimized
/// finite-state automaton ([`FstDict`]).  This reduces dictionary memory
/// by ~49× at the cost of slower per-lookup speed.
///
/// Like [`NewmmTokenizer`], cloning is O(1) (Arc ref-count increment).
///
/// Implements the same [`Tokenizer`] trait as [`NewmmTokenizer`] and
/// `DeepcutTokenizer`, so tokenizers are interchangeable at any call site
/// that accepts `&dyn Tokenizer`.
///
/// [`FstDict`]: super::fst_dict::FstDict
#[derive(Debug)]
pub struct NewmmFstTokenizer {
    inner: NewmmTokenizer<super::fst_dict::FstDict>,
}

// ---------------------------------------------------------------------------
// Clone implementations — O(1) Arc ref-count increment, no dict copy
// ---------------------------------------------------------------------------

impl<D: DictBackend> Clone for NewmmTokenizer<D> {
    fn clone(&self) -> Self {
        NewmmTokenizer {
            dict: Arc::clone(&self.dict),
        }
    }
}

impl Clone for NewmmFstTokenizer {
    fn clone(&self) -> Self {
        NewmmFstTokenizer {
            inner: self.inner.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

impl NewmmTokenizer<TrieChar> {
    /// Create a tokenizer by loading a dictionary file (TrieChar backend).
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn new(dict_path: &str) -> AnyResult<Self> {
        Ok(NewmmTokenizer {
            dict: Arc::new(create_dict_trie(DictSource::FilePath(PathBuf::from(
                dict_path,
            )))?),
        })
    }

    /// Create a tokenizer from an in-memory word list (TrieChar backend).
    ///
    /// Construction from a word list is infallible.
    pub fn from_word_list(word_list: Vec<String>) -> Self {
        let char_word_list: Vec<CharString> =
            word_list.into_iter().map(|w| CharString::new(&w)).collect();
        NewmmTokenizer {
            dict: Arc::new(TrieChar::new(&char_word_list)),
        }
    }
}

impl NewmmFstTokenizer {
    /// Create a tokenizer by loading a dictionary file (FstDict backend).
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn new(dict_path: &str) -> AnyResult<Self> {
        Ok(Self {
            inner: NewmmTokenizer {
                dict: Arc::new(create_dict_fst(DictSource::FilePath(PathBuf::from(
                    dict_path,
                )))?),
            },
        })
    }

    /// Create a tokenizer from an in-memory word list (FstDict backend).
    ///
    /// Returns an error if the word list cannot be turned into an FST
    /// (for example, if duplicate normalization fails internally).
    pub fn from_word_list(word_list: Vec<String>) -> AnyResult<Self> {
        Ok(Self {
            inner: NewmmTokenizer {
                dict: Arc::new(create_dict_fst(DictSource::WordList(word_list))?),
            },
        })
    }

    /// Add words to the tokenizer's dictionary.
    ///
    /// If other clones share the same `Arc`, the dictionary is copied
    /// before mutation (copy-on-write); all other clones are unaffected.
    pub fn add_word(&mut self, word_list: &[&str]) {
        self.inner.add_word(word_list);
    }

    /// Remove words from the tokenizer's dictionary.
    ///
    /// If other clones share the same `Arc`, the dictionary is copied
    /// before mutation (copy-on-write); all other clones are unaffected.
    pub fn remove_word(&mut self, word_list: &[&str]) {
        self.inner.remove_word(word_list);
    }

    /// Segment text with default options (safe=false, parallel disabled).
    pub fn segment(&self, text: &str) -> AnyResult<Vec<String>> {
        self.segment_with_options(text, false, None)
    }

    /// Segment text with automatically tuned parallel chunking.
    pub fn segment_parallel(&self, text: &str, safe: bool) -> AnyResult<Vec<String>> {
        let options = ParallelOptions::auto_for_text(text.len());
        self.segment_with_options(
            text,
            safe,
            if options.enabled {
                Some(options.chunk_size)
            } else {
                None
            },
        )
    }

    /// Segment text with explicit options.
    pub fn segment_with_options(
        &self,
        text: &str,
        safe: bool,
        parallel_chunk_size: Option<usize>,
    ) -> AnyResult<Vec<String>> {
        self.inner
            .segment_with_options(text, safe, parallel_chunk_size)
    }
}

impl Tokenizer for NewmmFstTokenizer {
    fn segment(&self, text: &str) -> AnyResult<Vec<String>> {
        self.segment_with_options(text, false, None)
    }
}

impl<D: DictBackend> NewmmTokenizer<D> {
    /// Add words to the tokenizer's dictionary.
    ///
    /// If other clones share the same `Arc`, the dictionary is copied
    /// before mutation (copy-on-write); all other clones are unaffected.
    pub fn add_word(&mut self, word_list: &[&str])
    where
        D: Clone,
    {
        let dict = Arc::make_mut(&mut self.dict);
        for word in word_list {
            dict.add_word(&CharString::new(word));
        }
    }

    /// Remove words from the tokenizer's dictionary.
    ///
    /// If other clones share the same `Arc`, the dictionary is copied
    /// before mutation (copy-on-write); all other clones are unaffected.
    pub fn remove_word(&mut self, word_list: &[&str])
    where
        D: Clone,
    {
        let dict = Arc::make_mut(&mut self.dict);
        for word in word_list {
            dict.remove_word(&CharString::new(word));
        }
    }

    fn bfs_paths_graph(
        graph: &HashMap<CharacterIndex, Vec<CharacterIndex>>,
        start: CharacterIndex,
        goal: CharacterIndex,
        current_queue: &mut VecDeque<(usize, Vec<usize>)>,
    ) -> AnyResult<Vec<CharacterIndex>> {
        current_queue.clear();

        // The visited set ensures each vertex is enqueued at most once.
        // This avoids the exponential blow-up from revisiting the same nodes.
        let mut visited: HashSet<CharacterIndex> =
            HashSet::with_capacity_and_hasher(goal - start, Default::default());
        visited.insert(start);

        let mut init_path: Vec<usize> = Vec::with_capacity(goal - start);
        init_path.push(start);
        current_queue.push_back((start, init_path));

        while let Some((vertex, path)) = current_queue.pop_front() {
            if let Some(idk) = graph.get(&vertex) {
                for &position in idk {
                    if position == goal {
                        let mut appended_path = path;
                        appended_path.push(position);
                        return Ok(appended_path);
                    } else if !visited.contains(&position) {
                        visited.insert(position);
                        let mut appended_path = path.clone();
                        appended_path.push(position);
                        current_queue.push_back((position, appended_path));
                    }
                }
            };
        }

        Err(BFSSearchError::new(graph, start, goal).into())
    }

    fn one_cut<'a>(input: &'a CharString, custom_dict: &D) -> AnyResult<Vec<&'a str>> {
        let text = input;
        let input_char_len = text.chars_len();
        let mut reused_queue: VecDeque<(usize, Vec<usize>)> = VecDeque::with_capacity(10);
        let mut graph_size: usize = 0;
        let mut graph: HashMap<CharacterIndex, Vec<CharacterIndex>> = HashMap::default();
        graph.reserve(input_char_len / 10);
        let mut result_str: Vec<&str> = Vec::with_capacity(input_char_len / 10);

        // All positions are character indices (not byte offsets).
        let valid_position = tcc_tokenizer::tcc_pos(text.as_str());
        let is_valid_position = |pos: usize| valid_position.binary_search(&pos).is_ok();
        let text_length = input_char_len;
        let mut position_list: BinaryHeap<CharacterIndex, MinComparator> = BinaryHeap::new_min();
        let mut existing_candidate: HashSet<CharacterIndex> = HashSet::default();
        existing_candidate.reserve(input_char_len / 10);
        position_list.push(0);
        existing_candidate.insert(0);
        let mut end_position: CharacterIndex = 0;

        while let Some(&begin_position) = position_list.peek()
            && begin_position < text_length
        {
            position_list.pop();
            let sub_text_prefix = text.substring(begin_position, text.chars_len());
            let prefixes = custom_dict.prefix_lengths_of(&sub_text_prefix);
            for word_length in prefixes {
                let end_position_candidate = begin_position + word_length;
                if is_valid_position(end_position_candidate) {
                    graph
                        .entry(begin_position)
                        .or_default()
                        .push(end_position_candidate);

                    graph_size += 1;
                    if !existing_candidate.contains(&end_position_candidate) {
                        existing_candidate.insert(end_position_candidate);
                        position_list.push(end_position_candidate);
                    }
                    if graph_size > MAX_GRAPH_SIZE {
                        break;
                    }
                }
            }
            let position_list_length = position_list.len();
            if position_list_length == 1 {
                // Only one candidate left.
                if let Some(&first_position_list) = position_list.peek() {
                    let group_of_end_position_candidate = Self::bfs_paths_graph(
                        &graph,
                        end_position,
                        first_position_list,
                        &mut reused_queue,
                    )?;
                    graph_size = 0;
                    graph.clear();

                    for &position in group_of_end_position_candidate.iter().skip(1) {
                        let token = text.substring_as_str(end_position, position);
                        result_str.push(token);
                        end_position = position;
                    }
                } else {
                    return Err(anyhow::anyhow!(
                        "newmm invariant violated: expected one candidate in position_list"
                    ));
                }
            } else if position_list_length == 0 {
                // No candidate: handle non-dictionary segment.
                let sub_str = sub_text_prefix.as_str();
                match NON_THAI_PATTERN.find(sub_str) {
                    // Non-Thai text: skip to end of match.
                    Some(match_point) => {
                        let matched_char_count = sub_str[..match_point.end()].chars().count();
                        end_position = begin_position + matched_char_count;
                    }
                    // Thai text with no dictionary match: find minimum skip.
                    None => {
                        let mut finish_without_break = true;
                        for position in begin_position + 1..text_length {
                            if is_valid_position(position) {
                                let prefix = text.substring(position, text_length);

                                let list_of_prefixes = custom_dict.prefix_lengths_of(&prefix);
                                let valid_word_filter = |word_length: &usize| {
                                    let new_position = position + word_length;
                                    let is_valid = is_valid_position(new_position);
                                    let word_str =
                                        text.substring_as_str(position, position + word_length);
                                    let is_two_thai_chars =
                                        THAI_TWOCHARS_PATTERN.is_match(word_str);
                                    is_valid && !is_two_thai_chars
                                };
                                let valid_words: Vec<usize> =
                                    if list_of_prefixes.len() >= USE_MULTITHREAD_THRESHOLD {
                                        list_of_prefixes
                                            .into_par_iter()
                                            .filter(valid_word_filter)
                                            .collect()
                                    } else {
                                        list_of_prefixes
                                            .into_iter()
                                            .filter(valid_word_filter)
                                            .collect()
                                    };

                                if !valid_words.is_empty() {
                                    end_position = position;
                                    finish_without_break = false;
                                    break;
                                };
                                if NON_THAI_PATTERN.is_match(prefix.as_str()) {
                                    end_position = position;
                                    finish_without_break = false;
                                    break;
                                }
                            }
                        }
                        if finish_without_break {
                            end_position = text_length;
                        }
                    }
                }

                graph_size = 0;
                graph.clear();
                let token = text.substring_as_str(begin_position, end_position);
                result_str.push(token);
                position_list.push(end_position);
                existing_candidate.insert(end_position);
            }
        }
        Ok(result_str)
    }

    fn segment_single(input: &CharString, custom_dict: &D, safe: bool) -> AnyResult<Vec<String>> {
        if !safe || input.chars_len() < TEXT_SCAN_END {
            return Self::one_cut(input, custom_dict)
                .map(|parts| parts.into_iter().map(|s| s.to_string()).collect());
        }

        let mut txt = input.substring(0, input.chars_len());
        let mut txt_parts: Vec<CharString> = Vec::with_capacity(txt.chars_len() / 10);
        while txt.chars_len() >= TEXT_SCAN_END {
            let sample = txt.substring(TEXT_SCAN_BEGIN, TEXT_SCAN_END);
            let cut_pos = if let Some(space_char_index) = rfind_space_char_index(sample.as_str()) {
                space_char_index + 1
            } else {
                let word_tokens = Self::one_cut(&sample, custom_dict)?;
                let mut token_max_index = 0;
                let mut token_max_length = 0;
                for (idx, &token) in word_tokens.iter().enumerate() {
                    let tok_chars = token.chars().count();
                    if tok_chars >= token_max_length {
                        token_max_length = tok_chars;
                        token_max_index = idx;
                    }
                }
                let mut cut_pos = TEXT_SCAN_BEGIN;
                for &token in word_tokens.iter().take(token_max_index) {
                    cut_pos += token.chars().count();
                }
                cut_pos
            };

            txt_parts.push(txt.substring(0, cut_pos));
            txt = txt.substring(cut_pos, txt.chars_len());
        }
        if !txt.is_empty() {
            txt_parts.push(txt);
        }

        let mut out: Vec<String> = Vec::new();
        for part in &txt_parts {
            let chunk = part.substring(0, part.chars_len());
            let words = Self::one_cut(&chunk, custom_dict)?;
            out.extend(words.into_iter().map(|s| s.to_string()));
        }
        Ok(out)
    }

    fn internal_segment(
        input: &CharString,
        custom_dict: &D,
        safe: bool,
        parallel_chunk_size: Option<usize>,
    ) -> AnyResult<Vec<String>> {
        if input.is_empty() {
            return Ok(vec![]);
        }
        let text = input.as_str();
        let parallel_options = ParallelOptions::from_chunk_size(parallel_chunk_size);
        let should_parallelize = parallel_options.should_parallelize(text.len());
        if !should_parallelize {
            return Self::segment_single(input, custom_dict, safe);
        }

        let tcc_positions = tcc_tokenizer::tcc_pos(text);
        let chunk_ranges = parallel_helper::split_text_into_chunk_ranges(
            text,
            parallel_options.chunk_size,
            &tcc_positions,
        );

        let token_vecs: Vec<Vec<String>> = chunk_ranges
            .into_par_iter()
            .map(|(start, end)| {
                let chunk = input.substring(start, end);
                Self::segment_single(&chunk, custom_dict, safe)
            })
            .collect::<AnyResult<Vec<_>>>()?;

        Ok(parallel_helper::flatten_tokens(token_vecs))
    }
}

impl<D: DictBackend> NewmmTokenizer<D> {
    /// Segment text with default options (safe=false, parallel disabled).
    pub fn segment(&self, text: &str) -> AnyResult<Vec<String>> {
        self.segment_with_options(text, false, None)
    }

    /// Segment text with automatically tuned parallel chunking.
    pub fn segment_parallel(&self, text: &str, safe: bool) -> AnyResult<Vec<String>> {
        let options = ParallelOptions::auto_for_text(text.len());
        self.segment_with_options(
            text,
            safe,
            if options.enabled {
                Some(options.chunk_size)
            } else {
                None
            },
        )
    }

    /// Segment text with explicit options.
    ///
    /// When parallel mode is active, text is split into chunks before
    /// tokenization. Token sequences near chunk boundaries may differ from
    /// full-text results. This is acceptable for throughput-oriented tasks
    /// such as text classification and word embedding, but may not be suitable
    /// for tasks that require precise linguistic unit identification.
    pub fn segment_with_options(
        &self,
        text: &str,
        safe: bool,
        parallel_chunk_size: Option<usize>,
    ) -> AnyResult<Vec<String>> {
        Self::internal_segment(
            &CharString::new(text),
            &*self.dict,
            safe,
            parallel_chunk_size,
        )
    }
}

impl<D: DictBackend> Tokenizer for NewmmTokenizer<D> {
    fn segment(&self, text: &str) -> AnyResult<Vec<String>> {
        self.segment_with_options(text, false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_word_list() -> Vec<String> {
        vec![
            "ภาษา".to_string(),
            "ไทย".to_string(),
            "ทดสอบ".to_string(),
            "การ".to_string(),
            "ตัด".to_string(),
            "คำ".to_string(),
        ]
    }

    #[test]
    fn newmm_defaults_match_explicit_options() {
        let tok = NewmmTokenizer::from_word_list(sample_word_list());
        let text = "ภาษาไทยทดสอบการตัดคำ";

        let via_default = tok.segment(text).unwrap();
        let via_explicit = tok.segment_with_options(text, false, None).unwrap();
        let via_result = tok.segment_with_options(text, false, None).unwrap();

        assert_eq!(via_default, via_explicit);
        assert_eq!(via_explicit, via_result);
    }

    #[test]
    fn newmm_trait_and_inherent_segment_are_consistent() {
        let tok = NewmmTokenizer::from_word_list(sample_word_list());
        let text = "ภาษาไทยทดสอบการตัดคำ";

        let via_inherent = tok.segment(text).unwrap();
        let via_trait = <NewmmTokenizer<TrieChar> as Tokenizer>::segment(&tok, text).unwrap();

        assert_eq!(via_inherent, via_trait);
    }

    #[test]
    fn newmm_fst_defaults_match_explicit_options() {
        let tok = NewmmFstTokenizer::from_word_list(sample_word_list()).unwrap();
        let text = "ภาษาไทยทดสอบการตัดคำ";

        let via_default = tok.segment(text).unwrap();
        let via_explicit = tok.segment_with_options(text, false, None).unwrap();
        let via_result = tok.segment_with_options(text, false, None).unwrap();

        assert_eq!(via_default, via_explicit);
        assert_eq!(via_explicit, via_result);
    }

    #[test]
    fn newmm_fst_trait_and_inherent_segment_are_consistent() {
        let tok = NewmmFstTokenizer::from_word_list(sample_word_list()).unwrap();
        let text = "ภาษาไทยทดสอบการตัดคำ";

        let via_inherent = tok.segment(text).unwrap();
        let via_trait = <NewmmFstTokenizer as Tokenizer>::segment(&tok, text).unwrap();

        assert_eq!(via_inherent, via_trait);
    }
}
