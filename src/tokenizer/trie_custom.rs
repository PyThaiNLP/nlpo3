/**
This module is meant to be a direct implementation of Dict Trie in PythaiNLP.

Many functions are implemented as a recursive function because of the limits imposed by
Rust Borrow Checker and this author's (Thanathip) little experience.

Rust Code: Thanathip Suntorntip (Gorlph)
*/
use crate::fixed_bytes_str::four_bytes::{
    CustomString, CustomStringBytesSlice, CustomStringBytesVec, FixedCharsLengthByteSlice,
    BYTES_PER_CHAR,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::borrow::BorrowMut;

#[derive(Debug)]
pub struct TrieNode {
    children: HashMap<CustomStringBytesVec, Self>,
    end: bool,
}

/** FOR CUSTOM 4-BYTE TEXT ONLY */
impl Default for TrieNode {
    fn default() -> Self {
        Self::new()
    }
}

impl TrieNode {
    pub fn new() -> Self {
        Self {
            // text: None,
            children: HashMap::default(),
            end: false,
        }
    }

    fn find_child(&self, word: &CustomStringBytesSlice) -> Option<&Self> {
        self.children.get(word)
    }

    fn remove_child(&mut self, word: &CustomStringBytesSlice) {
        self.children.remove(word);
    }

    fn find_mut_child(&mut self, word: &CustomStringBytesSlice) -> Option<&mut Self> {
        self.children.get_mut(word)
    }

    fn set_not_end(&mut self) {
        self.end = false;
    }

    fn add_word(&mut self, input_word: &CustomStringBytesSlice) {
        // thanks to https://stackoverflow.com/questions/36957286/how-do-you-implement-this-simple-trie-node-in-rust
        if input_word.is_empty() {
            self.end = true;
            return;
        }
        self.children
            .entry((&input_word[0..BYTES_PER_CHAR]).into())
            .or_insert_with(TrieNode::new)
            .add_word(input_word.slice_by_char_indice(1, input_word.chars_len()));
    }

    fn remove_word_from_node(&mut self, input_word: &CustomStringBytesSlice) {
        let mut word = input_word;
        let char_count = word.len() / BYTES_PER_CHAR;
        // if has atleast 1 char
        if word.len() >= BYTES_PER_CHAR {
            let character = &word.slice_by_char_indice(0, 1);
            if let Some(child) = self.find_mut_child(character) {
                // move 1 character
                word = &word[BYTES_PER_CHAR..];
                if char_count == 1 {
                    child.set_not_end();
                }
                child.remove_word_from_node(word);

                if !child.end && child.children.is_empty() {
                    self.remove_child(character);
                }
            };
        }
    }

    pub fn list_prefix<'d, 'p>(
        &'d self,
        prefix: &'p CustomStringBytesSlice,
    ) -> Vec<&'p CustomStringBytesSlice> {
        let mut result: Vec<&CustomStringBytesSlice> = Vec::with_capacity(100);
        let prefix_cpy = prefix;
        let mut current_index = 0;
        let mut current_node_wrap = Some(self);
        while current_index < prefix_cpy.chars_len() {
            let character = prefix_cpy.slice_by_char_indice(current_index, current_index + 1);
            if let Some(current_node) = current_node_wrap {
                if let Some(child) = current_node.find_child(character) {
                    if child.end {
                        let substring_of_prefix =
                            prefix_cpy.slice_by_char_indice(0, current_index + 1);
                        result.push(substring_of_prefix);
                    }
                    current_node_wrap = Some(child);
                } else {
                    break;
                }
            }
            current_index = current_index + 1;
        }
        result
    }
}

#[derive(Debug)]
pub struct Trie {
    words: HashSet<CustomStringBytesVec>,
    root: TrieNode,
}
impl Trie {
    pub fn new(words: &[CustomString]) -> Self {
        let mut instance = Self {
            words: HashSet::default(),
            root: TrieNode::new(),
        };
        for word in words.iter() {
            if !word.is_empty() {
                instance.add(word);
            }
        }
        instance
    }

    fn remove_word_from_set(&mut self, word: &CustomStringBytesSlice) {
        self.words.remove(word);
    }

    pub fn add(&mut self, word: &CustomString) {
        let stripped_word = word.trim();
        self.words.insert(stripped_word.raw_content().into());
        let current_cursor = self.root.borrow_mut();
        current_cursor.add_word(stripped_word.raw_content());
    }

    pub fn remove(&mut self, word: &CustomString) {
        let stripped_word = word.trim();
        let stripped_word_raw = stripped_word.raw_content();
        if self.words.contains(stripped_word_raw) {
            self.remove_word_from_set(stripped_word_raw);
            self.root.remove_word_from_node(stripped_word_raw);
        }
    }

    pub fn prefix<'d, 'p>(
        &'d self,
        prefix: &'p CustomStringBytesSlice,
    ) -> Vec<&'p CustomStringBytesSlice> {
        self.root.list_prefix(prefix)
    }

    /**
       This function differs from prefix(&self) mainly about return type

       prefix_ref is more efficient than prefix(&self) because it performs less allocation.

       The only downside I can think about this function is its non-descriptive name..
    */

    pub fn prefix_ref<'p, 't>(
        prefix: &'p CustomStringBytesSlice,
        dict_trie: &'t Self,
    ) -> Vec<&'p CustomStringBytesSlice> {
        let mut result: Vec<&CustomStringBytesSlice> = Vec::with_capacity(100);
        let prefix_cpy = prefix;
        let mut current_index = 0;
        let mut current_node_wrap = Some(&dict_trie.root);
        while current_index < prefix_cpy.chars_len() {
            let character = prefix_cpy.slice_by_char_indice(current_index, current_index + 1);
            if let Some(current_node) = current_node_wrap {
                if let Some(child) = current_node.find_child(character) {
                    if child.end {
                        let substring_of_prefix =
                            prefix_cpy.slice_by_char_indice(0, current_index + 1);
                        result.push(substring_of_prefix);
                    }
                    current_node_wrap = Some(child);
                } else {
                    break;
                }
            }
            current_index = current_index + 1;
        }
        result
    }

    pub fn contain(&self, word: &CustomStringBytesSlice) -> bool {
        self.words.contains(word)
    }

    pub fn iterate(&self) -> std::collections::hash_set::Iter<'_, Vec<u8>> {
        self.words.iter()
    }

    pub fn amount_of_words(&self) -> usize {
        self.words.len()
    }
}
