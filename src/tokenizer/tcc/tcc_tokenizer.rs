// SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

/**
 * TCC (Thai Character Cluster) tokenizer.
 *
 * Works directly on native UTF-8 `&str` slices — no custom byte encoding
 * required. Character positions are tracked as plain `usize` counts.
 */
use super::tcc_rules::{LOOKAHEAD_TCC, NON_LOOKAHEAD_TCC};

/**
The implementation of tokenizer according to Thai Character Clusters (TCCs)
rules purposed by `Theeramunkong et al. 2000. \
    <http://citeseerx.ist.psu.edu/viewdoc/summary?doi=10.1.1.59.2548>`_

Credits:
    * TCC: Jakkrit TeCho
    * Grammar: Wittawat Jitkrittum (`link to the source file \
      <https://github.com/wittawatj/jtcc/blob/master/TCC.g>`_)
    * Python code: Korakot Chaovavanich
    * Rust code: Thanathip Suntorntip
    * Rust rewrite (native Unicode): PyThaiNLP Project
*/
/// Advance `text` forward by `n` Unicode characters, returning the remainder.
#[inline]
fn advance_chars(text: &str, n: usize) -> &str {
    let byte_offset = text
        .char_indices()
        .nth(n)
        .map(|(b, _)| b)
        .unwrap_or(text.len());
    &text[byte_offset..]
}

/// Count the number of Unicode characters in `text` up to byte offset `end`.
#[inline]
fn byte_end_to_char_count(text: &str, byte_end: usize) -> usize {
    text[..byte_end].chars().count()
}

/// Returns a sorted vector of character indices marking the end of each TCC token in
/// `text`. Character indices are 0-based counts of Unicode scalar values. The vector
/// is naturally sorted because positions are generated left-to-right.
pub fn tcc_pos(text: &str) -> Vec<usize> {
    let total_chars = text.chars().count();
    let mut positions: Vec<usize> = Vec::with_capacity(total_chars / 4 + 1);

    let mut txt = text;
    let mut position: usize = 0;

    while !txt.is_empty() {
        if let Some(m) = NON_LOOKAHEAD_TCC.find(txt) {
            let matched = &txt[..m.end()];
            let match_char_count = byte_end_to_char_count(txt, m.end());

            if LOOKAHEAD_TCC.is_match(matched) {
                // The look-ahead pattern consumed one extra (following) char;
                // trim it off so the TCC ends before that character.
                let segment_char_count = match_char_count - 1;
                position += segment_char_count;
                positions.push(position);
                txt = advance_chars(txt, segment_char_count);
            } else {
                position += match_char_count;
                positions.push(position);
                txt = &txt[m.end()..];
            }
        } else {
            // Non-Thai character: treat as a single-character cluster.
            if let Some(c) = txt.chars().next() {
                txt = &txt[c.len_utf8()..];
                position += 1;
                positions.push(position);
            } else {
                break;
            }
        }
    }

    // The tokenizer relies on binary search over this output in hot paths.
    // Keep this invariant explicit for maintenance and refactors.
    debug_assert!(positions.windows(2).all(|w| w[0] < w[1]));
    debug_assert!(positions.last().copied().unwrap_or(0) <= total_chars);

    positions
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_karan() {
        let kr_result = tcc_pos("พิสูจน์ได้ค่ะ");
        // ends at พิ (position 2)
        assert!(kr_result.contains(&2), "expected position 2");
        // สูจน์ (position 7)
        assert!(kr_result.contains(&7), "expected position 7");
        // ได้ (position 10)
        assert!(kr_result.contains(&10), "expected position 10");
        // ค่ะ (position 13)
        assert!(kr_result.contains(&13), "expected position 13");
    }

    #[test]
    fn test_cluster_general_case() {
        // เรือน้อยลอยอยู่
        // expected clusters: ['เรือ', 'น้', 'อ', 'ย', 'ล', 'อ', 'ย', 'อ', 'ยู่']
        let gen_result = tcc_pos("เรือน้อยลอยอยู่");
        assert!(gen_result.contains(&4), "expected 4");
        assert!(gen_result.contains(&6), "expected 6");
        assert!(gen_result.contains(&7), "expected 7");
        assert!(gen_result.contains(&8), "expected 8");
        assert!(gen_result.contains(&9), "expected 9");
        assert!(gen_result.contains(&10), "expected 10");
        assert!(gen_result.contains(&11), "expected 11");
        assert!(gen_result.contains(&12), "expected 12");
        assert!(gen_result.contains(&15), "expected 15");
    }

    #[test]
    fn test_tcc_pos_is_strictly_increasing_and_in_bounds() {
        let text = "ภาษาไทยภาษาไทยABC123";
        let positions = tcc_pos(text);
        let total_chars = text.chars().count();

        assert!(positions.windows(2).all(|w| w[0] < w[1]));
        assert!(positions.iter().all(|&p| p > 0 && p <= total_chars));
        assert_eq!(positions.last().copied(), Some(total_chars));
    }
}
