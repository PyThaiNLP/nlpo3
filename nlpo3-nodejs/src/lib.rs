// SPDX-FileCopyrightText: 2024 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

use std::sync::Mutex;

use ahash::AHashMap as HashMap;
use lazy_static::lazy_static;
use neon::prelude::*;
use nlpo3::tokenizer::{newmm::NewmmTokenizer, tokenizer_trait::Tokenizer};

lazy_static! {
    static ref TOKENIZER_COLLECTION: Mutex<HashMap<String, Box<NewmmTokenizer>>> =
        Mutex::new(HashMap::new());
}

// Load a dictionary file to a tokenizer,
// and add that tokenizer to the tokenizer collection.
//
// Dictionary file must one word per line.
// If successful, will insert a NewmmTokenizer to TOKENIZER_COLLECTION.
// returns a tuple of string of loading result and a boolean
fn load_dict(mut cx: FunctionContext) -> JsResult<JsString> {
    let mut tokenizer_col_lock = TOKENIZER_COLLECTION.lock().unwrap();
    let file_path = cx.argument::<JsString>(0)?.value(&mut cx);
    let dict_name = cx.argument::<JsString>(1)?.value(&mut cx);
    if let Some(_) = tokenizer_col_lock.get(&dict_name) {
        Ok(cx.string(format!(
            "Failed: dictionary {} exists, please use another name.",
            dict_name
        )))
    } else {
        let tokenizer = NewmmTokenizer::new(&file_path);
        tokenizer_col_lock.insert(dict_name.to_owned(), Box::new(tokenizer));

        Ok(cx.string(format!(
            "Successful: dictionary name {} from file {} has been successfully loaded",
            dict_name, file_path
        )))
    }
}

// Break text into tokens.
// Use newmm algorithm.
/// Can use multithreading, but takes a lot of memory.
/// returns an array of string
fn segment(mut cx: FunctionContext) -> JsResult<JsArray> {
    let text = cx.argument::<JsString>(0)?.value(&mut cx);
    let dict_name = cx.argument::<JsString>(1)?.value(&mut cx);
    let safe = cx.argument::<JsBoolean>(2)?.value(&mut cx);
    let parallel = cx.argument::<JsBoolean>(3)?.value(&mut cx);
    if let Some(loaded_tokenizer) = TOKENIZER_COLLECTION.lock().unwrap().get(&dict_name) {
        let result = loaded_tokenizer.segment_to_string(&text, safe, parallel);
        let js_result_array = JsArray::new(&mut cx, result.len() as u32);
        for (i, obj) in result.iter().enumerate() {
            let js_string = cx.string(obj);
            js_result_array.set(&mut cx, i as u32, js_string).unwrap();
        }
        Ok(js_result_array)
    } else {
        panic!("Dictionary {} does not exist.", dict_name)
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("loadDict", load_dict)?;
    cx.export_function("segment", segment)?;
    Ok(())
}
