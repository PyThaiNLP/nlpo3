use ahash::AHashMap as HashMap;
use lazy_static::lazy_static;
use nlpo3::tokenizer;
use nlpo3::tokenizer::newmm_custom::Newmm;
use pyo3::prelude::*;
use pyo3::types::PyString;
use pyo3::{exceptions, wrap_pyfunction};
use std::sync::Mutex;
use tokenizer::tokenizer_trait::Tokenizer;
lazy_static! {
    static ref DICT_COLLECTION:Mutex<HashMap<String,Box<Newmm>>> = Mutex::new(HashMap::new());
    // static ref DEFAULT_DICT:Newmm = Newmm::new(None);
}

// / segment(text, dict_name, safe, parallel, /)
// / --
// /
// / This function is newmm algorithhm.
// 
// / Can use multithreading, but takes a lot of memory.

// / returns list of valid utf-8 bytes list
// / signature:    (text: str, safe = false, parallel = false) -> List[List[u8]]
#[pyfunction]
fn segment(
    text: &PyString,
    dict_name: &str,
    safe: Option<bool>,
    parallel: Option<bool>,
) -> PyResult<Vec<String>> {
    if let Some(loaded_dict) = DICT_COLLECTION.lock().unwrap().get(dict_name) {
        let result = loaded_dict.segment_to_string(text.to_str()?, safe, parallel);
        Ok(result)
    } else {
        Err(exceptions::PyRuntimeError::new_err(format!(
            "Dictionary {} does not exist.",
            dict_name
        )))
    }
}

/// load_dict(file_path, dict_name /)
/// --
///
/// This function loads a dictionary file and add it to dict collection
/// Dict file must be a file of words seperate by line.
/// returns a tuple of string of loading result and a boolean
/// // / signature:    (file_path: str, in_mem_dict_name: str) -> (str,boolean)
#[pyfunction]
fn load_dict(file_path: &str, dict_name: &str) -> PyResult<(String,bool)> {
    let mut dict_col_lock = DICT_COLLECTION.lock().unwrap();
    if dict_col_lock.get(dict_name).is_some() {
        Ok((format!(
            "Failed: dictionary {} already exists, please use another name.",
            dict_name
        ),false))
    } else {
        let newmm_dict = Newmm::new(Some(file_path));
        dict_col_lock.insert(dict_name.to_owned(), Box::new(newmm_dict));

        Ok((format!(
            "Successful: dictionary name {} from file {} has been successfully loaded.",
            dict_name, file_path
        ),true))
    }
}

#[pymodule]
fn _nlpo3_python_backend(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(load_dict, m)?)?;
    m.add_function(wrap_pyfunction!(segment, m)?)?;
    Ok(())
}
