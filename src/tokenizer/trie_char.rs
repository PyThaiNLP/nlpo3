/**
This module is meant to be a direct implementation of Dict Trie in PythaiNLP.

Many functions are implemented as a recursive function because of the limits imposed by
Rust Borrow Checker and this author's (Thanathip) little experience.

Rust Code: Thanathip Suntorntip (Gorlph)
*/
use std::borrow::BorrowMut;

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::fixed_bytes_str::four_bytes::{
    CustomString, CustomStringBytesSlice, CustomStringBytesVec, FixedCharsLengthByteSlice,
    BYTES_PER_CHAR,
};

#[derive(Debug)]
struct TrieNode {
    children: HashMap<char, Self>,
    end: bool,
}

impl Default for TrieNode {
    fn default() -> Self {
        Self::new()
    }
}

impl TrieNode {
    pub fn new() -> Self {
        Self {
            children: HashMap::default(),
            end: false,
        }
    }

    fn find_child(&self, word: &char) -> Option<&Self> {
        self.children.get(word)
    }

    #[allow(dead_code)]
    fn remove_child(&mut self, word: &char) {
        self.children.remove(word);
    }

    #[allow(dead_code)]
    fn find_mut_child(&mut self, word: &char) -> Option<&mut Self> {
        self.children.get_mut(word)
    }

    #[allow(dead_code)]
    fn set_not_end(&mut self) {
        self.end = false;
    }

    fn add_word(&mut self, input_word: &CustomString) {
        // thanks to https://stackoverflow.com/questions/36957286/how-do-you-implement-this-simple-trie-node-in-rust
        if input_word.is_empty() {
            self.end = true;
            return;
        }
        self.children
            .entry(*input_word.get_chars_content().get(0).unwrap())
            .or_insert_with(TrieNode::new)
            .add_word(&input_word.substring(1, input_word.chars_len()));
    }

    fn remove_word(&mut self, input_word: &CustomString) {
        let word = input_word;
        let char_count = word.chars_len();
        // if has at least 1 char
        if char_count >= BYTES_PER_CHAR {
            let character = word.get_chars_content().get(0).unwrap();
            if let Some(child) = self.find_mut_child(character) {
                // word = &word.substring(1, word.chars_len());
                if char_count == 1 {
                    child.set_not_end();
                }
                child.remove_word(word);
                if !child.end && child.children.is_empty() {
                    self.remove_child(character);
                }
            };
        }
    }
}

#[derive(Debug)]
// This version of Trie still stores custom bytes vector as words,
// but prefix operation and its node uses char instead.
pub struct TrieChar {
    words: HashSet<CustomStringBytesVec>,
    root: TrieNode,
}

impl TrieChar {
    pub fn new(words: &[CustomString]) -> Self {
        let mut instance = Self {
            words: HashSet::default(),
            root: TrieNode::new(),
        };
        for word in words.iter() {
            instance.add(word);
        }
        instance
    }

    #[allow(dead_code)]
    fn remove_word_from_set(&mut self, word: &CustomString) {
        self.words.remove(word.raw_content());
    }

    pub fn add(&mut self, word: &CustomString) {
        let stripped_word = word.trim();
        if !stripped_word.is_empty() {
            self.words.insert(stripped_word.raw_content().into());
            let current_cursor = self.root.borrow_mut();
            current_cursor.add_word(&stripped_word);
        }
    }

    pub fn remove(&mut self, word: &CustomString) {
        let stripped_word = word.trim();
        if !stripped_word.is_empty() {
            if self.words.contains(stripped_word.raw_content()) {
                self.remove_word_from_set(&stripped_word);
                self.root.remove_word(&stripped_word); // remove from node
            }
        }
    }

    pub fn contain(&self, word: &CustomString) -> bool {
        self.words.contains(word.raw_content())
    }

    pub fn iterate(&self) -> std::collections::hash_set::Iter<'_, Vec<u8>> {
        self.words.iter()
    }

    pub fn amount_of_words(&self) -> usize {
        self.words.len()
    }

    pub fn prefix_ref<'p, 't>(
        prefix: &'p CustomString,
        dict_trie: &'t Self,
    ) -> Vec<&'p CustomStringBytesSlice> {
        let mut result: Vec<&[u8]> = Vec::with_capacity(100);
        let prefix_cpy = prefix;
        let mut current_index = 0;
        let mut current_node_wrap = Some(&dict_trie.root);
        while current_index < prefix_cpy.chars_len() {
            let character = prefix_cpy.get_char_at(current_index);
            if let Some(current_node) = current_node_wrap {
                if let Some(child) = current_node.find_child(&character) {
                    if child.end {
                        let substring_of_prefix = prefix_cpy
                            .raw_content()
                            .slice_by_char_indice(0, current_index + 1);
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
