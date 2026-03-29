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

/// Encode a Thai word into Complete Soundex code.
///
/// Handles multi-syllable words via heuristic splitting.
/// Adds asterisk suffix for words containing ญญ, ญ+ย, ณ+ย, or starting with ญ.
///
/// # Examples
/// ```
/// use nlpo3::soundex::complete_soundex::complete_soundex;
///
/// assert_eq!(complete_soundex("ก้าน"), "กก1Bน2-");
/// assert_eq!(complete_soundex("กมล"), "กก1A-0-มม7Mน0-");
/// ```
pub fn complete_soundex(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let text = clean_text(text);
    if text.is_empty() {
        return String::new();
    }

    let parts = heuristic_split(&text);
    let mut result = String::new();
    for (syl, rule) in &parts {
        result.push_str(&process_syllable(syl, *rule));
    }

    // Asterisk suffix for specific patterns
    if text.contains("ญญ")
        || (text.contains('ญ') && text.contains('ย'))
        || (text.contains('ณ') && text.contains('ย'))
        || text.starts_with('ญ')
    {
        result.push('*');
    }

    result
}

/// Process a single syllable into its Complete Soundex code.
fn process_syllable(syl: &str, implicit_rule: Option<&str>) -> String {
    let chars: Vec<char> = syl.chars().collect();
    let mut idx = 0;

    // A. Leading Vowel
    let leading_vowel = if idx < chars.len() && "เแโไใ".contains(chars[idx]) {
        let v = chars[idx];
        idx += 1;
        Some(v)
    } else {
        None
    };

    // B. Initial Consonant + Cluster
    let mut init_char = None;
    let mut init_code = String::new();
    let mut cluster_char = "-".to_string();

    if idx < chars.len() && is_thai_consonant(chars[idx]) {
        init_char = Some(chars[idx]);

        // Special: ทร → ซซ
        if chars[idx] == 'ท' && idx + 1 < chars.len() && chars[idx + 1] == 'ร' {
            init_code = "ซซ".to_string();
            idx += 2;
        } else {
            init_code = initial_code(chars[idx]).to_string();
            idx += 1;

            // Check for cluster consonant (ร ล ว)
            if idx < chars.len() && "รลว".contains(chars[idx]) {
                let is_cluster = if idx + 1 < chars.len() {
                    let nc = chars[idx + 1];
                    "ะัิีึืุู่้๊๋".contains(nc)
                        || (leading_vowel.is_some()
                            && !"รลว".contains(nc)
                            && !is_thai_consonant(nc)
                            && nc != 'า')
                } else {
                    leading_vowel.is_some()
                };
                if is_cluster {
                    cluster_char = chars[idx].to_string();
                    idx += 1;
                }
            }
        }
    }

    // C. Map leading vowel to code
    let (mut vowel_code_str, mut final_code_str) = match leading_vowel {
        Some('โ') => ("7N".to_string(), "-".to_string()),
        Some('ไ') | Some('ใ') => ("1A".to_string(), "ย".to_string()),
        Some('แ') => ("6L".to_string(), "-".to_string()),
        Some('เ') => ("5J".to_string(), "-".to_string()),
        _ => (String::new(), "-".to_string()),
    };

    // D. Scan remaining chars for vowels, tones, finals
    let mut tone_code_str = "0".to_string();
    let mut final_candidates: Vec<char> = Vec::new();

    for &c in &chars[idx..] {
        if "่้๊๋".contains(c) {
            tone_code_str = tone_code(c).to_string();
        } else if "ะัาิีึืุู".contains(c)
            || c == 'ำ'
            || (c == 'อ' && leading_vowel == Some('เ'))
        {
            // Process vowel character
            if leading_vowel == Some('เ') && c == 'ื' {
                vowel_code_str = "BV".to_string(); // เอือ
            } else if leading_vowel == Some('เ') && c == 'อ' {
                if vowel_code_str != "BV" {
                    vowel_code_str = "9R".to_string(); // เอ
                }
            } else if c == 'ำ' {
                vowel_code_str = "1A".to_string();
                final_code_str = "ม".to_string();
            } else if c == 'อ' && leading_vowel.is_none() && vowel_code_str.is_empty() {
                vowel_code_str = "8P".to_string();
            } else if let Some(v) = match c {
                'ะ' => Some("1A"), 'ั' => Some("1A"), 'า' => Some("1B"),
                'ิ' => Some("2C"), 'ี' => Some("2D"),
                'ึ' => Some("3E"), 'ื' => Some("3F"),
                'ุ' => Some("4G"), 'ู' => Some("4H"),
                _ => None,
            } {
                vowel_code_str = v.to_string();
            }

            // Handle ะ shortening
            if c == 'ะ' {
                vowel_code_str = match vowel_code_str.as_str() {
                    "5J" => "5I", "6L" => "6K", "7N" => "7M", "1B" => "1A",
                    other => other,
                }.to_string();
            }
        } else {
            final_candidates.push(c);
        }
    }

    // E. Final consonant processing
    let mut dropped_r = false;
    if final_code_str == "-" {
        if syl.contains("รร") {
            vowel_code_str = "1A".to_string();
            final_code_str = if let Some(&f) = final_candidates.last() {
                final_code(f).to_string()
            } else {
                "น".to_string()
            };
        } else if !final_candidates.is_empty() {
            let raw: String = final_candidates.iter().collect();
            let f = if raw.len() >= 2 && raw.chars().rev().nth(1) == Some('ร')
                && final_code(raw.chars().last().unwrap()) != "-"
            {
                dropped_r = true;
                raw.chars().last().unwrap()
            } else if raw.ends_with("ตร") {
                dropped_r = true;
                'ต'
            } else {
                *final_candidates.last().unwrap()
            };
            final_code_str = final_code(f).to_string();
        }
    }

    // F. Special format check (tone before final)
    let special_format = {
        let ic = init_char.unwrap_or(' ');
        ic == 'ญ' || ic == 'ย' || ic == 'น'
            || final_candidates.iter().any(|&c| c == 'ญ' || c == 'ณ')
            || (final_candidates.iter().any(|&c| c == 'น') && vowel_code_str == "1A")
    };

    // G. Implicit vowel
    if vowel_code_str.is_empty() {
        vowel_code_str = match implicit_rule {
            Some("a") => "1A",
            Some("o") => "7M",
            _ => "7M",
        }.to_string();
    }

    // H. Fix leading vowel overrides
    if leading_vowel == Some('โ') { vowel_code_str = "7N".to_string(); }
    if leading_vowel == Some('แ') { vowel_code_str = "6L".to_string(); }

    // I. So Sua adjustment
    if init_char == Some('ส') && init_code == "ซศ" {
        if implicit_rule.is_none() && syl.chars().count() >= 2 {
            let consonants_after: Vec<char> = syl.chars().skip(1)
                .filter(|&c| is_thai_consonant(c))
                .collect();
            if consonants_after.is_empty() || consonants_after.iter().all(|c| "รลว".contains(*c)) {
                init_code = "ซซ".to_string();
            }
        }
    }

    // J. Format output
    if special_format {
        format!("{}{}{}{}{}", init_code, vowel_code_str, tone_code_str, final_code_str, cluster_char)
    } else if dropped_r && (final_code_str == "ก" || final_code_str == "-") {
        format!("{}{}-{}{}{}",init_code, vowel_code_str, final_code_str, tone_code_str, cluster_char)
    } else {
        format!("{}{}{}{}{}", init_code, vowel_code_str, final_code_str, tone_code_str, cluster_char)
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

    #[test]
    fn test_process_syllable_basic() {
        assert_eq!(process_syllable("ก้าน", None), "กก1Bน2-");
        assert_eq!(process_syllable("ก้ม", None), "กก7Mม2-");
        assert_eq!(process_syllable("แกน", None), "กก6Lน0-");
        assert_eq!(process_syllable("โก่ง", None), "กก7Nง1-");
        assert_eq!(process_syllable("นา", None), "นน1B0--");
        assert_eq!(process_syllable("ยา", None), "ยย1B0--");
    }

    #[test]
    fn test_complete_soundex_empty() {
        assert_eq!(complete_soundex(""), "");
    }

    #[test]
    fn test_complete_soundex_single_syllable() {
        // Verified against pythainlp output
        assert_eq!(complete_soundex("ก้าน"), "กก1Bน2-");
        assert_eq!(complete_soundex("กลับ"), "กก1Aบ0ล");
        assert_eq!(complete_soundex("ใกล้"), "กก1Aย2ล");
        assert_eq!(complete_soundex("โก่ง"), "กก7Nง1-");
        assert_eq!(complete_soundex("ก้ม"), "กก7Mม2-");
        assert_eq!(complete_soundex("แกน"), "กก6Lน0-");
        assert_eq!(complete_soundex("นา"), "นน1B0--");
        assert_eq!(complete_soundex("ยา"), "ยย1B0--");
        assert_eq!(complete_soundex("ปัน"), "ปป1A0น-");
        assert_eq!(complete_soundex("บุญ"), "บบ4G0น-");
        assert_eq!(complete_soundex("บุณ"), "บบ4G0น-");
        assert_eq!(complete_soundex("ปุญ"), "ปป4G0น-");
        assert_eq!(complete_soundex("ปัญ"), "ปป1A0น-");
    }

    #[test]
    fn test_complete_soundex_asterisk() {
        // ญ as initial → asterisk
        assert_eq!(complete_soundex("ญา"), "ยย1B0--*");
    }

    #[test]
    fn test_complete_soundex_multi_syllable() {
        assert_eq!(complete_soundex("กมล"), "กก1A-0-มม7Mน0-");
        assert_eq!(complete_soundex("ทราย"), "ซซ1Bย0-");
        assert_eq!(complete_soundex("มารค"), "มม1B-ก0-");
    }

    #[test]
    fn test_complete_soundex_cluster() {
        assert_eq!(complete_soundex("เครื่อง"), "คคBVง1ร");
    }

    #[test]
    fn test_complete_soundex_edge_cases() {
        // Single consonant → implicit vowel 7M (โอะ)
        assert_eq!(complete_soundex("ก"), "กก7M-0-");
        // Two consonants → heuristic split: ก-a ก-a
        assert_eq!(complete_soundex("กก"), "กก1A-0-กก1A-0-");
        // Karan removes everything → empty
        assert_eq!(complete_soundex("ก์"), "");
    }

    #[test]
    fn test_complete_soundex_karan_variants() {
        // สวรรค์ → clean → สวรร → aksorn nam split
        assert_eq!(complete_soundex("สวรรค์"), "ซศ1A-0-วว1Aน0-");
        // รักษ์ → clean → รัก
        assert_eq!(complete_soundex("รักษ์"), "รร1Aก0-");
    }
}
