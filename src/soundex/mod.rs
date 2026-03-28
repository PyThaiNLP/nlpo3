// SPDX-FileCopyrightText: 2024 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

//! Thai soundex algorithms for phonetic matching.

pub mod lk82;
pub mod metasound;
pub mod prayut_and_somchaip;
pub mod udom83;

pub use lk82::lk82;
pub use metasound::metasound;
pub use prayut_and_somchaip::prayut_and_somchaip;
pub use udom83::udom83;

/// Calculate similarity between two soundex codes.
///
/// Character-by-character comparison: matches / max(len(a), len(b)).
///
/// Based on Tapsai et al. (2020), Section 3.3.
///
/// # Examples
/// ```
/// use nlpo3::soundex::soundex_similarity;
///
/// assert_eq!(soundex_similarity("กก1Bน2-", "กก1Bน2-"), 1.0);
/// assert_eq!(soundex_similarity("", ""), 1.0);
/// assert_eq!(soundex_similarity("", "abc"), 0.0);
/// ```
pub fn soundex_similarity(code1: &str, code2: &str) -> f64 {
    if code1.is_empty() && code2.is_empty() {
        return 1.0;
    }
    if code1.is_empty() || code2.is_empty() {
        return 0.0;
    }

    let chars1: Vec<char> = code1.chars().collect();
    let chars2: Vec<char> = code2.chars().collect();
    let max_len = chars1.len().max(chars2.len());
    let min_len = chars1.len().min(chars2.len());

    let match_count = (0..min_len).filter(|&i| chars1[i] == chars2[i]).count();

    match_count as f64 / max_len as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_perfect() {
        assert_eq!(soundex_similarity("กก1Bน2-", "กก1Bน2-"), 1.0);
    }

    #[test]
    fn test_similarity_empty() {
        assert_eq!(soundex_similarity("", ""), 1.0);
        assert_eq!(soundex_similarity("", "abc"), 0.0);
        assert_eq!(soundex_similarity("abc", ""), 0.0);
    }

    #[test]
    fn test_similarity_tone_diff() {
        // 5 out of 6 match: คข7M_- where _ differs
        let sim = soundex_similarity("คข7M2-", "คข7M0-");
        assert!((sim - 5.0 / 6.0).abs() < 0.001);
    }

    #[test]
    fn test_similarity_different_length() {
        // 5 out of 11 match
        let sim = soundex_similarity("กก1A-", "กก1A-0-มม7M");
        assert!((sim - 5.0 / 11.0).abs() < 0.001);
    }
}
