// SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

/**
 * Rules for TCC (Thai Character Cluster) tokenization.
 *
 * Patterns are written directly as native Unicode regular expressions and
 * compiled with `regex::Regex`. The legacy four-byte encoded patterns and
 * the `custom_regex` conversion layer have been removed.
 */
use lazy_static::lazy_static;
use regex::Regex;

// Pattern shorthand substitutions:
//   c → [ก-ฮ]        (Thai consonant)
//   t → [่-๋]?       (optional tone mark)
//   k → (cc?[dิ]?[์])? (optional final consonant group, d = [ุู])
//   d → [ุู]
pub fn replace_tcc_symbol(tcc_pattern: &str) -> String {
    tcc_pattern
        .replace('k', "(cc?[dิ]?[์])?")
        .replace('c', "[ก-ฮ]")
        .replace('t', "[่-๋]?")
        .replace('d', &"อูอุ".replace('อ', ""))
}

lazy_static! {
    pub static ref NON_LOOKAHEAD_TCC: Regex = Regex::new(
        &[
            r"^เc็ck",         // 1
            r"^เcctาะk",       // 2
            r"^เccีtยะk",      // 3
            r"^เcc็ck",        // 4
            r"^เcิc์ck",       // 5
            r"^เcิtck",        // 6
            r"^เcีtยะ?k",      // 7
            r"^เcืtอะ?k",      // 8
            r"^เctา?ะ?k",      // 9
            r"^cัtวะk",        // 10
            r"^c[ัื]tc[ุิะ]?k", // 11
            r"^c[ิุู]์k",      // 12
            r"^c[ะ-ู]tk",      // 13
            r"^cรรc์ ็",       // 14
            r"^c็",            // 15
            r"^ct[ะาำ]?k",     // 16
            r"^ck",            // 17
            r"^แc็c",          // 18
            r"^แcc์",          // 19
            r"^แctะ",          // 20
            r"^แcc็c",         // 21
            r"^แccc์",         // 22
            r"^โctะ",          // 23
            r"^[เ-ไ]ct",       // 24
            r"^ก็",
            r"^อึ",
            r"^หึ",
            r"^(เccีtย)[เ-ไก-ฮ]k",   // look ahead 1
            r"^(เc[ิีุู]tย)[เ-ไก-ฮ]k", // look ahead 2
        ]
        .map(replace_tcc_symbol)
        .join("|")
    )
    .expect("tcc_rules: NON_LOOKAHEAD_TCC must be a valid static regex");

    pub static ref LOOKAHEAD_TCC: Regex = Regex::new(
        &[
            r"^(เccีtย)[เ-ไก-ฮ]k",    // เccีtย(?=[เ-ไก-ฮ]|$)
            r"^(เc[ิีุู]tย)[เ-ไก-ฮ]k", // เc[ิีุู]tย(?=[เ-ไก-ฮ]|$)
        ]
        .map(replace_tcc_symbol)
        .join("|")
    )
    .expect("tcc_rules: LOOKAHEAD_TCC must be a valid static regex");
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tcc_regex_compile() {
        // Verify that the compiled regexes can match expected Thai clusters.
        // เc็c pattern (case 1)
        assert!(NON_LOOKAHEAD_TCC.is_match("เก็ก"));
        // cัtวะ pattern (case 10)
        assert!(NON_LOOKAHEAD_TCC.is_match("กัวะ"));
        // Simple consonant (case 17)
        assert!(NON_LOOKAHEAD_TCC.is_match("ก"));
    }

    #[test]
    fn tcc_lookahead_compile() {
        // Look-ahead patterns should match when followed by another Thai char.
        assert!(LOOKAHEAD_TCC.is_match("เกียก"));
    }
}
