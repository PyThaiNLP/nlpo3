// SPDX-FileCopyrightText: 2024 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

//! Thai character classification utilities.
//!
//! Provides constants and helper functions for identifying Thai character types
//! (consonants, vowels, tone marks, signs, digits).

/// All 44 Thai consonants (ก-ฮ).
pub const THAI_CONSONANTS: &str = "กขฃคฅฆงจฉชซฌญฎฏฐฑฒณดตถทธนบปผฝพฟภมยรลวศษสหฬอฮ";

/// Thai tone marks: ่ ้ ๊ ๋ (U+0E48..U+0E4B).
pub const THAI_TONEMARKS: &str = "่้๊๋";

/// Thai above vowels: ั ิ ี ึ ื (U+0E31, U+0E34..U+0E37).
pub const THAI_ABOVE_VOWELS: &str = "ัิีึื";

/// Thai below vowels: ุ ู (U+0E38..U+0E39).
pub const THAI_BELOW_VOWELS: &str = "ุู";

/// Thai lead vowels (placed before consonant): เ แ โ ใ ไ (U+0E40..U+0E44).
pub const THAI_LEAD_VOWELS: &str = "เแโใไ";

/// Thai follow vowels (placed after consonant): ะ า ำ ๅ (U+0E30, U+0E32, U+0E33, U+0E45).
pub const THAI_FOLLOW_VOWELS: &str = "ะาำๅ";

/// All Thai vowels (above + below + lead + follow + ฤ ฦ ํ ็).
/// U+0E24, U+0E26, U+0E30..U+0E39, U+0E40..U+0E45, U+0E47, U+0E4D.
pub const THAI_VOWELS: &str = "ฤฦะัาำิีึืุูเแโใไๅํ็";

/// Thai signs: ฯ ์ ๆ ์ ํ ๎ (U+0E2F, U+0E3A, U+0E46, U+0E4C, U+0E4D, U+0E4E).
pub const THAI_SIGNS: &str = "ฯ์ๆ์ํ๎";

/// Thai digits: ๐๑๒๓๔๕๖๗๘๙ (U+0E50..U+0E59).
pub const THAI_DIGITS: &str = "๐๑๒๓๔๕๖๗๘๙";

/// Thai punctuations: ๏ ๚ ๛ (U+0E4F, U+0E5A, U+0E5B).
pub const THAI_PUNCTUATIONS: &str = "๏๚๛";

/// Thai Baht symbol: ฿ (U+0E3F).
pub const THAI_SYMBOLS: &str = "฿";

/// Thanthakhat / karan: ์ (U+0E4C).
pub const THANTHAKHAT: char = '์';

/// Check if a character is in the Thai Unicode block (U+0E01..U+0E5B).
pub fn is_thai_character(c: char) -> bool {
    ('\u{0E01}'..='\u{0E5B}').contains(&c)
}

/// Check if a character is a Thai consonant (ก-ฮ, U+0E01..U+0E2E).
pub fn is_thai_consonant(c: char) -> bool {
    ('\u{0E01}'..='\u{0E2E}').contains(&c)
}

/// Check if a character is a Thai tone mark (่้๊๋, U+0E48..U+0E4B).
pub fn is_thai_tonemark(c: char) -> bool {
    ('\u{0E48}'..='\u{0E4B}').contains(&c)
}

/// Check if a character is a Thai vowel.
/// Covers above, below, lead, follow vowels plus ฤ ฦ ํ ็.
pub fn is_thai_vowel(c: char) -> bool {
    matches!(c,
        'ฤ' | 'ฦ' |                        // U+0E24, U+0E26
        '\u{0E30}'..='\u{0E39}' |           // ะ ั า ำ ิ ี ึ ื ุ ู
        '\u{0E40}'..='\u{0E45}' |           // เ แ โ ใ ไ ๅ
        '็' | 'ํ'                            // U+0E47, U+0E4D
    )
}

/// Check if a character is a Thai digit (๐-๙, U+0E50..U+0E59).
pub fn is_thai_digit(c: char) -> bool {
    ('\u{0E50}'..='\u{0E59}').contains(&c)
}

/// Remove all Thai tone marks from text.
pub fn remove_tonemarks(text: &str) -> String {
    text.chars().filter(|c| !is_thai_tonemark(*c)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_thai_character() {
        assert!(is_thai_character('ก'));
        assert!(is_thai_character('า'));
        assert!(is_thai_character('๙'));
        assert!(!is_thai_character('A'));
        assert!(!is_thai_character('1'));
    }

    #[test]
    fn test_is_thai_consonant() {
        assert!(is_thai_consonant('ก'));
        assert!(is_thai_consonant('ฮ'));
        assert!(!is_thai_consonant('า'));
        assert!(!is_thai_consonant('A'));
    }

    #[test]
    fn test_is_thai_tonemark() {
        assert!(is_thai_tonemark('่'));  // U+0E48
        assert!(is_thai_tonemark('๋'));  // U+0E4B
        assert!(!is_thai_tonemark('ก'));
    }

    #[test]
    fn test_is_thai_digit() {
        assert!(is_thai_digit('๐'));
        assert!(is_thai_digit('๙'));
        assert!(!is_thai_digit('0'));
        assert!(!is_thai_digit('ก'));
    }

    #[test]
    fn test_remove_tonemarks() {
        assert_eq!(remove_tonemarks("สอง"), "สอง");
        assert_eq!(remove_tonemarks("ร้อย"), "รอย");
        assert_eq!(remove_tonemarks("สี่"), "สี");
    }
}
