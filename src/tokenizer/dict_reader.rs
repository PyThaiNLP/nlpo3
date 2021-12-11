use crate::four_bytes_str::custom_string::CustomString;

use super::super::four_bytes_str::custom_string;
use super::trie_char_ver::TrieChar as Trie;
use rayon::prelude::*;
use std::io::BufReader;
use std::{error::Error, io::prelude::*};
use std::{fs::File, path::PathBuf};

use super::super::fixed_bytes_str::four_bytes::CustomString;
use super::trie_char::TrieChar as Trie;

pub enum DictSource {
    FilePath(PathBuf),
    WordList(Vec<String>),
}

pub fn create_dict_trie(source: DictSource) -> Result<Trie, Box<dyn Error>> {
    match source {
        DictSource::FilePath(file_path) => {
            let file_reader = File::open(file_path.as_path());
            match file_reader {
                Ok(file) => {
                    let mut reader = BufReader::new(file);
                    let mut line = String::with_capacity(50);
                    let mut dict: Vec<CustomString> = Vec::with_capacity(600);
                    while reader.read_line(&mut line).unwrap() != 0 {
                        dict.push(CustomString::new(&line));
                        line.clear();
                    }
                    dict.shrink_to_fit();
                    Ok(Trie::new(&dict))
                }
                Err(error) => Err(Box::from(error)),
            }
        }
        DictSource::WordList(word_list) => {
            let custom_word_list: Vec<CustomString> = word_list
                .into_iter()
                .map(|word| CustomString::new(&word))
                .collect();
            Ok(Trie::new(&custom_word_list))
        }
    }
}

#[test]
fn test_trie() {
    let test_word_list = vec![
        "กากบาท".to_string(),
        "กาแฟ".to_string(),
        "กรรม".to_string(),
        "42".to_string(),
        "aง|.%".to_string(),
    ];
    let trie = create_dict_trie(DictSource::WordList(test_word_list)).unwrap();
    assert_eq!(trie.contain(&CustomString::new("กาแฟ")), true);
    assert_eq!(trie.amount_of_words(), 5);
}
