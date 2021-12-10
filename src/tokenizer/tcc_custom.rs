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
use lazy_static::lazy_static;
use regex::bytes::Regex;

use crate::four_bytes_str::custom_string::{CustomStringBytesSlice, FixedCharsLengthByteSlice,BYTES_PER_CHAR};
use crate::four_bytes_str::custom_regex::{regex_pattern_to_custom_pattern,replace_tcc_symbol};

// regex crate does not support look-any-direction
// \x00 is byte value 0, every unicode character in regex is padded with \x00 to 4 bytes length
// https://www.fileformat.info/info/unicode/
// Thai characters use 3 bytes per character, so it is padded with \x00 only once.
// The following regular expressions are translated from pythainlp/tokenize/tcc.py
/**
 * 
 * 
 * _RE_TCC = (
    """\
เc็c
เcctาะ
เccีtยะ
เcc็c
เcิc์c
เcิtc
เcีtยะ?
เcืtอะ?
เctา?ะ?
cัtวะ
c[ัื]tc[ุิะ]?
c[ิุู]์
c[ะ-ู]t
c็
ct[ะาำ]?
แc็c
แcc์
แctะ
แcc็c
แccc์
โctะ
[เ-ไ]ct
""".replace(
        "c", "[ก-ฮ]"
    )
    .replace("t", "[่-๋]?")
    .split()
)

 */
lazy_static! {
    static ref NON_LOOKAHEAD_TCC: Regex = Regex::new(
        &[
            r"^\x00เ\x00[ก-ฮ]\x00็\x00[ก-ฮ]", //1
            r"^\x00เ\x00[ก-ฮ]\x00[ก-ฮ](\x00[่-๋])?\x00า\x00ะ",//2
            r"^\x00เ\x00[ก-ฮ]\x00[ก-ฮ]\x00ี(\x00[่-๋])?\x00ย\x00ะ",//3
            r"^\x00เ\x00[ก-ฮ]\x00[ก-ฮ]\x00็\x00[ก-ฮ]",//4
            r"^\x00เ\x00[ก-ฮ]\x00ิ\x00[ก-ฮ]\x00์\x00[ก-ฮ]",//5
            r"^\x00เ\x00[ก-ฮ]\x00ิ(\x00[่-๋])?\x00[ก-ฮ]",//6
            r"^\x00เ\x00[ก-ฮ]\x00ี(\x00[่-๋])?\x00ย(\x00ะ)?",//7
            r"^\x00เ\x00[ก-ฮ]\x00ื(\x00[่-๋])?\x00อ(\x00ะ)?",//8
            r"^\x00เ\x00[ก-ฮ](\x00[่-๋])?(\x00า)?(\x00ะ)?",//9
            r"^\x00[ก-ฮ]\x00ั(\x00[่-๋])?\x00ว\x00ะ",//10
            r"^\x00[ก-ฮ]\x00[ัื](\x00[่-๋])?\x00[ก-ฮ](\x00[ุิะ])?",//11
            r"^\x00[ก-ฮ]\x00[ิุู]\x00์",//12
            r"^\x00[ก-ฮ]\x00[ะ-ู](\x00[่-๋])?",//13
            r"^\x00[ก-ฮ]\x00็",//14
            r"^\x00[ก-ฮ](\x00[่-๋])?(\x00[ะาำ])?",//15
            r"^\x00แ\x00[ก-ฮ]\x00็\x00[ก-ฮ]",//16
            r"^\x00แ\x00[ก-ฮ]\x00[ก-ฮ]\x00์",//17
            r"^\x00แ\x00[ก-ฮ](\x00[่-๋])?\x00ะ",//18
            r"^\x00แ\x00[ก-ฮ]\x00[ก-ฮ]\x00็\x00[ก-ฮ]",//19
            r"^\x00แ\x00[ก-ฮ]\x00[ก-ฮ]\x00[ก-ฮ]\x00์",//20
            r"^\x00โ\x00[ก-ฮ](\x00[่-๋])?\x00ะ",//21
            r"^\x00[เ-ไ]\x00[ก-ฮ](\x00[่-๋])?",//22
            r"^(\x00เ\x00[ก-ฮ]\x00[ก-ฮ]\x00ี(\x00[่-๋])?\x00ย)\x00[เ-ไก-ฮ]", // look ahead 1
            r"^(\x00เ\x00[ก-ฮ]\x00[ิีุู](\x00[่-๋])?\x00ย)\x00[เ-ไก-ฮ]",// look ahead 2
            ]
            // .map(|pattern|{regex_pattern_to_custom_pattern(pattern).unwrap()})
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
pub fn tcc_pos(custom_text_type: &CustomStringBytesSlice) -> HashSet<usize>{
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
