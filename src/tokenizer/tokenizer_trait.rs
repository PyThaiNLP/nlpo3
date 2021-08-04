/**
   This should be the only part exposed to lib.rs
*/
use anyhow::Result as AnyResult;
pub trait Tokenizer {
    // I need help naming these two functions...
    
    fn segment(&self, text: &str, safe: Option<bool>, parallel: Option<bool>) -> AnyResult<Vec<String>>;

    fn segment_to_string(
        &self,
        text: &str,
        safe: Option<bool>,
        parallel: Option<bool>,
    ) -> Vec<String>;
}
