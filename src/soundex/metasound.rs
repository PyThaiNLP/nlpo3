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
        // Group 2: D-class (dental/sibilant)
        'จ' | 'ฉ' | 'ช' | 'ฌ' | 'ซ' | 'ฐ' | 'ท' | 'ฒ' | 'ด' | 'ฎ' | 'ต' | 'ส' | 'ศ'
        | 'ษ' => '2',
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

    // Keep only consonants and thanthakhat
    let mut chars: Vec<char> = text
        .chars()
        .filter(|&c| is_thai_consonant(c) || c == THANTHAKHAT)
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
    fn test_metasound_non_empty() {
        assert!(!metasound("จะ", 4).is_empty());
        assert!(!metasound("ปา", 4).is_empty());
        assert!(!metasound("งง", 4).is_empty());
        assert!(!metasound("ลา", 4).is_empty());
        assert!(!metasound("มา", 4).is_empty());
        assert!(!metasound("ยา", 4).is_empty());
        assert!(!metasound("วา", 4).is_empty());
        assert!(!metasound("บูชา", 4).is_empty());
        assert!(!metasound("กมลา", 4).is_empty());
        assert!(!metasound("กาโวกาโว", 4).is_empty());
        assert!(!metasound("สุวรรณา", 4).is_empty());
        assert!(!metasound("ดอยบอย", 4).is_empty());
    }
}
