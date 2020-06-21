pub mod additional_items;
pub mod additional_properties;
pub mod all_of;
pub mod any_of;
pub mod boolean;
pub mod const_;
pub mod contains;
pub mod content;
pub mod dependencies;
pub mod enum_;
pub mod exclusive_maximum;
pub mod exclusive_minimum;
pub mod format;
pub mod if_;
pub mod items;
pub mod legacy;
pub mod max_items;
pub mod max_length;
pub mod max_properties;
pub mod maximum;
pub mod min_items;
pub mod min_length;
pub mod min_properties;
pub mod minimum;
pub mod multiple_of;
pub mod not;
pub mod one_of;
pub mod pattern;
pub mod pattern_properties;
pub mod properties;
pub mod property_names;
pub mod ref_;
pub mod required;
pub mod type_;
pub mod unique_items;
use crate::{error, validator::Validate};

pub type CompilationResult = Result<BoxedValidator, error::CompilationError>;
pub type BoxedValidator = Box<dyn Validate + Send + Sync>;
pub type Validators = Vec<BoxedValidator>;

fn format_validators(validators: &[BoxedValidator]) -> String {
    match validators.len() {
        0 => "{}".to_string(),
        1 => {
            let name = validators[0].name();
            match name.as_str() {
                // boolean validators are represented as is, without brackets because if they
                // occur in a vector, then the schema is not a key/value mapping
                "true" | "false" => name,
                _ => format!("{{{}}}", name),
            }
        }
        _ => format!(
            "{{{}}}",
            validators
                .iter()
                .map(|validator| format!("{:?}", validator))
                .collect::<Vec<String>>()
                .join(", ")
        ),
    }
}

fn format_vec_of_validators(validators: &[Validators]) -> String {
    validators
        .iter()
        .map(|v| format_validators(v))
        .collect::<Vec<String>>()
        .join(", ")
}

fn format_key_value_validators(validators: &[(String, Validators)]) -> String {
    validators
        .iter()
        .map(|(name, validators)| format!("{}: {}", name, format_validators(validators)))
        .collect::<Vec<String>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use crate::compilation::JSONSchema;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!({"additionalItems": false, "items": [{"type": "string"}]}), "additionalItems: false")]
    #[test_case(json!({"additionalItems": {"type": "integer"}, "items": [{"type": "string"}]}), "additionalItems: {type: integer}")]
    #[test_case(json!({"additionalProperties": {"type": "string"}}), "additionalProperties: {type: string}")]
    #[test_case(json!({"additionalProperties": {"type": "string"}, "properties": {"foo": {}}}), "additionalProperties: {type: string}")]
    #[test_case(json!({"additionalProperties": {"type": "string"}, "patternProperties": {"f.*o": {"type": "integer"}}}), "additionalProperties: {type: string}")]
    #[test_case(json!({"additionalProperties": {"type": "string"}, "properties": {"foo": {}}, "patternProperties": {"f.*o": {"type": "integer"}}}), "additionalProperties: {type: string}")]
    #[test_case(json!({"additionalProperties": false}), "additionalProperties: false")]
    #[test_case(json!({"additionalProperties": false, "properties": {"foo": {}}}), "additionalProperties: false")]
    #[test_case(json!({"additionalProperties": false, "patternProperties": {"f.*o": {"type": "integer"}}}), "additionalProperties: false")]
    #[test_case(json!({"additionalProperties": false, "properties": {"foo": {}}, "patternProperties": {"f.*o": {"type": "integer"}}}), "additionalProperties: false")]
    #[test_case(json!({"allOf": [{"type": "integer"}, {"minimum": 2}]}), "allOf: [{type: integer}, {minimum: 2}]")]
    #[test_case(json!({"anyOf": [{"type": "integer"}, {"minimum": 2}]}), "anyOf: [{type: integer}, {minimum: 2}]")]
    #[test_case(json!(true), "true")]
    #[test_case(json!(false), "false")]
    #[test_case(json!({"const": 1}), "const: 1")]
    #[test_case(json!({"contains": {"minimum": 5}}), "contains: {minimum: 5}")]
    #[test_case(json!({"contentMediaType": "application/json"}), "contentMediaType: application/json")]
    #[test_case(json!({"contentEncoding": "base64"}), "contentEncoding: base64")]
    #[test_case(json!({"contentEncoding": "base64", "contentMediaType": "application/json"}), "{contentMediaType: application/json, contentEncoding: base64}")]
    #[test_case(json!({"dependencies": {"bar": ["foo"]}}), "dependencies: {bar: {required: [foo]}}")]
    #[test_case(json!({"enum": [1]}), "enum: [1]")]
    #[test_case(json!({"exclusiveMaximum": 1}), "exclusiveMaximum: 1")]
    #[test_case(json!({"exclusiveMinimum": 1}), "exclusiveMinimum: 1")]
    #[test_case(json!({"format": "date"}), "format: date")]
    #[test_case(json!({"format": "date-time"}), "format: date-time")]
    #[test_case(json!({"format": "email"}), "format: email")]
    #[test_case(json!({"format": "hostname"}), "format: hostname")]
    #[test_case(json!({"format": "idn-email"}), "format: idn-email")]
    #[test_case(json!({"format": "idn-hostname"}), "format: idn-hostname")]
    #[test_case(json!({"format": "ipv4"}), "format: ipv4")]
    #[test_case(json!({"format": "ipv6"}), "format: ipv6")]
    #[test_case(json!({"format": "iri"}), "format: iri")]
    #[test_case(json!({"format": "iri-reference"}), "format: iri-reference")]
    #[test_case(json!({"format": "json-pointer"}), "format: json-pointer")]
    #[test_case(json!({"format": "regex"}), "format: regex")]
    #[test_case(json!({"format": "relative-json-pointer"}), "format: relative-json-pointer")]
    #[test_case(json!({"format": "time"}), "format: time")]
    #[test_case(json!({"format": "uri"}), "format: uri")]
    #[test_case(json!({"format": "uri-reference"}), "format: uri-reference")]
    #[test_case(json!({"format": "uri-template"}), "format: uri-template")]
    #[test_case(json!({"if": {"exclusiveMaximum": 0}, "then": {"minimum": -10}}), "if: {exclusiveMaximum: 0}, then: {minimum: -10}")]
    #[test_case(json!({"if": {"exclusiveMaximum": 0}, "else": {"minimum": -10}}), "if: {exclusiveMaximum: 0}, else: {minimum: -10}")]
    #[test_case(json!({"if": {"exclusiveMaximum": 0}, "then": {"minimum": -10}, "else": {"multipleOf": 2}}), "if: {exclusiveMaximum: 0}, then: {minimum: -10}, else: {multipleOf: 2}")]
    #[test_case(json!({"items": [{"type": "string"}]}), "items: [{type: string}]")]
    #[test_case(json!({"items": {"type": "integer"}}), "items: {type: integer}")]
    #[test_case(json!({"items": {"type": "integer", "minimum": 4}}), "items: {minimum: 4, type: integer}")]
    #[test_case(json!({"maxItems": 1}), "maxItems: 1")]
    #[test_case(json!({"maxLength": 1}), "maxLength: 1")]
    #[test_case(json!({"maxProperties": 1}), "maxProperties: 1")]
    #[test_case(json!({"maximum": 1}), "maximum: 1")]
    #[test_case(json!({"minItems": 1}), "minItems: 1")]
    #[test_case(json!({"minLength": 1}), "minLength: 1")]
    #[test_case(json!({"minProperties": 1}), "minProperties: 1")]
    #[test_case(json!({"minimum": 1}), "minimum: 1")]
    #[test_case(json!({"multipleOf": 1}), "multipleOf: 1")]
    #[test_case(json!({"multipleOf": 1.5}), "multipleOf: 1.5")]
    #[test_case(json!({"not": true}), "not: true")]
    #[test_case(json!({"oneOf": [{"type": "integer"}, {"minimum": 2}]}), "oneOf: [{type: integer}, {minimum: 2}]")]
    #[test_case(json!({"pattern": "^a*$"}), "pattern: ^a*$")]
    #[test_case(json!({"patternProperties": {"f.*o": {"type": "integer"}}}), "patternProperties: {f.*o: {type: integer}}")]
    #[test_case(json!({"properties": {"foo": {}}}), "properties: {foo: {}}")]
    #[test_case(json!({"propertyNames": {"maxLength": 3}}), "propertyNames: {maxLength: 3}")]
    #[test_case(json!({"propertyNames": false}), "propertyNames: false")]
    #[test_case(json!({"$ref": "#/properties/foo"}), "$ref: json-schema:///#/properties/foo")]
    #[test_case(json!({"required": ["foo"]}), "required: [foo]")]
    #[test_case(json!({"type": "null"}), "type: null")]
    #[test_case(json!({"type": "boolean"}), "type: boolean")]
    #[test_case(json!({"type": "string"}), "type: string")]
    #[test_case(json!({"type": "array"}), "type: array")]
    #[test_case(json!({"type": "object"}), "type: object")]
    #[test_case(json!({"type": "number"}), "type: number")]
    #[test_case(json!({"type": "integer"}), "type: integer")]
    #[test_case(json!({"type": "integer", "$schema": "http://json-schema.org/draft-04/schema#"}), "type: integer")]
    #[test_case(json!({"type": ["integer", "null"]}), "type: [integer, null]")]
    #[test_case(json!({"type": ["integer", "null"], "$schema": "http://json-schema.org/draft-04/schema#"}), "type: [integer, null]")]
    #[test_case(json!({"uniqueItems": true}), "uniqueItems: true")]
    fn debug_representation(schema: Value, expected: &str) {
        let compiled = JSONSchema::compile(&schema, None).unwrap();
        assert_eq!(format!("{:?}", compiled.validators[0]), expected);
    }

    #[test_case(json!({"items": [{}], "additionalItems": {"type": "integer"}}), json!([ null, 2, 3, "foo" ]), r#"'"foo"' is not of type 'integer'"#)]
    #[test_case(json!({"items": [{}, {}, {}], "additionalItems": false}), json!([ 1, 2, 3, 4 ]), r#"Additional items are not allowed (4 was unexpected)"#)]
    #[test_case(json!({"items": [{}, {}, {}], "additionalItems": false}), json!([ 1, 2, 3, 4, 5 ]), r#"Additional items are not allowed (4, 5 were unexpected)"#)]
    #[test_case(json!({"properties": {"foo": {}, "bar": {}}, "patternProperties": { "^v": {} }, "additionalProperties": false}), json!({"foo" : 1, "bar" : 2, "quux" : "boom"}), r#"False schema does not allow '"quux"'"#)]
    #[test_case(json!({"anyOf": [{"type": "integer"}, {"minimum": 2}]}), json!(1.5), r#"'1.5' is not valid under any of the given schemas"#)]
    #[test_case(json!({"const": 2}), json!(5), r#"'2' was expected"#)]
    #[test_case(json!({"contains": {"minimum": 5}}), json!([2, 3, 4]), r#"None of '[2,3,4]' are valid under the given schema"#)]
    #[test_case(json!({"enum": [1, 2, 3]}), json!(4), r#"'4' is not one of '[1,2,3]'"#)]
    #[test_case(json!({"exclusiveMaximum": 3.0}), json!(3.0), r#"3.0 is greater than or equal to the maximum of 3"#)]
    #[test_case(json!({"exclusiveMinimum": 1.1}), json!(1.1), r#"1.1 is less than or equal to the minimum of 1.1"#)]
    #[test_case(json!({"format": "ipv4"}), json!("2001:0db8:85a3:0000:0000:8a2e:0370:7334"), r#"'"2001:0db8:85a3:0000:0000:8a2e:0370:7334"' is not a 'ipv4'"#)]
    #[test_case(json!({"maximum": 3.0}), json!(3.5), r#"3.5 is greater than the maximum of 3"#)]
    #[test_case(json!({"maxItems": 2}), json!([1, 2, 3]), r#"[1,2,3] has more than 2 items"#)]
    #[test_case(json!({"maxLength": 2}), json!("foo"), r#"'"foo"' is longer than 2 characters"#)]
    #[test_case(json!({"maxProperties": 2}), json!({"foo": 1, "bar": 2, "baz": 3}), r#"{"bar":2,"baz":3,"foo":1} has more than 2 properties"#)]
    #[test_case(json!({"minimum": 1.1}), json!(0.6), r#"0.6 is less than the minimum of 1.1"#)]
    #[test_case(json!({"minItems": 1}), json!([]), r#"[] has less than 1 item"#)]
    #[test_case(json!({"minLength": 2}), json!("f"), r#"'"f"' is shorter than 2 characters"#)]
    #[test_case(json!({"minProperties": 1}), json!({}), r#"{} has less than 1 property"#)]
    #[test_case(json!({"multipleOf": 2}), json!(7), r#"7 is not a multiple of 2"#)]
    #[test_case(json!({"not": {"type": "integer"}}), json!(1), r#"{"type":"integer"} is not allowed for 1"#)]
    #[test_case(json!({"oneOf": [{"type": "integer"}, {"minimum": 2}]}), json!(1.1), r#"'1.1' is not valid under any of the given schemas"#)]
    #[test_case(json!({"oneOf": [{"type": "integer"}, {"minimum": 2}]}), json!(3), r#"'3' is valid under more than one of the given schemas"#)]
    #[test_case(json!({"pattern": "^a*$"}), json!("abc"), r#"'"abc"' does not match '^a*$'"#)]
    #[test_case(json!({"properties": {"foo": {}, "bar": {}}, "required": ["foo"]}), json!({"bar": 1}), r#"'foo' is a required property"#)]
    #[test_case(json!({"type": "integer"}), json!(1.1), r#"'1.1' is not of type 'integer'"#)]
    #[test_case(json!({"type": ["integer", "string"]}), json!(null), r#"'null' is not of types 'integer', 'string'"#)]
    #[test_case(json!({"uniqueItems": true}), json!([1, 1]), r#"'[1,1]' has non-unique elements"#)]
    fn error_message(schema: Value, instance: Value, expected: &str) {
        let compiled = JSONSchema::compile(&schema, None).unwrap();
        let errors: Vec<_> = compiled
            .validate(&instance)
            .expect_err(&format!(
                "Validation error is expected. Schema=`{:?}` Instance=`{:?}`",
                schema, instance
            ))
            .collect();
        assert_eq!(errors[0].to_string(), expected);
    }

    // Extra cases not covered by JSON test suite
    #[test_case(json!({"additionalProperties": {"type": "string"}}))]
    #[test_case(json!({"additionalProperties": {"type": "string"}, "properties": {"foo": {}}}))]
    #[test_case(json!({"additionalProperties": {"type": "string"}, "patternProperties": {"f.*o": {"type": "integer"}}}))]
    #[test_case(json!({"additionalProperties": {"type": "string"}, "properties": {"foo": {}}, "patternProperties": {"f.*o": {"type": "integer"}}}))]
    #[test_case(json!({"additionalProperties": false}))]
    #[test_case(json!({"additionalProperties": false, "properties": {"foo": {}}}))]
    #[test_case(json!({"additionalProperties": false, "patternProperties": {"f.*o": {"type": "integer"}}}))]
    #[test_case(json!({"additionalProperties": false, "properties": {"foo": {}}, "patternProperties": {"f.*o": {"type": "integer"}}}))]
    #[test_case(json!({"additionalItems": false, "items": [{"type": "string"}]}))]
    #[test_case(json!({"additionalItems": {"type": "integer"}, "items": [{"type": "string"}]}))]
    #[test_case(json!({"contains": {"minimum": 5}}))]
    #[test_case(json!({"contentMediaType": "application/json"}))]
    #[test_case(json!({"contentEncoding": "base64"}))]
    #[test_case(json!({"contentEncoding": "base64", "contentMediaType": "application/json"}))]
    #[test_case(json!({"dependencies": {"bar": ["foo"]}}))]
    #[test_case(json!({"exclusiveMaximum": 5}))]
    #[test_case(json!({"exclusiveMinimum": 5}))]
    #[test_case(json!({"format": "ipv4"}))]
    #[test_case(json!({"maximum": 2}))]
    #[test_case(json!({"maxItems": 2}))]
    #[test_case(json!({"maxProperties": 2}))]
    #[test_case(json!({"minProperties": 2}))]
    #[test_case(json!({"multipleOf": 2.5}))]
    #[test_case(json!({"multipleOf": 2}))]
    #[test_case(json!({"required": ["a"]}))]
    #[test_case(json!({"pattern": "^a"}))]
    #[test_case(json!({"patternProperties": {"f.*o": {"type": "integer"}}}))]
    #[test_case(json!({"propertyNames": {"maxLength": 3}}))]
    fn is_valid_another_type(schema: Value) {
        let instance = json!(null);
        let compiled = JSONSchema::compile(&schema, None).unwrap();
        assert!(compiled.is_valid(&instance))
    }

    #[test_case(json!({"additionalProperties": false}), json!({}))]
    #[test_case(json!({"additionalItems": false, "items": true}), json!([]))]
    fn is_valid(schema: Value, instance: Value) {
        let data = json!(instance);
        let compiled = JSONSchema::compile(&schema, None).unwrap();
        assert!(compiled.is_valid(&data))
    }
}
