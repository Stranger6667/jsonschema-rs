pub mod content_encoding_content_media_type;
pub mod type_string;
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

    if let Some(type_value) = object.get("type") {
        // Ignoring single_match as by starting to support the new types we will not have a single match anymore
        #[allow(clippy::single_match)]
        match type_value.as_str() {
            Some("string") => {
                if keywords.contains("const")
                    || keywords.contains("minLength")
                    || keywords.contains("minLength")
                {
                    if let Some(validator) = type_string::compile(object, context) {
                        validators.push(validator);
                        keywords.remove("const");
                        keywords.remove("maxLength");
                        keywords.remove("minLength");
                        keywords.remove("type");
                    }
                }
            }
            _ => {}
        };
    }

    keywords
}
