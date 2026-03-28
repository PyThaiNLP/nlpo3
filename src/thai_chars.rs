// SPDX-FileCopyrightText: 2024 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

//! Thai character constants and classification functions.
//!
//! Character definitions follow the Unicode Standard, Chapter 16.4 (Thai)
//! and TIS 620-2533 (Thai Industrial Standard for character encoding).
//!
//! Constants are aligned with PyThaiNLP's `pythainlp/__init__.py`
//! to ensure consistent character sets across Python and Rust implementations.
//!
//! Unicode block: Thai (U+0E00..U+0E7F)
//! Reference: <https://www.unicode.org/charts/PDF/U0E00.pdf>

/// All 44 Thai consonants (U+0E01 KO KAI .. U+0E2E HO NOKHUK).
///
/// Same as `pythainlp.thai_consonants`.
pub const THAI_CONSONANTS: &str = "กขฃคฅฆงจฉชซฌญฎฏฐฑฒณดตถทธนบปผฝพฟภมยรลวศษสหฬอฮ";

/// Thanthakhat / karan (U+0E4C) — marks a preceding consonant as silent.
pub const THANTHAKHAT: char = '\u{0E4C}';

/// Check if a character is a Thai consonant (U+0E01..U+0E2E).
pub fn is_thai_consonant(c: char) -> bool {
    ('\u{0E01}'..='\u{0E2E}').contains(&c)
}

/// Check if a character is in the Thai Unicode block (U+0E01..U+0E5B).
///
/// Simple range check covering the entire assigned Thai block.
/// Includes 4 currently unassigned code points (U+0E3B..U+0E3E)
/// within the range — these are not PUA and may be assigned in
/// future Unicode versions.
pub fn is_thai_character(c: char) -> bool {
    ('\u{0E01}'..='\u{0E5B}').contains(&c)
}

/// Check if a character is a Thai tone mark (U+0E48..U+0E4B).
///
/// Mai Ek, Mai Tho, Mai Tri, Mai Chattawa.
pub fn is_thai_tonemark(c: char) -> bool {
    ('\u{0E48}'..='\u{0E4B}').contains(&c)
}

/// Remove all Thai tone marks from text.
pub fn remove_tonemarks(text: &str) -> String {
    text.chars().filter(|c| !is_thai_tonemark(*c)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thai_consonants_count() {
        assert_eq!(THAI_CONSONANTS.chars().count(), 44);
    }

    #[test]
    fn test_is_thai_consonant() {
        assert!(is_thai_consonant('ก')); // first
        assert!(is_thai_consonant('ฮ')); // last
        assert!(!is_thai_consonant('า')); // vowel
        assert!(!is_thai_consonant('A'));
    }

    #[test]
    fn test_is_thai_character() {
        assert!(is_thai_character('ก'));  // consonant
        assert!(is_thai_character('า'));  // vowel
        assert!(is_thai_character('๙'));  // digit
        assert!(is_thai_character('฿'));  // baht symbol
        assert!(!is_thai_character('A')); // latin
        assert!(!is_thai_character('\u{0E00}')); // before block
        assert!(!is_thai_character('\u{0E5C}')); // after block
    }

    #[test]
    fn test_is_thai_tonemark() {
        assert!(is_thai_tonemark('\u{0E48}')); // mai ek
        assert!(is_thai_tonemark('\u{0E4B}')); // mai chattawa
        assert!(!is_thai_tonemark('ก'));
    }

    #[test]
    fn test_remove_tonemarks() {
        assert_eq!(remove_tonemarks("ร้อย"), "รอย");
        assert_eq!(remove_tonemarks("สี่"), "สี");
        assert_eq!(remove_tonemarks("สอง"), "สอง"); // no tonemarks
    }
}
