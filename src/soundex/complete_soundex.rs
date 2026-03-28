// SPDX-FileCopyrightText: 2024 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

//! Complete Soundex for Thai Words Similarity Analysis.
//!
//! Reference:
//! Chalermpol Tapsai, Phayung Meesad, and Choochart Haruechaiyasak. 2020.
//! Complete Soundex for Thai Words Similarity Analysis.
//! Information Technology Journal KMUTNB. 16(1):46-59.

use lazy_static::lazy_static;
use regex::Regex;

use crate::thai_chars::is_thai_consonant;

/// Initial consonant map (Table 5.1).
fn initial_code(c: char) -> &'static str {
    match c {
        'ก' => "กก", 'ข' | 'ฃ' => "คข", 'ค' | 'ฅ' | 'ฆ' => "คค",
        'ง' => "งง", 'จ' => "จจ",
        'ฉ' | 'ช' | 'ฌ' => "ชช",
        'ซ' => "ซซ", 'ศ' | 'ษ' | 'ส' => "ซศ",
        'ญ' | 'ย' => "ยย",
        'ด' | 'ฎ' => "ดด", 'ต' | 'ฏ' => "ตต",
        'ถ' | 'ฐ' => "ทธ", 'ท' | 'ธ' | 'ฑ' | 'ฒ' => "ทท",
        'น' | 'ณ' => "นน",
        'บ' => "บบ", 'ป' => "ปป",
        'ผ' => "พผ", 'ฝ' => "ฟฝ", 'พ' | 'ภ' => "พพ", 'ฟ' => "ฟฟ",
        'ม' => "มม",
        'ร' | 'ล' | 'ฬ' | 'ฤ' => "รร", 'ว' => "วว",
        'ห' | 'ฮ' => "ฮห", 'อ' => "ออ",
        _ => "xx",
    }
}

/// Final consonant map (Table 5.2) — normalizes to 8 ending classes.
fn final_code(c: char) -> &'static str {
    match c {
        'ก' | 'ข' | 'ค' | 'ฆ' => "ก",
        'ง' => "ง",
        'จ' | 'ช' | 'ซ' | 'ด' | 'ต' | 'ถ' | 'ท' | 'ธ'
        | 'ศ' | 'ษ' | 'ส' | 'ฎ' | 'ฏ' | 'ฐ' | 'ฑ' | 'ฒ' => "ด",
        'น' | 'ณ' | 'ญ' | 'ร' | 'ล' | 'ฬ' => "น",
        'บ' | 'ป' | 'พ' | 'ฟ' | 'ภ' => "บ",
        'ม' => "ม",
        'ย' => "ย",
        'ว' => "ว",
        _ => "-",
    }
}

/// Vowel map (Table 5.3).
fn vowel_code(v: &str) -> &'static str {
    match v {
        "ะ" | "ั" | "รร" | "ำ" | "ไ" | "ใ" | "เา" => "1A",
        "า" => "1B",
        "ิ" => "2C", "ี" => "2D",
        "ึ" => "3E", "ื" => "3F",
        "ุ" => "4G", "ู" => "4H",
        "เะ" | "เ็" => "5I", "เ" => "5J",
        "แะ" | "แ็" => "6K", "แ" => "6L",
        "โะ" => "7M", "โ" => "7N",
        "เาะ" | "อ" => "8O",
        "เอะ" => "9Q", "เอ" => "9R",
        "เอียะ" => "AS", "เอีย" => "AT",
        "เอือะ" => "BU", "เอือ" => "BV",
        "อัวะ" => "CW", "อัว" | "ว" => "CX",
        _ => "",
    }
}

/// Tone mark map (Table 5.4).
fn tone_code(c: char) -> &'static str {
    match c {
        '่' => "1", // U+0E48
        '้' => "2", // U+0E49
        '๊' => "3", // U+0E4A
        '๋' => "4", // U+0E4B
        _ => "0",
    }
}

lazy_static! {
    /// Karan pattern: consonant + optional vowel + ์
    static ref RE_KARAN: Regex = Regex::new(r"[ก-ฮ][ะ-ู]?์").unwrap();
    /// Two Thai consonants (for heuristic split)
    static ref RE_TWO_CONS: Regex = Regex::new(r"^[ก-ฮ]{2}$").unwrap();
    /// Three Thai consonants
    static ref RE_THREE_CONS: Regex = Regex::new(r"^[ก-ฮ]{3}$").unwrap();
    /// Three consonants + vowel
    static ref RE_THREE_CONS_VOWEL: Regex = Regex::new(r"^[ก-ฮ]{3}[า-ู]$").unwrap();
    /// Aksorn Nam with Ro Han pattern
    static ref RE_AKSORN_NAM: Regex = Regex::new(r"^[ขฃฉฐถผฝศษสฮกจดตฎฏบปอ]วรร").unwrap();
}

/// Remove silent characters (karan/thanthakhat) from text.
fn clean_text(text: &str) -> String {
    RE_KARAN.replace_all(text, "").to_string()
}

/// Split text using heuristic rules when syllable tokenizer is unavailable.
///
/// Returns: Vec<(syllable, implicit_vowel_rule)>
/// where rule is Some("a"), Some("o"), or None.
fn heuristic_split(text: &str) -> Vec<(&str, Option<&str>)> {
    // 0. อัต pattern
    if text.starts_with("อัต") && text.chars().count() > 3 {
        let rest_start = text.char_indices().nth(3).map(|(i, _)| i).unwrap_or(text.len());
        return vec![
            (&text[..rest_start], None),
            // Prepend ต to the rest
        ];
        // This needs special handling - we'll allocate for this case
    }

    // 1. Aksorn Nam with Ro Han (e.g. สวรรค์ -> ส-วรรค์)
    if RE_AKSORN_NAM.is_match(text) {
        let first_end = text.char_indices().nth(1).map(|(i, _)| i).unwrap_or(text.len());
        return vec![
            (&text[..first_end], Some("a")),
            (&text[first_end..], None),
        ];
    }

    // 2. Two consonants without vowel (e.g. กม -> ก-a ม-a)
    if RE_TWO_CONS.is_match(text) {
        let mid = text.char_indices().nth(1).map(|(i, _)| i).unwrap_or(text.len());
        return vec![
            (&text[..mid], Some("a")),
            (&text[mid..], Some("a")),
        ];
    }

    // 3. Three consonants -> C1-a C2C3-o
    if RE_THREE_CONS.is_match(text) {
        let first_end = text.char_indices().nth(1).map(|(i, _)| i).unwrap_or(text.len());
        return vec![
            (&text[..first_end], Some("a")),
            (&text[first_end..], Some("o")),
        ];
    }

    // 4. Three consonants + vowel -> C1-a C2-a C3V
    if RE_THREE_CONS_VOWEL.is_match(text) {
        let c1_end = text.char_indices().nth(1).map(|(i, _)| i).unwrap_or(text.len());
        let c2_end = text.char_indices().nth(2).map(|(i, _)| i).unwrap_or(text.len());
        return vec![
            (&text[..c1_end], Some("a")),
            (&text[c1_end..c2_end], Some("a")),
            (&text[c2_end..], None),
        ];
    }

    // Default: single syllable
    vec![(text, None)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_code() {
        assert_eq!(initial_code('ก'), "กก");
        assert_eq!(initial_code('ศ'), "ซศ");
        assert_eq!(initial_code('ส'), "ซศ");
        assert_eq!(initial_code('ท'), "ทท");
    }

    #[test]
    fn test_final_code() {
        assert_eq!(final_code('ก'), "ก");
        assert_eq!(final_code('น'), "น");
        assert_eq!(final_code('ม'), "ม");
        assert_eq!(final_code('ด'), "ด");
    }

    #[test]
    fn test_tone_code() {
        assert_eq!(tone_code('่'), "1");
        assert_eq!(tone_code('้'), "2");
        assert_eq!(tone_code('ก'), "0");
    }

    #[test]
    fn test_clean_text() {
        assert_eq!(clean_text("รักษ์"), "รัก");
        assert_eq!(clean_text("สวรรค์"), "สวรร");
        assert_eq!(clean_text("ก้าน"), "ก้าน"); // no karan, unchanged
    }

    #[test]
    fn test_heuristic_split_two_cons() {
        let result = heuristic_split("กม");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("ก", Some("a")));
        assert_eq!(result[1], ("ม", Some("a")));
    }

    #[test]
    fn test_heuristic_split_three_cons() {
        let result = heuristic_split("กมล");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("ก", Some("a")));
        assert_eq!(result[1], ("มล", Some("o")));
    }

    #[test]
    fn test_heuristic_split_single() {
        let result = heuristic_split("ก้าน");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], ("ก้าน", None));
    }
}
