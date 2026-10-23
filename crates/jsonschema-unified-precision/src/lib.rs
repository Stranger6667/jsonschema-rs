#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;
    use serde_json::Value;
    use test_case::test_case;

    fn parse(text: &str) -> Value {
        serde_json::from_str(text).expect("valid JSON")
    }

    #[test_case(r#"{"const": 1e400}"#, "1", false)]
    #[test_case(r#"{"enum": [1e400]}"#, "1", false)]
    #[test_case(r#"{"minimum": 1e400}"#, "1", false)]
    #[test_case(r#"{"minimum": -1e400}"#, "1", true)]
    #[test_case(r#"{"maximum": 1e400}"#, "1", true)]
    #[test_case(r#"{"maximum": -1e400}"#, "1", false)]
    #[test_case(r#"{"exclusiveMinimum": 1e400}"#, "1", false)]
    #[test_case(r#"{"exclusiveMaximum": -1e400}"#, "1", false)]
    #[test_case(r#"{"type": "integer"}"#, &format!("1{}", "0".repeat(400)), true)]
    #[test_case(r#"{"type": "integer"}"#, &format!("-1{}.000", "0".repeat(400)), true)]
    #[test_case(r#"{"type": "integer"}"#, &format!("1{}.5", "0".repeat(400)), false)]
    fn number_past_f64(schema: &str, instance: &str, expected: bool) {
        let validator = jsonschema::validator_for(&parse(schema)).expect("valid schema");
        let instance = parse(instance);
        assert_eq!(validator.is_valid(&instance), expected);
        assert_eq!(validator.validate(&instance).is_ok(), expected);
        assert_eq!(validator.iter_errors(&instance).next().is_none(), expected);
        assert_eq!(validator.evaluate(&instance).flag().valid, expected);
    }

    #[test_case(
        r##"schema = r#"{"minimum": 1e400}"#"##,
        "Invalid JSON: number out of range at /minimum"
    )]
    #[test_case(
        r##"schema = r#"{"exclusiveMaximum": -1e400}"#"##,
        "Invalid JSON: number out of range at /exclusiveMaximum"
    )]
    #[test_case(
        r##"schema = r#"{"multipleOf": 1e400}"#"##,
        "Invalid JSON: number out of range at /multipleOf"
    )]
    #[test_case(
        r##"schema = r#"{"const": 1e400}"#"##,
        "Invalid JSON: number out of range at /const"
    )]
    #[test_case(
        r##"schema = r#"{"enum": [1, 1e400]}"#"##,
        "Invalid JSON: number out of range at /enum/1"
    )]
    #[test_case(
        r##"schema = r#"{"$ref": "json-schema:///a"}"#, resources = { "json-schema:///a" => { schema = r#"{"maximum": 1e400}"# } }"##,
        "Invalid JSON for resource 'json-schema:///a': number out of range at /maximum"
    )]
    fn macro_rejects_number_past_f64(attr: &str, message: &str) {
        let attr: TokenStream = attr.parse().expect("valid tokens");
        let expanded = jsonschema_macros_core::expand(
            attr,
            quote!(
                struct Validator;
            ),
        );
        assert_eq!(
            expanded.to_string(),
            quote!(::core::compile_error! { #message }).to_string()
        );
    }
}
