use neon::prelude::*;
use oxidized_thainlp_main_lib::tokenizer::{newmm_custom::Newmm, tokenizer_trait::Tokenizer};
// use oxidized_newmm_tokenizer::tokenizer::newmm_custom::Newmm;
use ahash::AHashMap as HashMap;
use lazy_static::lazy_static;
use oxidized_thainlp_main_lib::tokenizer;
use std::{error::Error, sync::Mutex};
lazy_static! {
    static ref  DICT_COLLECTION:Mutex<HashMap<String,Box<Newmm>>> = Mutex::new(HashMap::new());
    // static ref DEFAULT_DICT:Newmm = Newmm::new(None);
}



fn load_dict(mut cx: FunctionContext) -> JsResult<JsString> {

    let mut dict_col_lock = DICT_COLLECTION.lock().unwrap();
    let file_path = cx.argument::<JsString>(0)?.value(&mut cx);
    let dict_name = cx.argument::<JsString>(1)?.value(&mut cx);
    if dict_name  == "default" {
        Ok(cx.string(format!("Failed: 'default' dictionary name is reserved")))
    } else if let Some(_) = dict_col_lock.get(&dict_name) {
        Ok(cx.string(format!(
            "Failed: dictionary {} exists, please use another name.",
            dict_name
        )))
    } else {
        let newmm_dict = Newmm::new(Some(&file_path));
        dict_col_lock.insert(dict_name.to_owned(), Box::new(newmm_dict));

        Ok(cx.string(format!(
            "Successful: dictionary name {} from file {} has been successfully loaded",
            dict_name, file_path
        )))
    }
}
fn segment(
    mut cx: FunctionContext,
) -> JsResult<JsArray> {
    let text = cx.argument::<JsString>(0)?.value(&mut cx);
    let dict_name = cx.argument::<JsString>(1)?.value(&mut cx);
    let safe =  cx.argument::<JsBoolean>(2)?.value(&mut cx);
    let parallel =  cx.argument::<JsBoolean>(3)?.value(&mut cx);
    if let Some(loaded_dict) = DICT_COLLECTION.lock().unwrap().get(&dict_name) {
        let result = loaded_dict.segment_to_string(&text, Some(safe), Some(parallel));
        let js_result_array = JsArray::new(&mut cx, result.len() as u32);
        for (i, obj) in result.iter().enumerate() {
            let js_string = cx.string(obj);
            js_result_array.set(&mut cx, i as u32, js_string).unwrap();
        }
        Ok(js_result_array)
    } else {
        panic!(
            "Dictinary {} does not exist.",
            dict_name)
       
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    {
        let mut dict_collect = DICT_COLLECTION.lock().unwrap();
        dict_collect.insert("default".to_string(), Box::from(Newmm::new(None)));
    }
    cx.export_function("loadDict", load_dict)?;
    cx.export_function("segment", segment)?;
    Ok(())
}

