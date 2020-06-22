pub mod content_encoding_content_media_type;
use crate::{compilation::CompilationContext, keywords::BoxedValidator};
use serde_json::{Map, Value};
use std::collections::HashSet;

pub(crate) fn compile_combined_validators(
    validators: &mut Vec<BoxedValidator>,
    context: &CompilationContext,
    object: &Map<String, Value>,
) -> HashSet<String> {
    let mut keywords: HashSet<_> = object.keys().cloned().collect();

    if keywords.contains("contentEncoding") && keywords.contains("contentMediaType") {
        if let Some(validator) = content_encoding_content_media_type::compile(object, context) {
            validators.push(validator);
            keywords.remove("contentMediaType");
            keywords.remove("contentEncoding");
        }
    }

    keywords
}
