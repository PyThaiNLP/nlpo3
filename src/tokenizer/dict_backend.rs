// SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

/**
 * Dictionary backend trait for `NewmmTokenizer`.
 *
 * `DictBackend` is the single interface that `NewmmTokenizer` requires from
 * its dictionary. `TrieChar`, `TrieCharLegacy`, and `FstDict` all implement
 * this trait, which means the tokenizer's string-representation choice
 * (`CharString`) and the dictionary-representation choice are **completely
 * independent**.
 *
 * # Which backend to choose?
 *
 * | Backend | Prefix-lookup speed | Memory | `contain()` | Add/remove |
 * |---------|---------------------|--------|-------------|------------|
 * | `TrieChar` | very fast (O(k) trie) | ~43 MB | O(k) trie walk | O(k) |
 * | `TrieCharLegacy` | very fast (O(k) trie) | ~49 MB | O(1) hash | O(k) |
 * | `FstDict` | slower (O(k·B) FST) | ~0.85 MB | O(1) hash + FST | O(1) delta |
 *
 * Use `TrieChar` (the default) for maximum tokenization speed and lower memory.
 * Use `TrieCharLegacy` when `contain()` is called frequently outside
 * tokenization and O(1) membership tests are important.
 * Use `FstDict` when memory is constrained or the dictionary is large.
 */
use crate::char_string::CharString;

/// The interface required from a dictionary backend used by `NewmmTokenizer`.
///
/// Implementations must be `Send + Sync` to allow the tokenizer to be shared
/// across threads (e.g., in Rayon parallel iterators).
pub trait DictBackend: Send + Sync {
    /// Return the character lengths of all entries in this dictionary that
    /// are a prefix of `prefix`.
    ///
    /// For example, if the dictionary contains "กา" (2 chars) and "กาแฟ"
    /// (4 chars), and `prefix` starts with "กาแฟดี", the result is `[2, 4]`.
    fn prefix_lengths_of(&self, prefix: &CharString) -> Vec<usize>;

    /// Add a word to the dictionary.
    fn add_word(&mut self, word: &CharString);

    /// Remove a word from the dictionary.
    fn remove_word(&mut self, word: &CharString);
}
