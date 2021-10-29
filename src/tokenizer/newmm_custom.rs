use super::super::fixed_bytes_str::four_bytes::{
    CustomStringBytesSlice, CustomStringBytesVec, FixedLengthCustomString,
};
use super::{
    dict_reader_custom::{create_dict_trie, DictSource},
    tcc_custom,
    tokenizer_trait::Tokenizer,
    trie_char_ver::TrieChar as Trie,
};
/**
Dictionary-based maximal matching word segmentation, constrained with
Thai Character Cluster (TCC) boundaries.

The code is based on the notebooks created by Korakot Chaovavanich,
with heuristic graph size limit added to avoid exponential wait time.

:See Also:
    * \
        https://github.com/PyThaiNLP/pythainlp/blob/dev/pythainlp/tokenize/newmm.py

Rust implementation: ["Thanathip Suntorntip"]
*/
use crate::fixed_bytes_str::four_bytes::{
    rfind_space_char_index, CustomString, FixedCharsLengthByteSlice,
    BYTES_PER_CHAR,
};
use anyhow::Result as AnyResult;
use binary_heap_plus::{BinaryHeap, MinComparator};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::bytes::Regex;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::error::Error;
use std::fmt::Display;
use std::{collections::VecDeque, path::PathBuf};
const MAX_GRAPH_SIZE: usize = 50;
const USE_MULTITHREAD_THRESHOLD: usize = 10000;

// Window size for safe mode
const TEXT_SCAN_POINT: usize = 120;
const TEXT_SCAN_LEFT: usize = 20;
const TEXT_SCAN_RIGHT: usize = 20;
const TEXT_SCAN_BEGIN: usize = TEXT_SCAN_POINT - TEXT_SCAN_LEFT;
const TEXT_SCAN_END: usize = TEXT_SCAN_POINT + TEXT_SCAN_RIGHT;

type CharacterIndex = usize;

lazy_static! {
    // Regex here is very ugly.
    // Any quantitative symbols (+ * {a,b}) must be used with grouped four bytes.
    // For example:
    // Normal String: \d+
    // Custom String: (\x00\x00\x00\d)+

    static ref NON_THAI_PATTERN: Regex = Regex::new(
        r"(?x)
        ^(\x00\x00\x00[-a-zA-Z])+| # Latin characters
        ^(\x00\x00\x00\d)+(\x00\x00\x00[,\.](\x00\x00\x00\d)+)*| # number
        ^(\x00[๐-๙])+(\x00\x00\x00[,\.](\x00[๐-๙])+)*| #  are you serious, Thai number?
        ^(\x00\x00\x00[\ \t])+| # space
        ^(\x00\x00\x00\r)?\x00\x00\x00\n  # newline" 
    )
    .unwrap();
}

lazy_static! {
    static ref THAI_TWOCHARS_PATTERN: Regex = Regex::new(r"^(\x00[ก-ฮ]){0,2}$").unwrap();
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
            self.goal, self.start, self.goal
        )
    }
}
impl Error for BFSSearchError {}
#[derive(Debug)]
pub struct Newmm {
    dict: Box<Trie>,
}
impl Newmm {
    pub fn new(dict_path: &str) -> Self {
        Self {
            dict: Box::from(
                create_dict_trie(DictSource::FilePath(PathBuf::from(dict_path))).unwrap(),
            ),
        }
    }
    fn bfs_paths_graph(
        graph: &HashMap<CharacterIndex, Vec<CharacterIndex>>,
        start: CharacterIndex,
        goal: CharacterIndex,
        current_queue: &mut VecDeque<(usize, Vec<usize>)>,
    ) -> AnyResult<Vec<CharacterIndex>> {
        current_queue.clear();

        // let mut current_queue: VecDeque<(usize, Vec<usize>)> = VecDeque::with_capacity(graph.len());

        let mut init_path: Vec<usize> = Vec::with_capacity(goal - start);
        init_path.push(start);
        current_queue.push_back((start, init_path));

        while let Some((vertex, path)) = current_queue.pop_front() {
            if let Some(idk) = graph.get(&vertex) {
                for position in idk {
                    if *position != goal {
                        let mut appended_path = path.clone();
                        appended_path.push(*position);
                        current_queue.push_back((*position, appended_path));
                    } else {
                        let mut appended_path = path;
                        appended_path.push(*position);

                        return Ok(appended_path);
                    };
                }
            };
        }

        Err(BFSSearchError::new(graph, start, goal).into())
    }

    fn one_cut<'a,'b>(
        input: &'a CustomString,
        custom_dict: &'b Trie,
    ) -> AnyResult<Vec<&'a CustomStringBytesSlice>> {
        let text = input;
        let input_char_len = text.chars_len();
        let mut reused_queue: VecDeque<(usize, Vec<usize>)> = VecDeque::with_capacity(10);
        let mut graph_size: usize = 0;
        let mut graph: HashMap<CharacterIndex, Vec<CharacterIndex>> = HashMap::default();
        graph.reserve(input_char_len / 10);
        let mut result_str: Vec<&CustomStringBytesSlice> = Vec::with_capacity(input_char_len / 10);

        // all position should be refered as character index
        let valid_position = tcc_custom::tcc_pos(text.raw_content());
        let text_length = input_char_len;
        let mut position_list: BinaryHeap<CharacterIndex, MinComparator> = BinaryHeap::new_min();
        let mut existing_candidate: HashSet<CharacterIndex> = HashSet::default();
        existing_candidate.reserve(input_char_len / 10);
        position_list.push(0);
        existing_candidate.insert(0);
        let mut end_position: CharacterIndex = 0;
        // as long as there is a value in the position_list priority queue
        // AND its value is less than text_length
        while match position_list.peek() {
            Some(position) if *position < text_length => true,
            None => false,
            _ => false,
        } {
            if let Some(begin_position) = position_list.pop() {
                let sub_text_prefix = text.substring(begin_position, text.chars_len());
                let prefixes = Trie::prefix_ref(&sub_text_prefix, custom_dict);
                for word in prefixes {
                    let word_length = word.chars_len();
                    let end_position_candidate = begin_position + word_length;
                    if valid_position.contains(&end_position_candidate) {
                        let target_graph = graph.get_mut(&begin_position);
                        match target_graph {
                            Some(existing_path) => {
                                existing_path.push(end_position_candidate);
                            }
                            None => {
                                graph.insert(begin_position, vec![end_position_candidate]);
                            }
                        }

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
                    //only one candidate!
                    if let Some(first_position_list) = position_list.peek() {
                        let group_of_end_position_candidate = Self::bfs_paths_graph(
                            &graph,
                            end_position,
                            *first_position_list,
                            &mut reused_queue,
                        )?;
                        graph_size = 0; // reset our graph

                        for position in group_of_end_position_candidate.iter().skip(1) {
                            let token_bytes = text.substring_as_bytes(end_position, *position);

                            result_str.push(token_bytes);
                            end_position = *position;
                        }
                    } else {
                        panic!("incorrect position list");
                    }
                } else if position_list_length == 0 {
                    // no candidate, deal with non-dict word
                    match NON_THAI_PATTERN.find(sub_text_prefix.raw_content()) {
                        Some(match_point) => {
                            let matched_start_char_index = match_point.start() / BYTES_PER_CHAR;
                            let matched_end_char_index = match_point.end() / BYTES_PER_CHAR;
                            //  non thai -> skip to the end of match
                            end_position = begin_position
                                + sub_text_prefix
                                    .raw_content()
                                    .slice_by_char_indice(
                                        matched_start_char_index,
                                        matched_end_char_index,
                                    )
                                    .chars_len();
                        }
                        None => {
                            // Is thai -> find min skip
                            let mut finish_without_break = true;
                            for position in begin_position + 1..text_length {
                                if valid_position.contains(&position) {
                                    let prefix = text.substring(position, text_length);

                                    let list_of_prefixes =
                                        Trie::prefix_ref(&prefix, custom_dict);
                                    let valid_word_filter = |word: &&[u8]| {
                                        let new_position = position + word.chars_len();
                                        let is_valid = valid_position.contains(&new_position);
                                        let is_two_thai_chars =
                                            THAI_TWOCHARS_PATTERN.is_match(word);
                                        is_valid && !is_two_thai_chars
                                    };
                                    let valid_words: Vec<&[u8]> =
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
                                    if NON_THAI_PATTERN.is_match(prefix.raw_content()) {
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
                    if let Some(existing_path) = graph.get_mut(&begin_position) {
                        existing_path.push(end_position);
                        graph_size += 1;
                        let token_bytes = text.substring_as_bytes(begin_position, end_position);

                        result_str.push(token_bytes);
                        position_list.push(end_position);
                        existing_candidate.insert(end_position);
                    } else {
                        let mut graph_elem: Vec<usize> = Vec::with_capacity(10);
                        graph_elem.push(end_position);
                        graph.insert(begin_position, graph_elem);
                        graph_size += 1;
                        let token_bytes = text.substring_as_bytes(begin_position, end_position);
                        result_str.push(token_bytes);
                        position_list.push(end_position);
                        existing_candidate.insert(end_position);
                    }
                }
            }
        }
        Ok(result_str)
    }

    fn internal_segment(
        input: &CustomString,
        custom_dict: &Trie,
        safe: bool,
        parallel: bool,
    ) -> AnyResult<Vec<String>> {
        if input.is_empty() {
            return Ok(vec![]);
        }
        if !safe || input.chars_len() < TEXT_SCAN_END {

            let result = Self::one_cut(input, custom_dict)?;
            Ok(if parallel {
                result
                    .into_par_iter()
                    .map(|custom_substring| {
                        CustomString::convert_raw_bytes_to_std_string(
                            custom_substring
                        )
                    })
                    .collect()
            } else {
                result
                    .into_iter()
                    .map(|custom_substring| {
                        CustomString::convert_raw_bytes_to_std_string(
                            custom_substring
                        )
                    })
                    .collect()
            })
        } else {
            let mut txt = input.substring(0, input.chars_len());
            let mut txt_parts: Vec<CustomString> = Vec::with_capacity(txt.chars_len() / 10);
            while txt.chars_len() >= TEXT_SCAN_END {
                let sample = txt.substring(TEXT_SCAN_BEGIN, TEXT_SCAN_END);

                let mut cut_pos;

                let space_char_index = rfind_space_char_index(sample.raw_content());
                // there is a space
                if let Some(space_char_index) = space_char_index {
                    cut_pos = space_char_index + 1;
                } else {
                    let word_tokens = Self::one_cut(&sample, custom_dict)?;
                    let mut token_max_index = 0;
                    let mut token_max_length = 0;
                    for (idx, token) in word_tokens.iter().enumerate() {
                        if token.chars_len() >= token_max_length {
                            token_max_length = token.chars_len();
                            token_max_index = idx;
                        }
                    }
                    // choose the position that covers longest token
                    cut_pos = TEXT_SCAN_BEGIN;
                    for i in 0..token_max_index {
                        cut_pos = cut_pos + word_tokens.get(i).unwrap().chars_len();
                    }
                }
                txt_parts.push(txt.substring(0, cut_pos));
                txt = txt.substring(cut_pos, txt.chars_len());
            }
            if !txt.is_empty() {
                txt_parts.push(txt);
            }

            Ok(if parallel {
                txt_parts
                    .par_iter()
                    .flat_map(|part| -> AnyResult<_> {
                        let bind_part = &part.substring(0, part.chars_len());
                        let words =
                            Self::one_cut(bind_part, custom_dict)?;
            
                        Ok(words
                            .into_par_iter()
                            .map(|word| {
                                CustomString::convert_raw_bytes_to_std_string(word)
                            })
                            .collect::<Vec<String>>())
                    })
                    .flatten()
                    .collect()
            } else {
                txt_parts
                    .iter()
                    .flat_map(|part| -> AnyResult<_> {
                        Ok(
                            Self::one_cut(&part.substring(0, part.chars_len()), custom_dict)?
                                .iter()
                                .map(|word| {
                                    CustomString::convert_raw_bytes_to_std_string(
                                        word,
                                    )
                                })
                                .collect::<Vec<String>>(),
                        )
                    })
                    .flatten()
                    .collect()
            })
        }
    }
}
impl Tokenizer for Newmm{
    fn segment(&self, text: &str, safe: bool, parallel: bool) -> AnyResult<Vec<String>> {
        Self::internal_segment(&CustomString::new(text), &self.dict, safe, parallel)
    }

    fn segment_to_string(&self, text: &str, safe: bool, parallel: bool) -> Vec<String> {
        self.segment(text, safe, parallel).unwrap()
    }
}