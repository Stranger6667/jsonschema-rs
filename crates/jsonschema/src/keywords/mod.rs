pub(crate) mod additional_items;
pub(crate) mod additional_properties;
pub(crate) mod all_of;
pub(crate) mod any_of;
pub(crate) mod boolean;
pub(crate) mod const_;
pub(crate) mod contains;
pub(crate) mod content;
pub(crate) mod custom;
pub(crate) mod dependencies;
pub(crate) mod enum_;
pub(crate) mod exclusive_maximum;
pub(crate) mod exclusive_minimum;
pub(crate) mod format;
pub(crate) mod helpers;
pub(crate) mod if_;
pub(crate) mod items;
pub(crate) mod legacy;
pub(crate) mod max_items;
pub(crate) mod max_length;
pub(crate) mod max_properties;
pub(crate) mod maximum;
pub(crate) mod min_items;
pub(crate) mod min_length;
pub(crate) mod min_properties;
pub(crate) mod minimum;
pub(crate) mod multiple_of;
pub(crate) mod not;
pub(crate) mod one_of;
pub(crate) mod pattern;
pub(crate) mod pattern_properties;
pub(crate) mod prefix_items;
pub(crate) mod properties;
pub(crate) mod property_names;
pub(crate) mod ref_;
pub(crate) mod required;
pub(crate) mod type_;
pub(crate) mod unevaluated_properties;
pub(crate) mod unique_items;
use referencing::Draft;
use serde_json::{Map, Value};

use crate::{compiler, error, validator::Validate};

pub(crate) type CompilationResult<'a> = Result<BoxedValidator, error::ValidationError<'a>>;
pub(crate) type BoxedValidator = Box<dyn Validate + Send + Sync>;

type CompileFunc<'a> =
    fn(&compiler::Context, &'a Map<String, Value>, &'a Value) -> Option<CompilationResult<'a>>;

pub(crate) fn get_for_draft(draft: Draft, keyword: &str) -> Option<CompileFunc> {
    match (draft, keyword) {
        // Keywords common to all drafts
        (_, "$ref") => Some(ref_::compile),
        (_, "additionalItems") => Some(additional_items::compile),
        (_, "additionalProperties") => Some(additional_properties::compile),
        (_, "allOf") => Some(all_of::compile),
        (_, "anyOf") => Some(any_of::compile),
        (_, "dependencies") => Some(dependencies::compile),
        (_, "enum") => Some(enum_::compile),
        (_, "format") => Some(format::compile),
        (_, "items") => Some(items::compile),
        (_, "maxItems") => Some(max_items::compile),
        (_, "maxLength") => Some(max_length::compile),
        (_, "maxProperties") => Some(max_properties::compile),
        (_, "minItems") => Some(min_items::compile),
        (_, "minLength") => Some(min_length::compile),
        (_, "minProperties") => Some(min_properties::compile),
        (_, "multipleOf") => Some(multiple_of::compile),
        (_, "not") => Some(not::compile),
        (_, "oneOf") => Some(one_of::compile),
        (_, "pattern") => Some(pattern::compile),
        (_, "patternProperties") => Some(pattern_properties::compile),
        (_, "properties") => Some(properties::compile),
        (_, "required") => Some(required::compile),
        (_, "uniqueItems") => Some(unique_items::compile),

        // Draft 4 specific
        (Draft::Draft4, "maximum") => Some(legacy::maximum_draft_4::compile),
        (Draft::Draft4, "minimum") => Some(legacy::minimum_draft_4::compile),
        (Draft::Draft4, "type") => Some(legacy::type_draft_4::compile),

        // Draft 6 and later
        (Draft::Draft6 | Draft::Draft7 | Draft::Draft201909 | Draft::Draft202012, "const") => {
            Some(const_::compile)
        }
        (Draft::Draft6 | Draft::Draft7 | Draft::Draft201909 | Draft::Draft202012, "contains") => {
            Some(contains::compile)
        }
        (
            Draft::Draft6 | Draft::Draft7 | Draft::Draft201909 | Draft::Draft202012,
            "exclusiveMaximum",
        ) => Some(exclusive_maximum::compile),
        (
            Draft::Draft6 | Draft::Draft7 | Draft::Draft201909 | Draft::Draft202012,
            "exclusiveMinimum",
        ) => Some(exclusive_minimum::compile),
        (Draft::Draft6 | Draft::Draft7 | Draft::Draft201909 | Draft::Draft202012, "maximum") => {
            Some(maximum::compile)
        }
        (Draft::Draft6 | Draft::Draft7 | Draft::Draft201909 | Draft::Draft202012, "minimum") => {
            Some(minimum::compile)
        }
        (
            Draft::Draft6 | Draft::Draft7 | Draft::Draft201909 | Draft::Draft202012,
            "propertyNames",
        ) => Some(property_names::compile),
        (Draft::Draft6 | Draft::Draft7 | Draft::Draft201909 | Draft::Draft202012, "type") => {
            Some(type_::compile)
        }

        // Draft 6 and 7 specific
        (Draft::Draft6 | Draft::Draft7, "contentMediaType") => Some(content::compile_media_type),
        (Draft::Draft6 | Draft::Draft7, "contentEncoding") => {
            Some(content::compile_content_encoding)
        }

        // Draft 7 and later
        (Draft::Draft7 | Draft::Draft201909 | Draft::Draft202012, "if") => Some(if_::compile),

        // Draft 2019-09 specific
        (Draft::Draft201909, "$recursiveRef") => Some(ref_::compile_recursive_ref),

        // Draft 2019-09 and 2020-12 specific
        (Draft::Draft201909 | Draft::Draft202012, "dependentRequired") => {
            Some(dependencies::compile_dependent_required)
        }
        (Draft::Draft201909 | Draft::Draft202012, "dependentSchemas") => {
            Some(dependencies::compile_dependent_schemas)
        }
        (Draft::Draft201909 | Draft::Draft202012, "prefixItems") => Some(prefix_items::compile),
        (Draft::Draft201909 | Draft::Draft202012, "unevaluatedProperties") => {
            Some(unevaluated_properties::compile)
        }

        // Draft 2020-12 specific
        (Draft::Draft202012, "$dynamicRef") => Some(ref_::compile),

        // Unknown or not-yet-implemented keyword
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"items": [{}], "additionalItems": {"type": "integer"}}), &json!([ null, 2, 3, "foo" ]), r#""foo" is not of type "integer""#)]
    #[test_case(&json!({"items": [{}, {}, {}], "additionalItems": false}), &json!([ 1, 2, 3, 4 ]), r#"Additional items are not allowed (4 was unexpected)"#)]
    #[test_case(&json!({"items": [{}, {}, {}], "additionalItems": false}), &json!([ 1, 2, 3, 4, 5 ]), r#"Additional items are not allowed (4, 5 were unexpected)"#)]
    #[test_case(&json!({"properties": {"foo": {}, "bar": {}}, "patternProperties": { "^v": {} }, "additionalProperties": false}), &json!({"foo" : 1, "bar" : 2, "quux" : "boom"}), r#"Additional properties are not allowed ('quux' was unexpected)"#)]
    #[test_case(&json!({"anyOf": [{"type": "integer"}, {"minimum": 2}]}), &json!(1.5), r#"1.5 is not valid under any of the schemas listed in the 'anyOf' keyword"#)]
    #[test_case(&json!({"const": 2}), &json!(5), r#"2 was expected"#)]
    #[test_case(&json!({"contains": {"minimum": 5}}), &json!([2, 3, 4]), r#"None of [2,3,4] are valid under the given schema"#)]
    #[test_case(&json!({"enum": [1, 2, 3]}), &json!(4), r#"4 is not one of [1,2,3]"#)]
    #[test_case(&json!({"exclusiveMaximum": 3}), &json!(3.0), r#"3.0 is greater than or equal to the maximum of 3"#)]
    #[test_case(&json!({"exclusiveMaximum": 3.0}), &json!(3.0), r#"3.0 is greater than or equal to the maximum of 3.0"#)]
    #[test_case(&json!({"exclusiveMinimum": 1}), &json!(1.0), r#"1.0 is less than or equal to the minimum of 1"#)]
    #[test_case(&json!({"exclusiveMinimum": 1.0}), &json!(1), r#"1 is less than or equal to the minimum of 1.0"#)]
    #[test_case(&json!({"format": "ipv4"}), &json!("2001:0db8:85a3:0000:0000:8a2e:0370:7334"), r#""2001:0db8:85a3:0000:0000:8a2e:0370:7334" is not a "ipv4""#)]
    #[test_case(&json!({"maximum": 3}), &json!(3.5), r#"3.5 is greater than the maximum of 3"#)]
    #[test_case(&json!({"maximum": 3.0}), &json!(3.5), r#"3.5 is greater than the maximum of 3.0"#)]
    #[test_case(&json!({"minimum": 3}), &json!(2.5), r#"2.5 is less than the minimum of 3"#)]
    #[test_case(&json!({"minimum": 3.0}), &json!(2.5), r#"2.5 is less than the minimum of 3.0"#)]
    #[test_case(&json!({"maxItems": 2}), &json!([1, 2, 3]), r#"[1,2,3] has more than 2 items"#)]
    #[test_case(&json!({"maxLength": 2}), &json!("foo"), r#""foo" is longer than 2 characters"#)]
    #[test_case(&json!({"maxProperties": 2}), &json!({"foo": 1, "bar": 2, "baz": 3}), r#"{"bar":2,"baz":3,"foo":1} has more than 2 properties"#)]
    #[test_case(&json!({"minimum": 1.1}), &json!(0.6), r#"0.6 is less than the minimum of 1.1"#)]
    #[test_case(&json!({"minItems": 1}), &json!([]), r#"[] has less than 1 item"#)]
    #[test_case(&json!({"minLength": 2}), &json!("f"), r#""f" is shorter than 2 characters"#)]
    #[test_case(&json!({"minProperties": 1}), &json!({}), r#"{} has less than 1 property"#)]
    #[test_case(&json!({"multipleOf": 2}), &json!(7), r#"7 is not a multiple of 2"#)]
    #[test_case(&json!({"not": {"type": "integer"}}), &json!(1), r#"{"type":"integer"} is not allowed for 1"#)]
    #[test_case(&json!({"oneOf": [{"type": "integer"}, {"minimum": 2}]}), &json!(1.1), r#"1.1 is not valid under any of the schemas listed in the 'oneOf' keyword"#)]
    #[test_case(&json!({"oneOf": [{"type": "integer"}, {"minimum": 2}]}), &json!(3), r#"3 is valid under more than one of the schemas listed in the 'oneOf' keyword"#)]
    #[test_case(&json!({"pattern": "^a*$"}), &json!("abc"), r#""abc" does not match "^a*$""#)]
    #[test_case(&json!({"properties": {"foo": {}, "bar": {}}, "required": ["foo"]}), &json!({"bar": 1}), r#""foo" is a required property"#)]
    #[test_case(&json!({"type": "integer"}), &json!(1.1), r#"1.1 is not of type "integer""#)]
    #[test_case(&json!({"type": ["integer", "string"]}), &json!(null), r#"null is not of types "integer", "string""#)]
    #[test_case(&json!({"uniqueItems": true}), &json!([1, 1]), r#"[1,1] has non-unique elements"#)]
    fn error_message(schema: &Value, instance: &Value, expected: &str) {
        let validator = crate::validator_for(schema).unwrap();
        let errors: Vec<_> = validator
            .validate(instance)
            .expect_err(&format!(
                "Validation error is expected. Schema=`{:?}` Instance=`{:?}`",
                schema, instance
            ))
            .collect();
        assert_eq!(errors[0].to_string(), expected);
    }

    // Extra cases not covered by JSON test suite
    #[test_case(&json!({"additionalProperties": {"type": "string"}}))]
    #[test_case(&json!({"additionalProperties": {"type": "string"}, "properties": {"foo": {}}}))]
    #[test_case(&json!({"additionalProperties": {"type": "string"}, "patternProperties": {"f.*o": {"type": "integer"}}}))]
    #[test_case(&json!({"additionalProperties": {"type": "string"}, "properties": {"foo": {}}, "patternProperties": {"f.*o": {"type": "integer"}}}))]
    #[test_case(&json!({"additionalProperties": false}))]
    #[test_case(&json!({"additionalProperties": false, "properties": {"foo": {}}}))]
    #[test_case(&json!({"additionalProperties": false, "patternProperties": {"f.*o": {"type": "integer"}}}))]
    #[test_case(&json!({"additionalProperties": false, "properties": {"foo": {}}, "patternProperties": {"f.*o": {"type": "integer"}}}))]
    #[test_case(&json!({"additionalItems": false, "items": [{"type": "string"}]}))]
    #[test_case(&json!({"additionalItems": {"type": "integer"}, "items": [{"type": "string"}]}))]
    #[test_case(&json!({"contains": {"minimum": 5}}))]
    #[test_case(&json!({"contentMediaType": "application/json"}))]
    #[test_case(&json!({"contentEncoding": "base64"}))]
    #[test_case(&json!({"contentEncoding": "base64", "contentMediaType": "application/json"}))]
    #[test_case(&json!({"dependencies": {"bar": ["foo"]}}))]
    #[test_case(&json!({"exclusiveMaximum": 5}))]
    #[test_case(&json!({"exclusiveMinimum": 5}))]
    #[test_case(&json!({"format": "ipv4"}))]
    #[test_case(&json!({"maximum": 2}))]
    #[test_case(&json!({"maxItems": 2}))]
    #[test_case(&json!({"maxProperties": 2}))]
    #[test_case(&json!({"minProperties": 2}))]
    #[test_case(&json!({"multipleOf": 2.5}))]
    #[test_case(&json!({"multipleOf": 2}))]
    #[test_case(&json!({"required": ["a"]}))]
    #[test_case(&json!({"pattern": "^a"}))]
    #[test_case(&json!({"patternProperties": {"f.*o": {"type": "integer"}}}))]
    #[test_case(&json!({"propertyNames": {"maxLength": 3}}))]
    fn is_valid_another_type(schema: &Value) {
        let instance = json!(null);
        assert!(crate::is_valid(schema, &instance));
    }
    #[test_case(&json!({"additionalProperties": false}), &json!({}))]
    #[test_case(&json!({"additionalItems": false, "items": true}), &json!([]))]
    fn is_valid(schema: &Value, instance: &Value) {
        assert!(crate::is_valid(schema, instance));
    }

    #[test_case(&json!({"type": "number"}), &json!(42))]
    #[test_case(&json!({"type": ["number", "null"]}), &json!(42))]
    fn integer_is_valid_number_multi_type(schema: &Value, instance: &Value) {
        // See: GH-147
        assert!(crate::is_valid(schema, instance));
    }
    // enum: Number
    #[test_case(&json!({"enum": [0.0]}), &json!(0))]
    // enum: Array
    #[test_case(&json!({"enum": [[1.0]]}), &json!([1]))]
    // enum: Object
    #[test_case(&json!({"enum": [{"a": 1.0}]}), &json!({"a": 1}))]
    // enum:: Object in Array
    #[test_case(&json!({"enum": [[{"b": 1.0}]]}), &json!([{"b": 1}]))]
    // enum:: Array in Object
    #[test_case(&json!({"enum": [{"c": [1.0]}]}), &json!({"c": [1]}))]
    // const: Number
    #[test_case(&json!({"const": 0.0}), &json!(0))]
    // const: Array
    #[test_case(&json!({"const": [1.0]}), &json!([1]))]
    // const: Object
    #[test_case(&json!({"const": {"a": 1.0}}), &json!({"a": 1}))]
    // const:: Object in Array
    #[test_case(&json!({"const": [{"b": 1.0}]}), &json!([{"b": 1}]))]
    // const:: Array in Object
    #[test_case(&json!({"const": {"c": [1.0]}}), &json!({"c": [1]}))]
    fn numeric_equivalence(schema: &Value, instance: &Value) {
        // See: GH-149
        assert!(crate::is_valid(schema, instance));
    }

    #[test]
    fn required_all_properties() {
        // See: GH-190
        let schema = json!({"required": ["foo", "bar"]});
        let instance = json!({});
        let validator = crate::validator_for(&schema).unwrap();
        let errors: Vec<_> = validator
            .validate(&instance)
            .expect_err("Validation errors")
            .collect();
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].to_string(), r#""foo" is a required property"#);
        assert_eq!(errors[1].to_string(), r#""bar" is a required property"#);
    }
}
