/**
The implementation of tokenizer according to Thai Character Clusters (TCCs)
rules purposed by `Theeramunkong et al. 2000. \
    <http://citeseerx.ist.psu.edu/viewdoc/summary?doi=10.1.1.59.2548>`_

Credits:
    * TCC: Jakkrit TeCho
    * Grammar: Wittawat Jitkrittum (`link to the source file \
      <https://github.com/wittawatj/jtcc/blob/master/TCC.g>`_)
    * Python code: Korakot Chaovavanich
    * Rust Code Translation: Thanathip Suntorntip
*/


use rustc_hash::FxHashSet as HashSet;

use crate::four_bytes_str::custom_regex::{regex_pattern_to_custom_pattern};
use crate::four_bytes_str::custom_string::{
    CustomStringBytesSlice, FixedCharsLengthByteSlice, BYTES_PER_CHAR,
};
use super::tcc_rules::{L,NON_LOOKAHEAD_TCC};

pub fn tcc_pos(custom_text_type: &CustomStringBytesSlice) -> HashSet<usize> {
    let mut set: HashSet<usize> = HashSet::default();
    set.reserve(custom_text_type.chars_len() / 10);
    let mut txt = custom_text_type;
    let mut position: usize = 0;
    while !txt.is_empty() {
        if let Some(result) = NON_LOOKAHEAD_TCC.find(txt) {
            let mut matched = &txt[result.start()..result.end()];
            let match_length = matched.len();
            if LOOKAHEAD_TCC.is_match(matched) {
                // trim one more char to the right.
                let end_bytes_index = match_length - (1 * BYTES_PER_CHAR);
                matched = &matched[0..end_bytes_index];
                let segment_size = matched.chars_len();
                position += segment_size;
                set.insert(position);
                txt = &txt[end_bytes_index..];
            } else {
                let segment_size = matched.chars_len();
                position += segment_size;
                set.insert(position);
                let end_bytes_index = match_length;
                txt = &txt[end_bytes_index..];
            }
        } else {
            // not thai
            let first_char = txt.slice_by_char_indice(0, 1);
            let segment_size = first_char.chars_len();
            position += segment_size;
            set.insert(position);
            txt = txt.slice_by_char_indice(1, txt.chars_len());
        }
    }
    set
}

// pub fn tcc_pos_p(custom_text_type: &CustomStringBytesSlice) -> HashSet<usize> {

// }