// SPDX-FileCopyrightText: 2024 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

//! Thai-English Cross-Language Transliterated Word Retrieval using Soundex.
//!
//! References:
//! Prayut Suwanvisat, Somchai Prasitjutrakul.
//! Thai-English Cross-Language Transliterated Word Retrieval using Soundex
//! Technique. In 1998.

use crate::thai_chars::is_thai_character;

// Character groups for phonetic encoding.
// Supports both Thai and English characters.

const C0: &str = "AEIOUHWYอ";
const C1: &str = "BFPVบฝฟปผพภว";
const C2: &str = "CGJKQSXZขฃคฅฆฉฌกจซศษส";
const C3: &str = "DTฎดฏตฐฑฒถทธ";
const C4: &str = "Lลฬ";
const C5: &str = "MNมณน";
const C6: &str = "Rร";
const C7: &str = "AEIOUอ";
const C8: &str = "Hหฮ";
const C1_1: &str = "Wว";
const C9: &str = "Yยญ";
const C52: &str = "ง";

/// Encode a character based on position and character group.
/// Returns None for characters that don't map to any group.
fn encode_char(c: char, is_first: bool) -> Option<&'static str> {
    if is_first && C0.contains(c) {
        return Some("0");
    }
    if C1.contains(c) {
        return Some("1");
    }
    if C2.contains(c) {
        return Some("2");
    }
    if C3.contains(c) {
        return Some("3");
    }
    if C4.contains(c) {
        return Some("4");
    }
    if C5.contains(c) {
        return Some("5");
    }
    if C6.contains(c) {
        return Some("6");
    }
    if C52.contains(c) {
        return Some("52");
    }
    if !is_first && C7.contains(c) {
        return Some("7");
    }
    if !is_first && C8.contains(c) {
        return Some("8");
    }
    if !is_first && C1_1.contains(c) {
        return Some("1");
    }
    if !is_first && C9.contains(c) {
        return Some("9");
    }
    None
}

/// Check if a character is an ASCII letter (A-Z).
fn is_ascii_upper(c: char) -> bool {
    c.is_ascii_uppercase()
}

/// Compute the Prayut & Somchaip soundex for Thai-English cross-language text.
///
/// This algorithm supports mixed Thai and English text, encoding characters
/// into phonetic groups. Unlike other soundex systems, it returns the
/// **rightmost** N characters (right-truncation).
///
/// # Arguments
/// * `text` - Thai or English text
/// * `length` - Desired length of the soundex code (default: 4)
///
/// # Examples
/// ```
/// use nlpo3::soundex::prayut_and_somchaip;
///
/// assert_eq!(prayut_and_somchaip("king", 2), "52");
/// assert_eq!(prayut_and_somchaip("คิง", 2), "52");
/// assert_eq!(prayut_and_somchaip("king", 4), "2752");
/// ```
pub fn prayut_and_somchaip(text: &str, length: usize) -> String {
    if text.is_empty() {
        return String::new();
    }

    let text_upper: String = text.chars().map(|c| {
        if c.is_ascii_lowercase() {
            c.to_ascii_uppercase()
        } else {
            c
        }
    }).collect();

    // Keep only Thai characters and ASCII uppercase letters
    let chars: Vec<char> = text_upper
        .chars()
        .filter(|&c| is_thai_character(c) || is_ascii_upper(c))
        .collect();

    if chars.is_empty() {
        return String::new();
    }

    // Encode each character
    let mut encoded: Vec<&str> = Vec::with_capacity(chars.len());
    for (i, &c) in chars.iter().enumerate() {
        if let Some(code) = encode_char(c, i == 0) {
            encoded.push(code);
        }
    }

    // Join and take the rightmost `length` characters
    let joined: String = encoded.concat();
    let char_vec: Vec<char> = joined.chars().collect();
    if char_vec.len() <= length {
        joined
    } else {
        char_vec[char_vec.len() - length..].iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        assert_eq!(prayut_and_somchaip("", 4), "");
    }

    #[test]
    fn test_english_words() {
        assert_eq!(prayut_and_somchaip("king", 2), "52");
        // K→2, I→7, N→5, G→2 = "2752", right-4 = "2752"
        assert_eq!(prayut_and_somchaip("king", 4), "2752");
    }

    #[test]
    fn test_thai_words() {
        assert_eq!(prayut_and_somchaip("คิง", 2), "52");
    }

    #[test]
    fn test_cross_language_equivalence() {
        // Same phonetic encoding for Thai and English versions
        assert_eq!(
            prayut_and_somchaip("king", 2),
            prayut_and_somchaip("คิง", 2)
        );
    }

    #[test]
    fn test_right_truncation() {
        // Right-truncation: takes last N characters
        let full = prayut_and_somchaip("king", 10);
        let short = prayut_and_somchaip("king", 2);
        assert!(full.ends_with(&short));
    }

    #[test]
    fn test_vp_equivalence() {
        assert_eq!(prayut_and_somchaip("vp", 4), "11");
        assert_eq!(
            prayut_and_somchaip("vp", 4),
            prayut_and_somchaip("วีพี", 4)
        );
    }

    #[test]
    fn test_exact_values() {
        // Verified against pythainlp output
        assert_eq!(prayut_and_somchaip("บา", 4), "1");
        assert_eq!(prayut_and_somchaip("อด", 4), "03");
        assert_eq!(prayut_and_somchaip("ลน", 4), "45");
        assert_eq!(prayut_and_somchaip("มอ", 4), "57");
        assert_eq!(prayut_and_somchaip("ณาญ", 4), "59");
        assert_eq!(prayut_and_somchaip("กาง", 4), "252");
    }
}
