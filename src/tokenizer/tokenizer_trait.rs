use anyhow::Result as AnyResult;
pub trait Tokenizer {
    fn segment(
        &self,
        text: &str,
        safe: Option<bool>,
        parallel: Option<bool>,
    ) -> AnyResult<Vec<String>>;

    fn segment_to_string(
        &self,
        text: &str,
        safe: Option<bool>,
        parallel: Option<bool>,
    ) -> Vec<String>;

    fn segment_with_cache(
        &self,
        text: &str,
        safe: Option<bool>,
        parallel: Option<bool>,
    ) -> AnyResult<Vec<String>>;

}
