//! Python-binding for nlpO3, an natural language process library.
//!
//! Provides a tokenizer.
//!
//! Authors:
//! Thanathip Suntorntip
//! Arthit Suriyawongkul

use std::sync::Mutex;

use ahash::AHashMap as HashMap;
use lazy_static::lazy_static;
use nlpo3::tokenizer::newmm::NewmmTokenizer;
use nlpo3::tokenizer::tokenizer_trait::Tokenizer;
use pyo3::prelude::*;
use pyo3::types::PyString;
use pyo3::{exceptions, wrap_pyfunction};

lazy_static! {
    static ref DICT_COLLECTION: Mutex<HashMap<String, Box<NewmmTokenizer>>> =
        Mutex::new(HashMap::new());
}

/// Break text into tokens.
/// Use newmm algorithhm.
/// Can use multithreading, but takes a lot of memory.
/// returns list of valid utf-8 bytes list
/// signature: (text: str, dict_name: str, safe: boolean = false, parallel: boolean = false) -> List[List[u8]]
#[pyfunction]
fn segment(text: &PyString, dict_name: &str, safe: bool, parallel: bool) -> PyResult<Vec<String>> {
    if let Some(loaded_dict) = DICT_COLLECTION.lock().unwrap().get(dict_name) {
        let result = loaded_dict.segment_to_string(text.to_str()?, safe, parallel);
        Ok(result)
    } else {
        Err(exceptions::PyRuntimeError::new_err(format!(
            "Dictionary name {} does not exist.",
            dict_name
        )))
    }
}

/// Load a dictionary file to the dict collection.
/// Dictionary file must one word per line.
/// returns a tuple of string of loading result and a boolean
/// signature: (file_path: str, dict_name: str) -> (str, boolean)
#[pyfunction]
fn load_dict(file_path: &str, dict_name: &str) -> PyResult<(String, bool)> {
    let mut dict_col_lock = DICT_COLLECTION.lock().unwrap();
    if dict_col_lock.get(dict_name).is_some() {
        Ok((
            format!(
                "Failed: dictionary name {} already exists, please use another name.",
                dict_name
            ),
            false,
        ))
    } else {
        let tokenizer = NewmmTokenizer::new(file_path);
        dict_col_lock.insert(dict_name.to_owned(), Box::new(tokenizer));

        Ok((
            format!(
                "Successful: file {} has been successfully loaded to dictionary name {}.",
                file_path, dict_name
            ),
            true,
        ))
    }
}

/* 
/// Add words to existing dictionary
#[pyfunction]
fn add_word(dict_name: &str, words: Vec<&str>) -> PyResult<(String, bool)> {
    let mut dict_col_lock = DICT_COLLECTION.lock().unwrap();
    if let Some(newmm_dict) = dict_col_lock.get(dict_name) {
        newmm_dict.add_word(&words);
        Ok((format!("Add new word(s) successfully."), true))
    } else {
        Ok((
            format!(
                "Cannot add new word(s) - dictionary instance named '{}' does not exist.",
                dict_name
            ),
            false,
        ))
    }
}

/// Remove words from existing dictionary
#[pyfunction]
fn remove_word(dict_name: &str, words: Vec<&str>) -> PyResult<(String, bool)> {
    let mut dict_col_lock = DICT_COLLECTION.lock().unwrap();
    if let Some(newmm_dict) = dict_col_lock.get(dict_name) {
        newmm_dict.remove_word(&words);
        Ok((format!("Remove word(s) successfully."), true))
    } else {
        Ok((
            format!(
                "Cannot remove word(s) - dictionary instance named '{}' does not exist.",
                dict_name
            ),
            false,
        ))
    }
}
*/

#[pymodule]
fn _nlpo3_python_backend(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(load_dict, m)?)?;
    m.add_function(wrap_pyfunction!(segment, m)?)?;
    Ok(())
}
