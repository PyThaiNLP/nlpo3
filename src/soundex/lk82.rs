// SPDX-FileCopyrightText: 2024 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

//! Thai soundex - LK82 system.
//!
//! Original paper:
//! Vichit Lorchirachoonkul. 1982. A Thai soundex system.
//! Information Processing & Management, 18(5):243-255.
//! <https://doi.org/10.1016/0306-4573(82)90003-6>

use lazy_static::lazy_static;
use regex::Regex;

use crate::thai_chars::{is_thai_consonant, remove_tonemarks};

lazy_static! {
    /// Silenced consonant patterns (karan).
    static ref RE_KARANT: Regex =
        Regex::new(r"จน์|มณ์|ณฑ์|ทร์|ตร์|[ก-ฮ]์|[ก-ฮ][ะ-ู]์").unwrap();
    /// Signs/symbols to remove: ฯ ์ ๆ ็ ํ (U+0E2F, U+0E3A, U+0E46, U+0E47, U+0E4D).
    static ref RE_SIGN: Regex =
        Regex::new(r"[ฯ\u{0E3A}ๆ็ํ]").unwrap();
}

/// TRANS1: first-character normalization (consonant class grouping).
fn trans1(c: char) -> char {
    match c {
        'ก' | 'ข' | 'ฃ' | 'ค' | 'ฅ' | 'ฆ' => 'ก',
        'ง' => 'ง',
        'จ' => 'จ',
        'ฉ' | 'ช' | 'ฌ' => 'ช',
        'ซ' | 'ศ' | 'ษ' | 'ส' => 'ซ',
        'ญ' | 'ย' => 'ย',
        'ฎ' | 'ด' => 'ด',
        'ฏ' | 'ต' => 'ต',
        'ณ' | 'น' => 'น',
        'ฐ' | 'ฑ' | 'ฒ' | 'ถ' | 'ท' | 'ธ' => 'ท',
        'บ' => 'บ',
        'ป' => 'ป',
        'ผ' | 'พ' | 'ภ' => 'พ',
        'ฝ' | 'ฟ' => 'ฟ',
        'ม' => 'ม',
        'ร' | 'ล' | 'ฬ' | 'ฤ' | 'ฦ' => 'ร',
        'ว' => 'ว',
        'ห' | 'ฮ' => 'ห',
        'อ' => 'อ',
        _ => c,
    }
}

/// TRANS2: remainder character encoding.
/// Maps Thai characters to single-character codes (digits, letters A-F).
fn trans2(c: char) -> char {
    match c {
        // Group 1: K-class
        'ก' | 'ข' | 'ฃ' | 'ค' | 'ฅ' | 'ฆ' => '1',
        // Group 2: NG
        'ง' => '2',
        // Group 3: dental/sibilant
        'จ' | 'ฉ' | 'ช' | 'ซ' | 'ฌ' | 'ฎ' | 'ฏ' | 'ฐ' | 'ฑ' | 'ฒ' | 'ด' | 'ต' | 'ถ'
        | 'ท' | 'ธ' | 'ศ' | 'ษ' | 'ส' => '3',
        // Group 4: nasal/liquid
        'ญ' | 'ณ' | 'น' | 'ร' | 'ล' | 'ฬ' | 'ฤ' | 'ฦ' => '4',
        // Group 5: labial stop
        'บ' | 'ป' | 'พ' | 'ฟ' | 'ภ' | 'ผ' | 'ฝ' => '5',
        // Group 6: M / Sara Am
        'ม' | 'ำ' => '6',
        // Group 7: semivowel
        'ย' | 'ว' | 'ไ' | 'ใ' => '7',
        // Group 8: H-class
        'ห' | 'ฮ' => '8',
        // Group 9: Sara Aa
        'า' => '9',
        // Group A: Lakkhangyao, Sara Ue, Sara Uee
        'ๅ' | 'ึ' | 'ื' => 'A',
        // Group B: Sara E
        'เ' => 'B',
        // Group C: Sara Ae
        'แ' => 'C',
        // Group D: Sara O
        'โ' => 'D',
        // Group E: Sara U, Sara Uu
        'ุ' | 'ู' => 'E',
        // Group F: Or Ang
        'อ' => 'F',
        _ => c,
    }
}

/// Compute the LK82 Thai soundex for the given text.
///
/// Returns a 5-character phonetic code string, or an empty string
/// if the input is empty or reduces to nothing after preprocessing.
///
/// # Examples
/// ```
/// use nlpo3::soundex::lk82;
///
/// assert_eq!(lk82("ลัก"), "ร1000");
/// assert_eq!(lk82("รัก"), "ร1000");
/// assert_eq!(lk82("รักษ์"), "ร1000");
/// assert_eq!(lk82("บูรณการ"), "บE419");
/// assert_eq!(lk82("ปัจจุบัน"), "ป3E54");
/// ```
pub fn lk82(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    // Remove tone marks
    let text = remove_tonemarks(text);
    // Remove karan patterns
    let text = RE_KARANT.replace_all(&text, "").to_string();
    // Remove signs
    let text = RE_SIGN.replace_all(&text, "").to_string();

    if text.is_empty() {
        return String::new();
    }

    let chars: Vec<char> = text.chars().collect();
    let mut res: Vec<String> = Vec::new();
    let start_idx;

    // Encode first character(s)
    if is_thai_consonant(chars[0]) {
        res.push(trans1(chars[0]).to_string());
        start_idx = 1;
    } else {
        // Leading vowel: encode second char with TRANS1, first with TRANS2
        if chars.len() > 1 {
            res.push(trans1(chars[1]).to_string());
        }
        res.push(trans2(chars[0]).to_string());
        start_idx = 2;
    }

    // Encode the rest
    let remaining: Vec<char> = if start_idx < chars.len() {
        chars[start_idx..].to_vec()
    } else {
        Vec::new()
    };
    let len_text = remaining.len();
    let mut i_v: Option<usize> = None; // position of last vowel separator

    for (i, &c) in remaining.iter().enumerate() {
        if "ะัิี".contains(c) {
            // ะ ั ิ ี (U+0E30,31,34,35): separator only
            i_v = Some(i);
            res.push(String::new());
        } else if "าึืูๅ".contains(c) {
            // า ึ ื ู ๅ (U+0E32,36,37,39,45): separator + encode
            i_v = Some(i);
            res.push(trans2(c).to_string());
        } else if c == 'ุ' {
            // ุ (U+0E38): separator; encode unless preceded by ต or ธ
            i_v = Some(i);
            if i == 0 || (remaining[i - 1] != 'ต' && remaining[i - 1] != 'ธ') {
                res.push(trans2(c).to_string());
            } else {
                res.push(String::new());
            }
        } else if c == 'ห' || c == 'อ' {
            // ห อ: encode only if next char is ึ ื ุ ู
            if i + 1 < len_text && "ึืุู".contains(remaining[i + 1])
            {
                res.push(trans2(c).to_string());
            }
            // else: skip (don't push anything)
        } else if "ยรฤฦว".contains(c) {
            // ย ร ฤ ฦ ว: encode only if after vowel or next is ึ ื ุ ู
            if i_v == Some(i.wrapping_sub(1))
                || (i + 1 < len_text
                    && "ึืุู".contains(remaining[i + 1]))
            {
                res.push(trans2(c).to_string());
            }
        } else {
            res.push(trans2(c).to_string());
        }
    }

    // Remove consecutive duplicate codes
    let mut res2: Vec<&str> = Vec::new();
    if !res.is_empty() {
        res2.push(&res[0]);
        for i in 1..res.len() {
            if res[i] != res[i - 1] {
                res2.push(&res[i]);
            }
        }
    }

    // Join and pad with zeros, truncate to 5 chars
    let joined: String = res2.concat();
    let padded = format!("{}0000", joined);
    padded.chars().take(5).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lk82_empty() {
        assert_eq!(lk82(""), "");
    }

    #[test]
    fn test_lk82_karan_removes_all() {
        assert_eq!(lk82("น์"), "");
    }

    #[test]
    fn test_lk82_known_values() {
        assert_eq!(lk82("ลัก"), "ร1000");
        assert_eq!(lk82("รัก"), "ร1000");
        assert_eq!(lk82("รักษ์"), "ร1000");
        assert_eq!(lk82("บูรณการ"), "บE419");
        assert_eq!(lk82("ปัจจุบัน"), "ป3E54");
        assert_eq!(lk82("รถ"), "ร3000");
    }

    #[test]
    fn test_lk82_phonetic_equivalence() {
        assert_eq!(lk82("เหตุ"), lk82("เหด"));
    }

    #[test]
    fn test_lk82_exact_values() {
        // Verified against pythainlp output
        assert_eq!(lk82("เกาะ"), "กB900");
        assert_eq!(lk82("อุยกูร์"), "อE71E");
        assert_eq!(lk82("หยากไย่"), "ห9170");
        assert_eq!(lk82("หอ"), "ห0000");
        assert_eq!(lk82("อยู่"), "อ7E00");
        assert_eq!(lk82("อู่"), "อE000");
        assert_eq!(lk82("อย่าง"), "อ9200");
        assert_eq!(lk82("เหย้า"), "หB900");
        assert_eq!(lk82("หยุด"), "ห7E30");
        assert_eq!(lk82("หืออือ"), "หAFA0");
    }
}
