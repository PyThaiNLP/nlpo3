// SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

//! Helper utilities for parallel tokenization.
//!
//! Text chunks are split at boundaries that respect Thai Character Cluster (TCC)
//! boundaries and prefer natural punctuation breaks. Never breaks inside a TCC cluster.

use rayon::prelude::*;

/// Context window to search for good break points around the target split location.
const BREAK_SEARCH_WINDOW: usize = 100;

/// Precomputed character/byte position index for a UTF-8 string.
struct TextIndex {
    char_to_byte: Vec<usize>,
    byte_len: usize,
}

impl TextIndex {
    fn new(text: &str) -> Self {
        let mut char_to_byte: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
        char_to_byte.push(text.len());
        Self {
            char_to_byte,
            byte_len: text.len(),
        }
    }

    fn char_count(&self) -> usize {
        self.char_to_byte.len().saturating_sub(1)
    }

    fn char_to_byte(&self, char_index: usize) -> usize {
        self.char_to_byte[char_index.min(self.char_count())]
    }

    fn byte_to_char(&self, byte_offset: usize) -> Option<usize> {
        if byte_offset > self.byte_len {
            return None;
        }
        self.char_to_byte.binary_search(&byte_offset).ok()
    }
}

/// Split text into chunks for parallel processing.
///
/// Chunks respect Thai Character Cluster (TCC) boundaries and prefer breaking at:
/// 1. Sentence-ending punctuation (。!?！？)
/// 2. Whitespace boundaries
/// 3. TCC cluster boundaries (never breaks inside a cluster)
///
/// # Arguments
/// - `text`: The text to chunk
/// - `target_chunk_size`: Approximate target chunk size
/// - `valid_break_points`: Sorted slice of valid character indices where chunking can occur
pub fn split_text_into_chunks<'a>(
    text: &'a str,
    target_chunk_size: usize,
    valid_break_points: &[usize],
) -> Vec<&'a str> {
    let index = TextIndex::new(text);
    split_text_into_chunk_ranges_with_index(text, &index, target_chunk_size, valid_break_points)
        .into_iter()
        .map(|(start, end)| {
            let start_byte = index.char_to_byte(start);
            let end_byte = index.char_to_byte(end);
            &text[start_byte..end_byte]
        })
        .collect()
}

/// Split text into character-index ranges for parallel processing.
///
/// Each returned tuple is `(start_char, end_char)` and always aligned to
/// valid UTF-8 character boundaries and TCC-aware break points.
pub fn split_text_into_chunk_ranges(
    text: &str,
    target_chunk_size: usize,
    valid_break_points: &[usize],
) -> Vec<(usize, usize)> {
    let index = TextIndex::new(text);
    split_text_into_chunk_ranges_with_index(text, &index, target_chunk_size, valid_break_points)
}

fn split_text_into_chunk_ranges_with_index(
    text: &str,
    index: &TextIndex,
    target_chunk_size: usize,
    valid_break_points: &[usize],
) -> Vec<(usize, usize)> {
    debug_assert!(valid_break_points.windows(2).all(|w| w[0] <= w[1]));

    let char_count = index.char_count();

    if target_chunk_size == 0 {
        return vec![(0, char_count)];
    }
    if text.len() <= target_chunk_size {
        return vec![(0, char_count)];
    }

    let est_chunks = text.len().div_ceil(target_chunk_size).max(1);
    let mut chunks: Vec<(usize, usize)> = Vec::with_capacity(est_chunks);
    let mut start: usize = 0; // character index
    while start < char_count {
        let start_byte = index.char_to_byte(start);
        let mut target_end_byte = start_byte.saturating_add(target_chunk_size).min(text.len());
        target_end_byte = clamp_to_char_boundary_right(text, target_end_byte);
        let target_end = index.byte_to_char(target_end_byte).unwrap_or(char_count);
        // Find the best break point near the target
        let break_pos = find_best_break_point(text, index, start, target_end, valid_break_points);
        // Extract chunk from start to break_pos (in byte offsets)
        if break_pos > start {
            chunks.push((start, break_pos));
            start = break_pos;
        } else {
            // Fallback: preserve text and make forward progress.
            let fallback_break_pos =
                find_next_break_in_range(valid_break_points, start.saturating_add(1), char_count)
                    .unwrap_or((start + 1).min(char_count));
            chunks.push((start, fallback_break_pos));
            start = fallback_break_pos;
        }
    }
    chunks
}

/// Find the best break point in the range [start, end] that respects TCC boundaries.
///
/// Preference order:
/// 1. Sentence-ending punctuation with space after (ideal case)
/// 2. Other whitespace at valid TCC positions
/// 3. Valid TCC positions without whitespace
/// 4. Fallback: the target position if no better option
fn find_best_break_point(
    text: &str,
    index: &TextIndex,
    start: usize,
    target_end: usize,
    tcc_positions: &[usize],
) -> usize {
    let start_byte = index.char_to_byte(start);
    let target_byte = index.char_to_byte(target_end.min(index.char_count()));

    // Define the search range: around the target position
    let raw_search_start_byte = if target_byte > BREAK_SEARCH_WINDOW {
        target_byte - BREAK_SEARCH_WINDOW
    } else {
        start_byte
    };
    let raw_search_end_byte = (target_byte + BREAK_SEARCH_WINDOW).min(text.len());
    let search_start_byte = clamp_to_char_boundary_left(text, raw_search_start_byte);
    let search_end_byte = clamp_to_char_boundary_right(text, raw_search_end_byte);

    if search_start_byte >= search_end_byte {
        let end = target_end.min(index.char_count());
        if let Some(pos) = find_prev_break_in_range(tcc_positions, start, end) {
            return pos;
        }
        if let Some(pos) =
            find_next_break_in_range(tcc_positions, end.saturating_add(1), index.char_count())
        {
            return pos;
        }
        return start;
    }

    let search_range = &text[search_start_byte..search_end_byte];

    let best_pos = start; // fallback to "no decision"

    // Look for punctuation breaks (priority: highest)
    if let Some(idx) = search_range.rfind(['。', '!', '?', '！', '？']) {
        let punct_pos = search_start_byte + idx;
        // Prefer a space after punctuation if available
        if let Some(after) = punct_pos.checked_add(get_char_at_offset(text, punct_pos).len_utf8())
            && after < text.len()
            && text[after..].starts_with(' ')
            && let Some(char_idx) = index.byte_to_char(after + 1)
            && is_valid_break_point(tcc_positions, char_idx)
        {
            return char_idx;
        }
        // Fallback to just after the punctuation
        if let Some(char_idx) = index.byte_to_char(punct_pos)
            && let Some(next_idx) = char_idx.checked_add(1)
            && is_valid_break_point(tcc_positions, next_idx)
        {
            return next_idx;
        }
    }

    // Look for whitespace breaks (priority: high)
    if let Some((byte_idx, space_char)) = search_range
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_whitespace())
    {
        let space_byte = search_start_byte + byte_idx;
        if let Some(char_idx) = index.byte_to_char(space_byte.saturating_add(space_char.len_utf8()))
            && is_valid_break_point(tcc_positions, char_idx)
        {
            return char_idx;
        }
    }

    // Fall back to nearest TCC position within search range
    if let Some(pos) =
        find_prev_break_in_range(tcc_positions, start, target_end.min(index.char_count()))
    {
        return pos;
    }

    // Keep boundaries aligned with valid break points by searching forward.
    if let Some(pos) = find_next_break_in_range(
        tcc_positions,
        target_end.saturating_add(1),
        index.char_count(),
    ) {
        return pos;
    }

    best_pos
}

/// Find the largest break point within [start, end].
fn find_prev_break_in_range(sorted_points: &[usize], start: usize, end: usize) -> Option<usize> {
    if sorted_points.is_empty() || start > end {
        return None;
    }

    let insertion = sorted_points.partition_point(|&p| p <= end);
    if insertion == 0 {
        return None;
    }

    let candidate = sorted_points[insertion - 1];
    if candidate >= start {
        Some(candidate)
    } else {
        None
    }
}

/// Find the smallest break point within [start, end].
fn find_next_break_in_range(sorted_points: &[usize], start: usize, end: usize) -> Option<usize> {
    if sorted_points.is_empty() || start > end {
        return None;
    }

    let insertion = sorted_points.partition_point(|&p| p < start);
    let candidate = sorted_points.get(insertion).copied()?;
    if candidate <= end {
        Some(candidate)
    } else {
        None
    }
}

/// Check if a character position is a valid break point using binary search on sorted array.
fn is_valid_break_point(sorted_positions: &[usize], pos: usize) -> bool {
    sorted_positions.binary_search(&pos).is_ok()
}

/// Get the character at a byte offset.
fn get_char_at_offset(text: &str, byte_offset: usize) -> char {
    text[byte_offset..].chars().next().unwrap_or('\0')
}

/// Move offset left to the nearest UTF-8 char boundary.
fn clamp_to_char_boundary_left(text: &str, mut offset: usize) -> usize {
    offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

/// Move offset right to the nearest UTF-8 char boundary.
fn clamp_to_char_boundary_right(text: &str, mut offset: usize) -> usize {
    offset = offset.min(text.len());
    while offset < text.len() && !text.is_char_boundary(offset) {
        offset += 1;
    }
    offset
}

/// Apply a segment function to chunks with optional parallelization.
///
/// - If `parallel=true`, uses Rayon to process chunks in parallel
/// - Otherwise, processes sequentially
pub fn tokenize_chunks<F>(
    chunks: Vec<&str>,
    parallel: bool,
    segment_fn: F,
) -> anyhow::Result<Vec<Vec<String>>>
where
    F: Fn(&str) -> anyhow::Result<Vec<String>> + Send + Sync,
{
    if parallel {
        chunks.into_par_iter().map(segment_fn).collect()
    } else {
        chunks.into_iter().map(segment_fn).collect()
    }
}

/// Flatten a vector of token vectors into a single vector.
pub fn flatten_tokens(token_vecs: Vec<Vec<String>>) -> Vec<String> {
    let total = token_vecs.iter().map(Vec::len).sum();
    let mut out = Vec::with_capacity(total);
    for mut chunk in token_vecs {
        out.append(&mut chunk);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_to_char_boundary_handles_mid_byte_offsets() {
        let text = "AกB";
        let thai_start = text
            .char_indices()
            .find(|(_, ch)| *ch == 'ก')
            .map(|(idx, _)| idx)
            .unwrap();
        let thai_end = thai_start + 'ก'.len_utf8();
        let mid = thai_start + 1;

        assert!(!text.is_char_boundary(mid));
        assert_eq!(clamp_to_char_boundary_left(text, mid), thai_start);
        assert_eq!(clamp_to_char_boundary_right(text, mid), thai_end);
    }

    #[test]
    fn test_split_text_into_chunks_preserves_text_roundtrip() {
        let text = "ภาษาไทยภาษาไทยภาษาไทยABC";
        let char_count = text.chars().count();
        let valid_break_points: Vec<usize> = (0..=char_count).collect();

        let chunks = split_text_into_chunks(text, 8, &valid_break_points);
        assert!(chunks.len() > 1);

        let rebuilt = chunks.concat();
        assert_eq!(rebuilt, text);
    }

    #[test]
    fn test_text_index_roundtrip() {
        let text = "abcกขค";
        let idx = TextIndex::new(text);

        for char_idx in 0..=idx.char_count() {
            let byte = idx.char_to_byte(char_idx);
            assert_eq!(idx.byte_to_char(byte), Some(char_idx));
        }
    }

    #[test]
    fn test_find_prev_break_in_range() {
        let sorted = vec![0, 4, 8, 12, 16];
        assert_eq!(find_prev_break_in_range(&sorted, 5, 11), Some(8));
        assert_eq!(find_prev_break_in_range(&sorted, 13, 15), None);
        assert_eq!(find_prev_break_in_range(&sorted, 0, 2), Some(0));
    }

    #[test]
    fn test_find_next_break_in_range() {
        let sorted = vec![0, 4, 8, 12, 16];
        assert_eq!(find_next_break_in_range(&sorted, 5, 11), Some(8));
        assert_eq!(find_next_break_in_range(&sorted, 13, 15), None);
        assert_eq!(find_next_break_in_range(&sorted, 0, 2), Some(0));
    }

    #[test]
    fn test_split_text_sparse_breakpoints_preserves_text() {
        let text = "ภาษาไทยภาษาไทยภาษาไทยABC";
        let char_count = text.chars().count();
        let valid_break_points: Vec<usize> = [0, char_count].into_iter().collect();

        let chunks = split_text_into_chunks(text, 8, &valid_break_points);
        let rebuilt = chunks.concat();
        assert_eq!(rebuilt, text);
    }

    #[test]
    fn test_split_text_zero_chunk_size_returns_whole_text() {
        let text = "ภาษาไทยภาษาไทยภาษาไทยABC";
        let char_count = text.chars().count();
        let valid_break_points: Vec<usize> = (0..=char_count).collect();

        let chunks = split_text_into_chunks(text, 0, &valid_break_points);
        assert_eq!(chunks, vec![text]);
    }

    #[test]
    fn test_split_text_into_chunk_ranges_preserves_text() {
        let text = "ภาษาไทยภาษาไทยภาษาไทยABC";
        let char_count = text.chars().count();
        let valid_break_points: Vec<usize> = (0..=char_count).collect();

        let ranges = split_text_into_chunk_ranges(text, 8, &valid_break_points);
        let idx = TextIndex::new(text);
        let rebuilt = ranges
            .iter()
            .map(|(start, end)| {
                let s = idx.char_to_byte(*start);
                let e = idx.char_to_byte(*end);
                &text[s..e]
            })
            .collect::<Vec<_>>()
            .concat();

        assert_eq!(rebuilt, text);
    }
}
