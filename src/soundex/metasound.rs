// SPDX-FileCopyrightText: 2024 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

//! Thai soundex - MetaSound system.
//!
//! References:
//! Snae & Bruckner. (2009). Novel Phonetic Name Matching Algorithm with
//! a Statistical Ontology for Analysing Names Given in Accordance
//! with Thai Astrology.

use crate::thai_chars::{is_thai_consonant, THANTHAKHAT};

/// Encode a Thai consonant to its MetaSound digit.
fn metasound_code(c: char) -> char {
    match c {
        // Group 1: K-class
        'ก' | 'ข' | 'ฃ' | 'ค' | 'ฆ' | 'ฅ' => '1',
        // Group 2: D-class (dental/sibilant) — 18 consonants per paper Table p.507
        'จ' | 'ฉ' | 'ช' | 'ฌ' | 'ซ' | 'ฎ' | 'ฏ' | 'ฐ' | 'ฑ' | 'ฒ'
        | 'ด' | 'ต' | 'ถ' | 'ท' | 'ธ' | 'ศ' | 'ษ' | 'ส' => '2',
        // Group 3: B-class (labial)
        'ฟ' | 'ฝ' | 'พ' | 'ผ' | 'ภ' | 'บ' | 'ป' => '3',
        // Group 4: NG
        'ง' => '4',
        // Group 5: N-class (nasal/liquid)
        'ล' | 'ฬ' | 'ร' | 'น' | 'ณ' | 'ฦ' | 'ญ' => '5',
        // Group 6: M
        'ม' => '6',
        // Group 7: Y
        'ย' => '7',
        // Group 8: W
        'ว' => '8',
        // Default
        _ => '0',
    }
}

/// Compute the MetaSound Thai soundex for the given text.
///
/// Returns a phonetic code of the specified `length` (default 4),
/// or an empty string if the input is empty.
///
/// # Examples
/// ```
/// use nlpo3::soundex::metasound;
///
/// assert_eq!(metasound("ลัก", 4), "ล100");
/// assert_eq!(metasound("รัก", 4), "ร100");
/// assert_eq!(metasound("รักษ์", 4), "ร100");
/// assert_eq!(metasound("บูรณการ", 5), "บ5515");
/// assert_eq!(metasound("บูรณการ", 4), "บ551");
/// ```
pub fn metasound(text: &str, length: usize) -> String {
    if text.is_empty() {
        return String::new();
    }

    // Keep only consonants, ฦ, and thanthakhat
    let mut chars: Vec<char> = text
        .chars()
        .filter(|&c| is_thai_consonant(c) || c == 'ฦ' || c == THANTHAKHAT)
        .collect();

    // Remove karan (thanthakhat and the consonant before it)
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == THANTHAKHAT {
            if i > 0 {
                chars[i - 1] = ' ';
            }
            chars[i] = ' ';
        }
        i += 1;
    }

    // Filter out space placeholders before truncation
    chars.retain(|&c| c != ' ');

    // Truncate to desired length
    chars.truncate(length);

    // Retain first consonant, encode the rest
    for item in chars.iter_mut().skip(1) {
        *item = metasound_code(*item);
    }

    // Pad with '0' to reach desired length
    while chars.len() < length {
        chars.push('0');
    }

    chars.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metasound_empty() {
        assert_eq!(metasound("", 4), "");
    }

    #[test]
    fn test_metasound_known_values() {
        assert_eq!(metasound("ลัก", 4), "ล100");
        assert_eq!(metasound("รัก", 4), "ร100");
        assert_eq!(metasound("รักษ์", 4), "ร100");
        assert_eq!(metasound("บูรณการ", 5), "บ5515");
        assert_eq!(metasound("บูรณการ", 6), "บ55150");
        assert_eq!(metasound("บูรณการ", 4), "บ551");
    }

    #[test]
    fn test_metasound_phonetic_equivalence() {
        assert_eq!(metasound("เหตุ", 4), metasound("เหด", 4));
        assert_eq!(metasound("รักษ์", 4), metasound("รัก", 4));
    }

    #[test]
    fn test_metasound_specific_values() {
        assert_eq!(metasound("บูรณะ", 4), "บ550");
        assert_eq!(metasound("คน", 4), "ค500");
        assert_eq!(metasound("คนA", 4), "ค500");
        assert_eq!(metasound("ดา", 4), "ด000");
        assert_eq!(metasound("ปัจจุบัน", 4), "ป223");
    }

    #[test]
    fn test_metasound_fixed_consonants() {
        // ฏ ฑ ถ ธ were missing from _C2 in pythainlp — fixed per paper Table p.507
        // See: https://github.com/PyThaiNLP/pythainlp/issues/1383
        assert_eq!(metasound("กถ", 2), "ก2");  // ถ → class 2
        assert_eq!(metasound("กธ", 2), "ก2");  // ธ → class 2
        assert_eq!(metasound("กฏ", 2), "ก2");  // ฏ → class 2
        assert_eq!(metasound("กฑ", 2), "ก2");  // ฑ → class 2
    }

    #[test]
    fn test_metasound_uncategorized() {
        // ห อ ฮ correctly map to '0' per paper (no /h/ or glottal class)
        assert_eq!(metasound("กห", 2), "ก0");
        assert_eq!(metasound("กอ", 2), "ก0");
        assert_eq!(metasound("กฮ", 2), "ก0");
    }

    #[test]
    fn test_metasound_non_empty() {
        assert_eq!(metasound("จะ", 4), "จ000");
        assert_eq!(metasound("ปา", 4), "ป000");
        assert_eq!(metasound("งง", 4), "ง400");
        assert_eq!(metasound("ลา", 4), "ล000");
        assert_eq!(metasound("มา", 4), "ม000");
        assert_eq!(metasound("ยา", 4), "ย000");
        assert_eq!(metasound("วา", 4), "ว000");
        assert_eq!(metasound("บูชา", 4), "บ200");
        assert_eq!(metasound("กมลา", 4), "ก650");
        assert_eq!(metasound("สุวรรณา", 4), "ส855");
        assert_eq!(metasound("ดอยบอย", 4), "ด073");
    }
}
