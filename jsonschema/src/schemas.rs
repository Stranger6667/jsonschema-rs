use crate::{compilation::context::CompilationContext, keywords};
use serde_json::{Map, Value};

/// JSON Schema Draft version
#[non_exhaustive]
#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq)]
pub enum Draft {
    /// JSON Schema Draft 4
    Draft4,
    /// JSON Schema Draft 6
    Draft6,
    /// JSON Schema Draft 7
    Draft7,
    #[cfg(feature = "draft201909")]
    /// JSON Schema Draft 2019-09
    Draft201909,

    #[cfg(feature = "draft202012")]
    /// JSON Schema Draft 2020-12
    Draft202012,
}

impl Default for Draft {
    fn default() -> Self {
        Draft::Draft7
    }
}

impl Draft {
    pub(crate) const fn validate_formats_by_default(self) -> bool {
        match self {
            Draft::Draft4 | Draft::Draft6 | Draft::Draft7 => true,
            #[cfg(all(feature = "draft201909", feature = "draft202012"))]
            Draft::Draft201909 | Draft::Draft202012 => false,
            #[cfg(all(feature = "draft201909", not(feature = "draft202012")))]
            Draft::Draft201909 => false,
            #[cfg(all(feature = "draft202012", not(feature = "draft201909")))]
            Draft::Draft202012 => false,
        }
    }
}

type CompileFunc<'a> = fn(
    &'a Map<String, Value>,
    &'a Value,
    &CompilationContext,
) -> Option<keywords::CompilationResult<'a>>;

impl Draft {
    #[allow(clippy::match_same_arms)]
    pub(crate) fn get_validator(self, keyword: &str) -> Option<CompileFunc> {
        match keyword {
            "$ref" => Some(keywords::ref_::compile),
            "additionalItems" => Some(keywords::additional_items::compile),
            "additionalProperties" => Some(keywords::additional_properties::compile),
            "allOf" => Some(keywords::all_of::compile),
            "anyOf" => Some(keywords::any_of::compile),
            "const" => match self {
                Draft::Draft4 => None,
                Draft::Draft6 | Draft::Draft7 => Some(keywords::const_::compile),
                #[cfg(feature = "draft201909")]
                Draft::Draft201909 => Some(keywords::const_::compile),
                #[cfg(feature = "draft202012")]
                Draft::Draft202012 => Some(keywords::const_::compile),
            },
            "contains" => match self {
                Draft::Draft4 => None,
                Draft::Draft6 | Draft::Draft7 => Some(keywords::contains::compile),
                #[cfg(feature = "draft201909")]
                Draft::Draft201909 => Some(keywords::contains::compile),
                #[cfg(feature = "draft202012")]
                Draft::Draft202012 => Some(keywords::contains::compile),
            },
            "contentMediaType" => match self {
                Draft::Draft7 | Draft::Draft6 => Some(keywords::content::compile_media_type),
                Draft::Draft4 => None,
                #[cfg(feature = "draft201909")]
                // Should be collected as an annotation
                Draft::Draft201909 => None,
                #[cfg(feature = "draft202012")]
                Draft::Draft202012 => None,
            },
            "contentEncoding" => match self {
                Draft::Draft7 | Draft::Draft6 => Some(keywords::content::compile_content_encoding),
                Draft::Draft4 => None,
                #[cfg(feature = "draft201909")]
                // Should be collected as an annotation
                Draft::Draft201909 => None,
                #[cfg(feature = "draft202012")]
                Draft::Draft202012 => None,
            },
            "dependencies" => Some(keywords::dependencies::compile),
            #[cfg(any(feature = "draft201909", feature = "draft202012"))]
            "dependentRequired" => Some(keywords::dependencies::compile_dependent_required),
            #[cfg(any(feature = "draft201909", feature = "draft202012"))]
            "dependentSchemas" => Some(keywords::dependencies::compile_dependent_schemas),
            "enum" => Some(keywords::enum_::compile),
            "exclusiveMaximum" => match self {
                Draft::Draft7 | Draft::Draft6 => Some(keywords::exclusive_maximum::compile),
                Draft::Draft4 => None,
                #[cfg(feature = "draft201909")]
                Draft::Draft201909 => Some(keywords::exclusive_maximum::compile),
                #[cfg(feature = "draft202012")]
                Draft::Draft202012 => Some(keywords::exclusive_maximum::compile),
            },
            "exclusiveMinimum" => match self {
                Draft::Draft7 | Draft::Draft6 => Some(keywords::exclusive_minimum::compile),
                Draft::Draft4 => None,
                #[cfg(feature = "draft201909")]
                Draft::Draft201909 => Some(keywords::exclusive_minimum::compile),
                #[cfg(feature = "draft202012")]
                Draft::Draft202012 => Some(keywords::exclusive_minimum::compile),
            },
            "format" => Some(keywords::format::compile),
            "if" => match self {
                Draft::Draft7 => Some(keywords::if_::compile),
                Draft::Draft6 | Draft::Draft4 => None,
                #[cfg(feature = "draft201909")]
                Draft::Draft201909 => Some(keywords::if_::compile),
                #[cfg(feature = "draft202012")]
                Draft::Draft202012 => Some(keywords::if_::compile),
            },
            "items" => Some(keywords::items::compile),
            "maximum" => match self {
                Draft::Draft4 => Some(keywords::legacy::maximum_draft_4::compile),
                Draft::Draft6 | Draft::Draft7 => Some(keywords::maximum::compile),
                #[cfg(feature = "draft201909")]
                Draft::Draft201909 => Some(keywords::maximum::compile),
                #[cfg(feature = "draft202012")]
                Draft::Draft202012 => Some(keywords::maximum::compile),
            },
            "maxItems" => Some(keywords::max_items::compile),
            "maxLength" => Some(keywords::max_length::compile),
            "maxProperties" => Some(keywords::max_properties::compile),
            "minimum" => match self {
                Draft::Draft4 => Some(keywords::legacy::minimum_draft_4::compile),
                Draft::Draft6 | Draft::Draft7 => Some(keywords::minimum::compile),
                #[cfg(feature = "draft201909")]
                Draft::Draft201909 => Some(keywords::minimum::compile),
                #[cfg(feature = "draft202012")]
                Draft::Draft202012 => Some(keywords::minimum::compile),
            },
            "minItems" => Some(keywords::min_items::compile),
            "minLength" => Some(keywords::min_length::compile),
            "minProperties" => Some(keywords::min_properties::compile),
            "multipleOf" => Some(keywords::multiple_of::compile),
            "not" => Some(keywords::not::compile),
            "oneOf" => Some(keywords::one_of::compile),
            "pattern" => Some(keywords::pattern::compile),
            "patternProperties" => Some(keywords::pattern_properties::compile),
            #[cfg(feature = "draft202012")]
            "prefixItems" => Some(keywords::prefix_items::compile),
            "properties" => Some(keywords::properties::compile),
            "propertyNames" => match self {
                Draft::Draft4 => None,
                Draft::Draft6 | Draft::Draft7 => Some(keywords::property_names::compile),
                #[cfg(feature = "draft201909")]
                Draft::Draft201909 => Some(keywords::property_names::compile),
                #[cfg(feature = "draft202012")]
                Draft::Draft202012 => Some(keywords::property_names::compile),
            },
            "required" => Some(keywords::required::compile),
            "type" => match self {
                Draft::Draft4 => Some(keywords::legacy::type_draft_4::compile),
                Draft::Draft6 | Draft::Draft7 => Some(keywords::type_::compile),
                #[cfg(feature = "draft201909")]
                Draft::Draft201909 => Some(keywords::type_::compile),
                #[cfg(feature = "draft202012")]
                Draft::Draft202012 => Some(keywords::type_::compile),
            },
            "unevaluatedProperties" => match self {
                #[cfg(feature = "draft201909")]
                Draft::Draft201909 => Some(keywords::unevaluated_properties::compile),
                #[cfg(feature = "draft202012")]
                Draft::Draft202012 => Some(keywords::unevaluated_properties::compile),
                _ => None,
            },
            "uniqueItems" => Some(keywords::unique_items::compile),
            _ => None,
        }
    }
}

/// Get the `Draft` from a JSON Schema URL.
#[inline]
pub(crate) fn draft_from_url(url: &str) -> Option<Draft> {
    match url {
        #[cfg(feature = "draft202012")]
        "https://json-schema.org/draft/2020-12/schema#" => Some(Draft::Draft202012),
        #[cfg(feature = "draft201909")]
        "https://json-schema.org/draft/2019-09/schema#" => Some(Draft::Draft201909),
        "http://json-schema.org/draft-07/schema#" => Some(Draft::Draft7),
        "http://json-schema.org/draft-06/schema#" => Some(Draft::Draft6),
        "http://json-schema.org/draft-04/schema#" => Some(Draft::Draft4),
        _ => None,
    }
}

/// Get the `Draft` from a JSON Schema.
#[inline]
pub(crate) fn draft_from_schema(schema: &Value) -> Option<Draft> {
    schema
        .get("$schema")
        .and_then(Value::as_str)
        .and_then(draft_from_url)
}

#[inline]
pub(crate) fn id_of(draft: Draft, schema: &Value) -> Option<&str> {
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

    #[cfg_attr(feature = "draft201909", test_case(&json!({"$schema": "https://json-schema.org/draft/2019-09/schema#"}), Some(Draft::Draft201909)))]
    #[cfg_attr(feature = "draft202012", test_case(&json!({"$schema": "https://json-schema.org/draft/2020-12/schema#"}), Some(Draft::Draft202012)))]
    #[test_case(&json!({"$schema": "http://json-schema.org/draft-07/schema#"}), Some(Draft::Draft7))]
    #[test_case(&json!({"$schema": "http://json-schema.org/draft-06/schema#"}), Some(Draft::Draft6))]
    #[test_case(&json!({"$schema": "http://json-schema.org/draft-04/schema#"}), Some(Draft::Draft4))]
    #[test_case(&json!({"$schema": "http://example.com/custom/schema#"}), None)]
    fn test_draft_from_schema(schema: &Value, draft: Option<Draft>) {
        assert_eq!(draft_from_schema(schema), draft)
    }

    #[test]
    fn test_default() {
        assert_eq!(Draft::default(), Draft::Draft7)
    }
}
