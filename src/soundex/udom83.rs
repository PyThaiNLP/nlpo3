// SPDX-FileCopyrightText: 2024 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

//! Thai soundex - Udom83 system.
//!
//! Original paper:
//! Wannee Udompanich. String searching for Thai alphabet
//! using Soundex compression technique. Master Thesis,
//! Department of Computer Engineering Graduate School,
//! Chulalongkorn University, 1983.

use lazy_static::lazy_static;
use regex::Regex;

use crate::thai_chars::THAI_CONSONANTS;

/// All 44 Thai consonants as a regex character class pattern.
fn cons_class() -> String {
    THAI_CONSONANTS.to_string()
}

lazy_static! {
    // Preprocessing regex patterns (applied in order)
    // รร + lead vowel (เ-ไ) → ัน + lead vowel
    static ref RE_1: Regex = Regex::new(r"รร([เ-ไ])").unwrap();
    // รร + consonant + (consonant or lead vowel) → ั + match
    static ref RE_2: Regex = Regex::new(&format!(
        r"รร([{c}][{c}เ-ไ])", c = cons_class()
    )).unwrap();
    // รร + consonant + (vowel or tone) → ัน + match
    static ref RE_3: Regex = Regex::new(&format!(
        r"รร([{c}][ะ-ู่-์])", c = cons_class()
    )).unwrap();
    // remaining รร → ัน
    static ref RE_4: Regex = Regex::new(r"รร").unwrap();
    // ไ + consonant + ย → consonant + ย
    static ref RE_5: Regex = Regex::new(&format!(
        r"ไ([{c}]ย)", c = cons_class()
    )).unwrap();
    // ไ/ใ + consonant → consonant + ย
    static ref RE_6: Regex = Regex::new(&format!(
        r"[ไใ]([{c}])", c = cons_class()
    )).unwrap();
    // ำ + ม + vowel → ม + vowel
    static ref RE_7: Regex = Regex::new(r"ำ(ม[ะ-ู])").unwrap();
    // ำ + ม → ม
    static ref RE_8: Regex = Regex::new(r"ำม").unwrap();
    // remaining ำ → ม
    static ref RE_9: Regex = Regex::new(r"ำ").unwrap();
    // karan patterns + consonant with ์
    static ref RE_10: Regex = Regex::new(&format!(
        r"จน์|มณ์|ณฑ์|ทร์|ตร์|[{c}]์|[{c}][ะ-ู]์", c = cons_class()
    )).unwrap();
    // remove all vowels and marks (ะ-์, U+0E30..U+0E4C)
    static ref RE_11: Regex = Regex::new(r"[ะ-์]").unwrap();
}

/// TRANS1: first-character normalization for Udom83.
fn trans1(c: char) -> char {
    match c {
        'ก' => 'ก',
        'ข' | 'ฃ' | 'ค' | 'ฅ' | 'ฆ' => 'ข',
        'ง' => 'ง',
        'จ' => 'จ',
        'ฉ' | 'ช' | 'ฌ' => 'ช',
        'ซ' | 'ศ' | 'ษ' | 'ส' => 'ส',
        'ฎ' | 'ด' => 'ด',
        'ฏ' | 'ต' => 'ต',
        'ฐ' | 'ฑ' | 'ฒ' | 'ถ' | 'ท' | 'ธ' => 'ท',
        'ณ' | 'น' => 'น',
        'บ' => 'บ',
        'ป' => 'ป',
        'ผ' | 'พ' | 'ภ' => 'พ',
        'ฝ' | 'ฟ' => 'ฟ',
        'ม' => 'ม',
        'ญ' | 'ย' => 'ย',
        'ร' | 'ล' | 'ฬ' | 'ฤ' | 'ฦ' => 'ร',
        'ว' => 'ว',
        'อ' => 'อ',
        'ห' | 'ฮ' => 'ฮ',
        _ => c,
    }
}

/// TRANS2: remainder character encoding for Udom83.
fn trans2(c: char) -> char {
    match c {
        // Group 0
        'ม' | 'ว' | 'ำ' => '0',
        // Group 1: K-class
        'ก' | 'ข' | 'ฃ' | 'ค' | 'ฅ' | 'ฆ' => '1',
        // Group 2: NG, Y
        'ง' | 'ย' => '2',
        // Group 3: nasal
        'ญ' | 'ณ' | 'น' => '3',
        // Group 4: dental
        'ฎ' | 'ฏ' | 'ด' | 'ต' | 'ศ' | 'ษ' | 'ส' => '4',
        // Group 5: labial
        'บ' | 'ป' | 'พ' | 'ภ' => '5',
        // Group 6: aspirate/labial
        'ผ' | 'ฝ' | 'ฟ' | 'ห' | 'อ' | 'ฮ' => '6',
        // Group 7: affricate/sibilant
        'จ' | 'ฉ' | 'ช' | 'ซ' | 'ฌ' => '7',
        // Group 8: dental aspirate
        'ฐ' | 'ฑ' | 'ฒ' | 'ถ' | 'ท' | 'ธ' => '8',
        // Group 9: liquid
        'ร' | 'ฤ' | 'ล' | 'ฦ' => '9',
        _ => c,
    }
}

/// Compute the Udom83 Thai soundex for the given text.
///
/// Returns a 7-character phonetic code string, or an empty string
/// if the input is empty or reduces to nothing after preprocessing.
///
/// # Examples
/// ```
/// use nlpo3::soundex::udom83;
///
/// assert_eq!(udom83("รถ"), "ร800000");
/// assert_eq!(udom83(""), "");
/// ```
pub fn udom83(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    // Apply preprocessing regex substitutions in order
    let text = RE_1.replace_all(text, "ัน$1").to_string();
    let text = RE_2.replace_all(&text, "ั$1").to_string();
    let text = RE_3.replace_all(&text, "ัน$1").to_string();
    let text = RE_4.replace_all(&text, "ัน").to_string();
    let text = RE_5.replace_all(&text, "$1").to_string();
    let text = RE_6.replace_all(&text, "${1}ย").to_string();
    let text = RE_7.replace_all(&text, "ม$1").to_string();
    let text = RE_8.replace_all(&text, "ม").to_string();
    let text = RE_9.replace_all(&text, "ม").to_string();
    let text = RE_10.replace_all(&text, "").to_string();
    let text = RE_11.replace_all(&text, "").to_string();

    if text.is_empty() {
        return String::new();
    }

    let chars: Vec<char> = text.chars().collect();

    // First char via TRANS1, rest via TRANS2, pad and truncate to 7
    let first = trans1(chars[0]);
    let rest: String = chars[1..].iter().map(|&c| trans2(c)).collect();
    let padded = format!("{}{}000000", first, rest);
    padded.chars().take(7).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udom83_empty() {
        assert_eq!(udom83(""), "");
    }

    #[test]
    fn test_udom83_karan_removes_all() {
        assert_eq!(udom83("น์"), "");
    }

    #[test]
    fn test_udom83_known_values() {
        assert_eq!(udom83("รถ"), "ร800000");
    }

    #[test]
    fn test_udom83_phonetic_equivalence() {
        assert_eq!(udom83("เหตุ"), udom83("เหด"));
    }

    #[test]
    fn test_udom83_docstring_values() {
        assert_eq!(udom83("ปัจจุบัน"), "ป775300");
        assert_eq!(udom83("ลัก"), "ร100000");
        assert_eq!(udom83("รัก"), "ร100000");
        assert_eq!(udom83("รักษ์"), "ร100000");
        assert_eq!(udom83("บูรณการ"), "บ931900");
    }

    #[test]
    fn test_udom83_common_words() {
        assert_eq!(udom83("สวัสดี"), "ส044000");
        assert_eq!(udom83("ประเทศ"), "ป984000");
        assert_eq!(udom83("กรุงเทพ"), "ก928500");
    }
}
