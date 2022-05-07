use crate::vocabularies::{Keyword, Validate};
use std::ops::Range;

#[derive(Debug)]
pub struct Ref {
    pub(crate) reference: String,
    pub(crate) range: Range<usize>,
}

impl Validate for Ref {
    fn is_valid(&self, keywords: &[Keyword], instance: &serde_json::Value) -> bool {
        keywords[self.range.clone()]
            .iter()
            .all(|keyword| keyword.is_valid(keywords, instance))
    }
}
