use anyhow::Result as AnyResult;

pub trait Tokenizer {
    fn segment(&self, text: &str, safe: bool, parallel: bool) -> AnyResult<Vec<String>>;

    fn segment_to_string(&self, text: &str, safe: bool, parallel: bool) -> Vec<String>;
}
