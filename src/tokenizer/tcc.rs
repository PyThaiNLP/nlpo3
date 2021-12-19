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
use lazy_static::lazy_static;
use regex::bytes::Regex;
use rustc_hash::FxHashSet as HashSet;

use crate::four_bytes_str::custom_regex::{regex_pattern_to_custom_pattern, replace_tcc_symbol};
use crate::four_bytes_str::custom_string::{
    CustomStringBytesSlice, FixedCharsLengthByteSlice, BYTES_PER_CHAR,
};

// regex crate does not support look-any-direction

lazy_static! {
    static ref NON_LOOKAHEAD_TCC: Regex = Regex::new(
        &[
            r"^เc็c", //1
            r"^เcctาะ",//2
            r"^เccีtยะ",//3
            r"^เcc็c",//4
            r"^เcิc์c",//5
            r"^เcิtc",//6
            r"^เcีtยะ?",//7
            r"^เcืtอะ?",//8
            r"^เctา?ะ?",//9
            r"^cัtวะ",//10
            r"^c[ัื]tc[ุิะ]?",//11
            r"^c[ิุู]์",//12
            r"^c[ะ-ู]t",//13
            r"^c็",//14
            r"^ct[ะาำ]?",//15
            r"^แc็c",//16
            r"^แcc์",//17
            r"^แctะ",//18
            r"^แcc็c",//19
            r"^แccc์",//20
            r"^โctะ",//21
            r"^[เ-ไ]ct",//22
            r"^(เccีtย)[เ-ไก-ฮ]", // look ahead 1
            r"^(เc[ิีุู]tย)[เ-ไก-ฮ]",// look ahead 2
            ].map(|pattern| {regex_pattern_to_custom_pattern(&replace_tcc_symbol(pattern)).unwrap()})
        .join("|")
    )
    .unwrap();
}

lazy_static! {
    static ref LOOKAHEAD_TCC: Regex = Regex::new(
        &[
            "^(เccีtย)[เ-ไก-ฮ]",//เccีtย(?=[เ-ไก-ฮ]|$)
            r"^(เc[ิีุู]tย)[เ-ไก-ฮ]",//เc[ิีุู]tย(?=[เ-ไก-ฮ]|$)
        ].map(|pattern| {regex_pattern_to_custom_pattern(&replace_tcc_symbol(pattern)).unwrap()})
        .join("|")
    )
    .unwrap();
}

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

#[allow(dead_code)]
pub fn tcc_segment(custom_text_type: &CustomStringBytesSlice) -> Vec<&CustomStringBytesSlice> {
    let mut txt = custom_text_type;
    let mut tcc_result: Vec<&[u8]> = Vec::with_capacity(txt.len() / 10);
    while !txt.is_empty() {
        if let Some(result) = NON_LOOKAHEAD_TCC.find(txt) {
            let mut matched = &txt[result.start()..result.end()];
            let match_length = matched.len();
            if LOOKAHEAD_TCC.is_match(matched) {
                // trim one more char to the right.
                let end_bytes_index = match_length - (1 * BYTES_PER_CHAR);
                matched = &matched[0..end_bytes_index];
                tcc_result.push(matched);
                txt = &txt[end_bytes_index..];
            } else {
                tcc_result.push(matched);

                let end_bytes_index = match_length;
                txt = &txt[end_bytes_index..];
            }
        } else {
            // not thai
            let first_char = txt.slice_by_char_indice(0, 1);
            tcc_result.push(first_char);
            txt = txt.slice_by_char_indice(1, txt.chars_len());
        }
    }
    tcc_result
}
