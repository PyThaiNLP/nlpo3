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
// TODO: use slice_by_chars_indice on &[u8]
use crate::fixed_bytes_str::four_bytes::{
    rfind_space_char_index, CustomString, FixedCharsLengthByteSlice, BYTES_PER_CHAR,
};

use super::super::fixed_bytes_str::four_bytes::{CustomStringBytesSlice, CustomStringBytesVec};
use super::{
    dict_reader_custom::{create_default_dict, create_dict_trie, DictSource},
    tcc_custom,
    tokenizer_trait::Tokenizer,
    trie_custom::Trie,
};
use ahash::{AHashMap as HashMap, AHashSet as HashSet};
use binary_heap_plus::{BinaryHeap, MinComparator};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::bytes::Regex;
use std::{collections::VecDeque, path::PathBuf};
const MAX_GRAPH_SIZE: usize = 50;
const USE_MULTITHREAD_THRESHOLD: usize = 1000000;

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
        ^(\x00\x00\x00[\ \t])+| # space
        ^(\x00\x00\x00\r)?\x00\x00\x00\n  # newline" 
    )
    .unwrap();
}

lazy_static! {
    static ref THAI_TWOCHARS_PATTERN: Regex = Regex::new(r"^(\x00[ก-ฮ]){0,2}$").unwrap();
}

pub struct Newmm {
    dict: Box<Trie>,
}

impl Newmm {
    pub fn new(dict_path: Option<&str>) -> Self {
        match dict_path {
            None => Self {
                dict: Box::from(create_default_dict()),
            },
            Some(path) => Self {
                dict: Box::from(
                    create_dict_trie(DictSource::FilePath(PathBuf::from(path))).unwrap(),
                ),
            },
        }
    }

    fn bfs_paths_graph(
        graph: &HashMap<CharacterIndex, Vec<CharacterIndex>>,
        start: CharacterIndex,
        goal: CharacterIndex,
        current_queue: &mut VecDeque<(usize, Vec<usize>)>,
    ) -> Vec<CharacterIndex> {
        current_queue.clear();

        // let mut current_queue: VecDeque<(usize, Vec<usize>)> = VecDeque::with_capacity(graph.len());

        let mut init_path: Vec<usize> = Vec::with_capacity(goal - start);
        init_path.push(start);
        current_queue.push_back((start, init_path));
        while current_queue.len() > 0 {
            let (vertex, path) = current_queue.pop_front().unwrap();
            if let Some(idk) = graph.get(&vertex) {
                for position in idk {
                    if *position != goal {
                        let mut appended_path = path.clone();
                        appended_path.push(*position);
                        current_queue.push_back((*position, appended_path));
                    } else {
                        let mut appended_path = path;
                        appended_path.push(*position);

                        return appended_path;
                    };
                }
            };
        }
        panic!("something wrong");
    }

    fn one_cut(input: &CustomStringBytesSlice, custom_dict: &Trie) -> Vec<CustomStringBytesVec> {
        let text = input;
        let input_char_len = text.chars_len();
        let mut reused_queue: VecDeque<(usize, Vec<usize>)> = VecDeque::with_capacity(10);
        let mut graph_size: usize = 0;
        let mut graph: HashMap<CharacterIndex, Vec<CharacterIndex>> =
            HashMap::with_capacity(input_char_len / 100);
        let mut result_str: Vec<CustomStringBytesVec> = Vec::with_capacity(input_char_len / 100);

        // all position should be refered as character index
        let valid_position = tcc_custom::tcc_pos(input);
        let text_length = input_char_len;
        let mut position_list: BinaryHeap<CharacterIndex, MinComparator> = BinaryHeap::new_min();
        let mut existing_candidate: HashSet<CharacterIndex> = HashSet::with_capacity(50);
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
                let sub_text_prefix = text.slice_by_char_indice(begin_position, text.chars_len());
                let prefixes = custom_dict.prefix(sub_text_prefix);
                for word in prefixes {
                    let word_length = word.as_slice().chars_len();
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
                        );
                        graph_size = 0; // reset our graph

                        for position in group_of_end_position_candidate.iter().skip(1) {
                            let token = text.slice_by_char_indice(end_position, *position);

                            result_str.push(Vec::from(token));
                            end_position = *position;
                        }
                    } else {
                        panic!("incorrect position list");
                    }
                } else if position_list_length == 0 {
                    // no candidate, deal with non-dict word
                    match NON_THAI_PATTERN.find(sub_text_prefix) {
                        Some(match_point) => {
                            let matched_start_char_index = match_point.start() / BYTES_PER_CHAR;
                            let matched_end_char_index = match_point.end() / BYTES_PER_CHAR;
                            //  non thai -> skip to the end of match
                            end_position = begin_position
                                + sub_text_prefix
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
                                    let prefix = &text.slice_by_char_indice(position, text_length);

                                    let list_of_prefixes = custom_dict.prefix(&prefix);
                                    let valid_word_filter = |word: &Vec<u8>| {
                                        let new_position = position + word.as_slice().chars_len();
                                        let is_valid = valid_position.contains(&new_position);
                                        let is_two_thai_chars =
                                            THAI_TWOCHARS_PATTERN.is_match(&word);
                                        is_valid && !is_two_thai_chars
                                    };
                                    let valid_words: Vec<Vec<u8>> =
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

                                    if valid_words.len() > 0 {
                                        end_position = position;
                                        finish_without_break = false;
                                        break;
                                    };
                                    if NON_THAI_PATTERN.is_match(&prefix) {
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
                    let current_graph_opt = graph.get_mut(&begin_position);
                    match current_graph_opt {
                        Some(existing_path) => {
                            existing_path.push(end_position);
                            graph_size += 1;
                            let token = text.slice_by_char_indice(begin_position, end_position);

                            result_str.push(Vec::from(token));
                            position_list.push(end_position);
                            existing_candidate.insert(end_position);
                        }
                        None => {
                            let mut graph_elem: Vec<usize> = Vec::with_capacity(10);
                            graph_elem.push(end_position);
                            graph.insert(begin_position, graph_elem);
                            graph_size += 1;
                            let token = text.slice_by_char_indice(begin_position, end_position);
                            result_str.push(Vec::from(token));
                            position_list.push(end_position);
                            existing_candidate.insert(end_position);
                        }
                    }
                }
            }
        }

        result_str.shrink_to_fit();
        result_str
    }
    pub fn internal_segment(
        input: &CustomString,
        custom_dict: &Trie,
        safe: bool,
        parallel: bool,
    ) -> Vec<String> {
        if input.len() == 0 {
            return vec![];
        }
        if !safe || input.chars_len() < TEXT_SCAN_END {
            let result = Self::one_cut(input.raw_content(), custom_dict);
            return if parallel {
                result
                    .into_par_iter()
                    .map(|custom_string_bytes| {
                        CustomString::convert_raw_bytes_to_std_string(&custom_string_bytes)
                    })
                    .collect()
            } else {
                result
                    .into_iter()
                    .map(|custom_string_bytes| {
                        CustomString::convert_raw_bytes_to_std_string(&custom_string_bytes)
                    })
                    .collect()
            };
        } else {
            let mut txt = input.raw_content();
            let mut txt_parts: Vec<CustomStringBytesVec> = Vec::with_capacity(txt.len() / 10);
            while txt.chars_len() >= TEXT_SCAN_END {
                let sample: &[u8] = txt.slice_by_char_indice(TEXT_SCAN_BEGIN, TEXT_SCAN_END);

                let mut cut_pos;

                let space_char_index = rfind_space_char_index(sample);
                // there is a space
                if let Some(space_char_index) = space_char_index {
                    cut_pos = space_char_index + 1;
                } else {
                    let word_tokens = Self::one_cut(sample, &custom_dict);
                    let mut token_max_index = 0;
                    let mut token_max_length = 0;
                    for (idx, token) in word_tokens.iter().enumerate() {
                        if token.as_slice().chars_len() >= token_max_length {
                            token_max_length = token.as_slice().chars_len();
                            token_max_index = idx;
                        }
                    }
                    // choose the position that covers longest token
                    cut_pos = TEXT_SCAN_BEGIN;
                    for i in 0..token_max_index {
                        cut_pos = cut_pos + word_tokens.get(i).unwrap().as_slice().chars_len();
                    }
                }
                txt_parts.push(txt.slice_by_char_indice(0, cut_pos).to_owned());
                txt = txt.slice_by_char_indice(cut_pos, txt.chars_len());
            }
            if txt.len() > 0 {
                txt_parts.push(txt.to_owned());
            }

            if parallel {
                txt_parts
                    .into_par_iter()
                    .flat_map(|part| {
                        Self::one_cut(&part, &custom_dict)
                            .into_par_iter()
                            .map(|word| CustomString::convert_raw_bytes_to_std_string(&word))
                            .collect::<Vec<String>>()
                    })
                    .collect::<Vec<String>>()
            } else {
                txt_parts
                    .iter()
                    .flat_map(|part| {
                        Self::one_cut(&part, &custom_dict)
                            .iter()
                            .map(|word| CustomString::convert_raw_bytes_to_std_string(&word))
                            .collect::<Vec<String>>()
                    })
                    .collect::<Vec<String>>()
            }
        }
    }
}

impl Tokenizer for Newmm {
    fn segment(&self, text: &str, safe: Option<bool>, parallel: Option<bool>) -> Vec<String> {
        let safe_flag = match safe {
            Some(val) => val,
            None => false,
        };
        let parallel_flag = match parallel {
            Some(val) => val,
            _ => false,
        };
        let custom_string = CustomString::new(text);
        let tokens = Self::internal_segment(&custom_string, &self.dict, safe_flag, parallel_flag);
        tokens
    }

    fn segment_to_string(
        &self,
        text: &str,
        safe: Option<bool>,
        parallel: Option<bool>,
    ) -> Vec<String> {
        self.segment(text, safe, parallel)
    }
}
