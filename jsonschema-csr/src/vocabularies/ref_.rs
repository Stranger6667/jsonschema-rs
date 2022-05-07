use crate::vocabularies::{Keyword, Validate};
use std::ops::Range;

const EMPTY_RANGE: Range<usize> = usize::MAX..usize::MAX;

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

pub(crate) fn compile(
    schema: &serde_json::Value,
    reference: String,
    global: &mut [Keyword],
) -> Keyword {
    // Reference
    //   - `#` - should point to the root scope. not yet evaluated
    //   - `#/other/place` - how to find it?
    //      - already in `global` - can I find it? should I?
    //      - not in `global`
    //   - `https://whatever.com/schema.json#/something` - ???
    // Ideas:
    //   - Collect all references separately? then
    Ref {
        reference,
        range: EMPTY_RANGE,
    }
    .into()
}
