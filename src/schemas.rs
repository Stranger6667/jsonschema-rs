use crate::{compilation::CompilationContext, keywords};
use serde_json::{Map, Value};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Draft {
    Draft4,
    Draft6,
    Draft7,
}

type CompileFunc =
    fn(&Map<String, Value>, &Value, &CompilationContext) -> Option<keywords::CompilationResult>;

impl Draft {
    pub(crate) fn get_validator(self, keyword: &str) -> Option<CompileFunc> {
        match self {
            Draft::Draft7 => match keyword {
                "additionalItems" => Some(keywords::additional_items::compile),
                "additionalProperties" => Some(keywords::additional_properties::compile),
                "allOf" => Some(keywords::all_of::compile),
                "anyOf" => Some(keywords::any_of::compile),
                "const" => Some(keywords::const_::compile),
                "contains" => Some(keywords::contains::compile),
                "contentMediaType" => Some(keywords::content::compile_media_type),
                "contentEncoding" => Some(keywords::content::compile_content_encoding),
                "dependencies" => Some(keywords::dependencies::compile),
                "enum" => Some(keywords::enum_::compile),
                "exclusiveMaximum" => Some(keywords::exclusive_maximum::compile),
                "exclusiveMinimum" => Some(keywords::exclusive_minimum::compile),
                "format" => Some(keywords::format::compile),
                "if" => Some(keywords::if_::compile),
                "items" => Some(keywords::items::compile),
                "maximum" => Some(keywords::maximum::compile),
                "maxItems" => Some(keywords::max_items::compile),
                "maxLength" => Some(keywords::max_length::compile),
                "maxProperties" => Some(keywords::max_properties::compile),
                "minimum" => Some(keywords::minimum::compile),
                "minItems" => Some(keywords::min_items::compile),
                "minLength" => Some(keywords::min_length::compile),
                "minProperties" => Some(keywords::min_properties::compile),
                "multipleOf" => Some(keywords::multiple_of::compile),
                "not" => Some(keywords::not::compile),
                "oneOf" => Some(keywords::one_of::compile),
                "pattern" => Some(keywords::pattern::compile),
                "patternProperties" => Some(keywords::pattern_properties::compile),
                "properties" => Some(keywords::properties::compile),
                "propertyNames" => Some(keywords::property_names::compile),
                "required" => Some(keywords::required::compile),
                "type" => Some(keywords::type_::compile),
                "uniqueItems" => Some(keywords::unique_items::compile),
                _ => None,
            },
            Draft::Draft6 => match keyword {
                "additionalItems" => Some(keywords::additional_items::compile),
                "additionalProperties" => Some(keywords::additional_properties::compile),
                "allOf" => Some(keywords::all_of::compile),
                "anyOf" => Some(keywords::any_of::compile),
                "const" => Some(keywords::const_::compile),
                "contains" => Some(keywords::contains::compile),
                "contentMediaType" => Some(keywords::content::compile_media_type),
                "contentEncoding" => Some(keywords::content::compile_content_encoding),
                "dependencies" => Some(keywords::dependencies::compile),
                "enum" => Some(keywords::enum_::compile),
                "exclusiveMaximum" => Some(keywords::exclusive_maximum::compile),
                "exclusiveMinimum" => Some(keywords::exclusive_minimum::compile),
                "format" => Some(keywords::format::compile),
                "items" => Some(keywords::items::compile),
                "maximum" => Some(keywords::maximum::compile),
                "maxItems" => Some(keywords::max_items::compile),
                "maxLength" => Some(keywords::max_length::compile),
                "maxProperties" => Some(keywords::max_properties::compile),
                "minimum" => Some(keywords::minimum::compile),
                "minItems" => Some(keywords::min_items::compile),
                "minLength" => Some(keywords::min_length::compile),
                "minProperties" => Some(keywords::min_properties::compile),
                "multipleOf" => Some(keywords::multiple_of::compile),
                "not" => Some(keywords::not::compile),
                "oneOf" => Some(keywords::one_of::compile),
                "pattern" => Some(keywords::pattern::compile),
                "patternProperties" => Some(keywords::pattern_properties::compile),
                "properties" => Some(keywords::properties::compile),
                "propertyNames" => Some(keywords::property_names::compile),
                "required" => Some(keywords::required::compile),
                "type" => Some(keywords::type_::compile),
                "uniqueItems" => Some(keywords::unique_items::compile),
                _ => None,
            },
            Draft::Draft4 => match keyword {
                "additionalItems" => Some(keywords::additional_items::compile),
                "additionalProperties" => Some(keywords::additional_properties::compile),
                "allOf" => Some(keywords::all_of::compile),
                "anyOf" => Some(keywords::any_of::compile),
                "dependencies" => Some(keywords::dependencies::compile),
                "enum" => Some(keywords::enum_::compile),
                "format" => Some(keywords::format::compile),
                "items" => Some(keywords::items::compile),
                "maximum" => Some(keywords::legacy::maximum_draft_4::compile),
                "maxItems" => Some(keywords::max_items::compile),
                "maxLength" => Some(keywords::max_length::compile),
                "maxProperties" => Some(keywords::max_properties::compile),
                "minimum" => Some(keywords::legacy::minimum_draft_4::compile),
                "minItems" => Some(keywords::min_items::compile),
                "minLength" => Some(keywords::min_length::compile),
                "minProperties" => Some(keywords::min_properties::compile),
                "multipleOf" => Some(keywords::multiple_of::compile),
                "not" => Some(keywords::not::compile),
                "oneOf" => Some(keywords::one_of::compile),
                "pattern" => Some(keywords::pattern::compile),
                "patternProperties" => Some(keywords::pattern_properties::compile),
                "properties" => Some(keywords::properties::compile),
                "required" => Some(keywords::required::compile),
                "type" => Some(keywords::legacy::type_draft_4::compile),
                "uniqueItems" => Some(keywords::unique_items::compile),
                _ => None,
            },
        }
    }
}

/// Get the `Draft` from a JSON Schema URL.
pub fn draft_from_url(url: &str) -> Option<Draft> {
    match url {
        "http://json-schema.org/draft-07/schema#" => Some(Draft::Draft7),
        "http://json-schema.org/draft-06/schema#" => Some(Draft::Draft6),
        "http://json-schema.org/draft-04/schema#" => Some(Draft::Draft4),
        _ => None,
    }
}

/// Get the `Draft` from a JSON Schema.
pub fn draft_from_schema(schema: &Value) -> Option<Draft> {
    schema
        .as_object()
        .and_then(|x| x.get("$schema"))
        .and_then(Value::as_str)
        .and_then(|x| draft_from_url(x))
}

pub fn id_of(draft: Draft, schema: &Value) -> Option<&str> {
    if let Value::Object(object) = schema {
        if draft == Draft::Draft4 {
            object.get("id")
        } else {
            object.get("$id")
        }
        .and_then(Value::as_str)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!({"$schema": "http://json-schema.org/draft-07/schema#"}), Some(Draft::Draft7))]
    #[test_case(json!({"$schema": "http://json-schema.org/draft-06/schema#"}), Some(Draft::Draft6))]
    #[test_case(json!({"$schema": "http://json-schema.org/draft-04/schema#"}), Some(Draft::Draft4))]
    #[test_case(json!({"$schema": "http://example.com/custom/schema#"}), None)]
    fn test_draft_from_schema(schema: Value, draft: Option<Draft>) {
        assert_eq!(draft_from_schema(&schema), draft)
    }
}
