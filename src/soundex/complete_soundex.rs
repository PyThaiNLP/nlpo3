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
}
