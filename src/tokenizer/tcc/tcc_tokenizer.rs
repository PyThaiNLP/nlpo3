// SPDX-FileCopyrightText: 2024 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

/**
 * TCC (Thai Character Cluster) tokenizer.
*/
use super::tcc_rules::{LOOKAHEAD_TCC, NON_LOOKAHEAD_TCC};

use crate::four_bytes_str::custom_string::{
    CustomStringBytesSlice, FixedCharsLengthByteSlice, BYTES_PER_CHAR,
};
use rustc_hash::FxHashSet as HashSet;
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

/// Returns a set of "character" indice at the end of each token
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
                let end_bytes_index = match_length - BYTES_PER_CHAR;
                let end_char_index = end_bytes_index / BYTES_PER_CHAR;
                matched = matched.slice_by_char_indice(0, end_char_index);
                let segment_size = matched.chars_len();
                position += segment_size;
                set.insert(position);
                txt = txt.slice_by_char_indice(end_char_index, txt.chars_len());
            } else {
                let segment_size = matched.chars_len();
                position += segment_size;
                set.insert(position);
                let end_bytes_index = match_length;
                let end_char_index = end_bytes_index / BYTES_PER_CHAR;
                txt = txt.slice_by_char_indice(end_char_index, txt.chars_len());
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
#[test]
fn test_cluster_karan() {
    use crate::four_bytes_str::custom_string::CustomString;
    let kr_result = tcc_pos(CustomString::new("พิสูจน์ได้ค่ะ").raw_content());
    // ends at พิ
    assert!(kr_result.contains(&2));
    //สูจน์
    assert!(kr_result.contains(&7));
    //ได้
    assert!(kr_result.contains(&10));
    //ค่ะ
    assert!(kr_result.contains(&13));
}
// เรือน้อยลอยอยู่
#[test]
///
fn test_cluster_general_case() {
    use crate::four_bytes_str::custom_string::CustomString;
    let gen_result = tcc_pos(CustomString::new("เรือน้อยลอยอยู่").raw_content());
    //expected cluster ['เรือ', 'น้', 'อ', 'ย', 'ล', 'อ', 'ย', 'อ', 'ยู่']
    assert!(gen_result.contains(&4));
    assert!(gen_result.contains(&6));
    assert!(gen_result.contains(&7));
    assert!(gen_result.contains(&8));
    assert!(gen_result.contains(&9));
    assert!(gen_result.contains(&10));
    assert!(gen_result.contains(&11));
    assert!(gen_result.contains(&12));
    assert!(gen_result.contains(&15));
}
