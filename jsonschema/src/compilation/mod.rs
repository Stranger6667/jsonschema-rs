//! Schema compilation.
//! The main idea is to compile the input JSON Schema to a validators tree that will contain
//! everything needed to perform such validation in runtime.
pub(crate) mod context;
pub(crate) mod options;

use crate::{
    error::ErrorIterator,
    keywords::{self, custom_keyword::compile_custom_keyword_validator},
    output::Output,
    paths::{JSONPointer, JsonPointerNode},
    primitive_type::{PrimitiveType, PrimitiveTypesBitMap},
    schema_node::SchemaNode,
    validator::Validate,
    Draft, ValidationError,
};
use ahash::AHashMap;
use context::CompilationContext;
use once_cell::sync::Lazy;
use options::CompilationOptions;
use serde_json::Value;
use std::sync::Arc;
use url::Url;

pub(crate) const DEFAULT_ROOT_URL: &str = "json-schema:///";

/// The structure that holds a JSON Schema compiled into a validation tree
#[derive(Debug)]
pub struct JSONSchema {
    pub(crate) node: SchemaNode,
    config: Arc<CompilationOptions>,
}

pub(crate) static DEFAULT_SCOPE: Lazy<Url> =
    Lazy::new(|| url::Url::parse(DEFAULT_ROOT_URL).expect("Is a valid URL"));

impl JSONSchema {
    /// Return a default `CompilationOptions` that can configure
    /// `JSONSchema` compilation flow.
    ///
    /// Using options you will be able to configure the draft version
    /// to use during `JSONSchema` compilation
    ///
    /// Example of usage:
    /// ```rust
    /// # use crate::jsonschema::{Draft, JSONSchema};
    /// # let schema = serde_json::json!({});
    /// let maybe_jsonschema: Result<JSONSchema, _> = JSONSchema::options()
    ///     .with_draft(Draft::Draft7)
    ///     .compile(&schema);
    /// ```
    #[must_use]
    pub fn options() -> CompilationOptions {
        CompilationOptions::default()
    }

    /// Compile the input schema into a validation tree.
    ///
    /// The method is equivalent to `JSONSchema::options().compile(schema)`
    pub fn compile(schema: &Value) -> Result<JSONSchema, ValidationError> {
        Self::options().compile(schema)
    }

    /// Run validation against `instance` and return an iterator over `ValidationError` in the error case.
    #[inline]
    pub fn validate<'instance>(
        &'instance self,
        instance: &'instance Value,
    ) -> Result<(), ErrorIterator<'instance>> {
        let instance_path = JsonPointerNode::new();
        let mut errors = self.node.validate(instance, &instance_path).peekable();
        if errors.peek().is_none() {
            Ok(())
        } else {
            Err(Box::new(errors))
        }
    }

    /// Run validation against `instance` but return a boolean result instead of an iterator.
    /// It is useful for cases, where it is important to only know the fact if the data is valid or not.
    /// This approach is much faster, than `validate`.
    #[must_use]
    #[inline]
    pub fn is_valid(&self, instance: &Value) -> bool {
        self.node.is_valid(instance)
    }

    /// Apply the schema and return an `Output`. No actual work is done at this point, the
    /// evaluation of the schema is deferred until a method is called on the `Output`. This is
    /// because different output formats will have different performance characteristics.
    ///
    /// # Examples
    ///
    /// "basic" output format
    ///
    /// ```rust
    /// # use crate::jsonschema::{Draft, JSONSchema, output::{Output, BasicOutput}};
    /// let schema_json = serde_json::json!({
    ///     "title": "string value",
    ///     "type": "string"
    /// });
    /// let instance = serde_json::json!{"some string"};
    /// let schema = JSONSchema::options().compile(&schema_json).unwrap();
    /// let output: BasicOutput = schema.apply(&instance).basic();
    /// let output_json = serde_json::to_value(output).unwrap();
    /// assert_eq!(output_json, serde_json::json!({
    ///     "valid": true,
    ///     "annotations": [
    ///         {
    ///             "keywordLocation": "",
    ///             "instanceLocation": "",
    ///             "annotations": {
    ///                 "title": "string value"
    ///             }
    ///         }
    ///     ]
    /// }));
    /// ```
    #[must_use]
    pub const fn apply<'a, 'b>(&'a self, instance: &'b Value) -> Output<'a, 'b> {
        Output::new(self, &self.node, instance)
    }

    /// The [`Draft`] which this schema was compiled against
    #[must_use]
    pub fn draft(&self) -> Draft {
        self.config.draft()
    }

    /// The [`CompilationOptions`] that were used to compile this schema
    #[must_use]
    pub fn config(&self) -> Arc<CompilationOptions> {
        Arc::clone(&self.config)
    }
}

/// Compile JSON schema into a tree of validators.
#[inline]
pub(crate) fn compile_validators<'a>(
    schema: &'a Value,
    context: &CompilationContext,
) -> Result<SchemaNode, ValidationError<'a>> {
    let context = context.push(schema)?;
    let relative_path = context.clone().into_pointer();
    match schema {
        Value::Bool(value) => match value {
            true => Ok(SchemaNode::new_from_boolean(&context, None)),
            false => Ok(SchemaNode::new_from_boolean(
                &context,
                Some(
                    keywords::boolean::FalseValidator::compile(relative_path)
                        .expect("Should always compile"),
                ),
            )),
        },
        Value::Object(object) => {
            // In Draft 2019-09 and later, `$ref` can be evaluated alongside other attribute aka
            // adjacent validation. We check here to see if adjacent validation is supported, and if
            // so, we use the normal keyword validator collection logic.
            //
            // Otherwise, we isolate `$ref` and generate a schema reference validator directly.
            let maybe_reference = object
                .get("$ref")
                .filter(|_| !keywords::ref_::supports_adjacent_validation(context.config.draft()));
            if let Some(reference) = maybe_reference {
                let unmatched_keywords = object
                    .iter()
                    .filter_map(|(k, v)| {
                        if k.as_str() == "$ref" {
                            None
                        } else {
                            Some((k.clone(), v.clone()))
                        }
                    })
                    .collect();

                let validator = keywords::ref_::compile(object, reference, &context)
                    .expect("should always return Some")?;

                let validators = vec![("$ref".to_string(), validator)];
                Ok(SchemaNode::new_from_keywords(
                    &context,
                    validators,
                    Some(unmatched_keywords),
                ))
            } else {
                let mut validators = Vec::with_capacity(object.len());
                let mut unmatched_keywords = AHashMap::new();
                let mut is_if = false;
                let mut is_props = false;
                for (keyword, subschema) in object {
                    if keyword == "if" {
                        is_if = true;
                    }
                    if keyword == "properties"
                        || keyword == "additionalProperties"
                        || keyword == "patternProperties"
                    {
                        is_props = true;
                    }
                    // first check if this keyword was added as a custom keyword
                    // it may override existing keyword behavior
                    if let Some(keyword_definition) =
                        context.config.get_custom_keyword_definition(keyword)
                    {
                        let validator = compile_custom_keyword_validator(
                            &context,
                            keyword.clone(),
                            keyword_definition,
                            subschema.clone(),
                            schema.clone(),
                        )?;
                        validators.push((keyword.clone(), validator));
                    } else if let Some(validator) = context
                        .config
                        .draft()
                        .get_validator(keyword)
                        .and_then(|f| f(object, subschema, &context))
                    {
                        validators.push((keyword.clone(), validator?));
                    } else {
                        unmatched_keywords.insert(keyword.to_string(), subschema.clone());
                    }
                }
                if is_if {
                    unmatched_keywords.remove("then");
                    unmatched_keywords.remove("else");
                }
                if is_props {
                    unmatched_keywords.remove("additionalProperties");
                    unmatched_keywords.remove("patternProperties");
                    unmatched_keywords.remove("properties");
                }
                let unmatched_keywords = if unmatched_keywords.is_empty() {
                    None
                } else {
                    Some(unmatched_keywords)
                };
                Ok(SchemaNode::new_from_keywords(
                    &context,
                    validators,
                    unmatched_keywords,
                ))
            }
        }
        _ => Err(ValidationError::multiple_type_error(
            JSONPointer::default(),
            relative_path,
            schema,
            PrimitiveTypesBitMap::new()
                .add_type(PrimitiveType::Boolean)
                .add_type(PrimitiveType::Object),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::JSONSchema;
    use crate::{
        compilation::options::CustomKeywordDefinition, error::ValidationError, paths::JSONPointer,
        ErrorIterator,
    };
    use num_cmp::NumCmp;
    use regex::Regex;
    use serde_json::{from_str, json, Value};
    use std::{borrow::Cow, fs::File, io::Read, path::Path, sync::Arc};

    fn load(path: &str, idx: usize) -> Value {
        let path = Path::new(path);
        let mut file = File::open(path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).ok().unwrap();
        let data: Value = from_str(&content).unwrap();
        let case = &data.as_array().unwrap()[idx];
        case.get("schema").unwrap().clone()
    }

    #[test]
    fn only_keyword() {
        // When only one keyword is specified
        let schema = json!({"type": "string"});
        let compiled = JSONSchema::compile(&schema).unwrap();
        let value1 = json!("AB");
        let value2 = json!(1);
        // And only this validator
        assert_eq!(compiled.node.validators().len(), 1);
        assert!(compiled.validate(&value1).is_ok());
        assert!(compiled.validate(&value2).is_err());
    }

    #[test]
    fn validate_ref() {
        let schema = load("tests/suite/tests/draft7/ref.json", 1);
        let value = json!({"bar": 3});
        let compiled = JSONSchema::compile(&schema).unwrap();
        assert!(compiled.validate(&value).is_ok());
        let value = json!({"bar": true});
        assert!(compiled.validate(&value).is_err());
    }

    #[test]
    fn wrong_schema_type() {
        let schema = json!([1]);
        let compiled = JSONSchema::compile(&schema);
        assert!(compiled.is_err());
    }

    #[test]
    fn multiple_errors() {
        let schema = json!({"minProperties": 2, "propertyNames": {"minLength": 3}});
        let value = json!({"a": 3});
        let compiled = JSONSchema::compile(&schema).unwrap();
        let result = compiled.validate(&value);
        let errors: Vec<ValidationError> = result.unwrap_err().collect();
        assert_eq!(errors.len(), 2);
        assert_eq!(
            errors[0].to_string(),
            r#"{"a":3} has less than 2 properties"#
        );
        assert_eq!(errors[1].to_string(), r#""a" is shorter than 3 characters"#);
    }

    #[test]
    fn custom_keyword_definition() {
        // Define a custom validator that verifies the object's keys consist of
        // only ASCII representable characters.
        #[allow(clippy::needless_pass_by_value)]
        fn custom_object_validator(
            instance: &Value,
            instance_path: JSONPointer,
            schema: Arc<Value>,
            schema_path: JSONPointer,
            _: Arc<Value>,
        ) -> ErrorIterator<'_> {
            if schema.as_str().map_or(true, |str| str != "ascii-keys") {
                let error = ValidationError {
                    instance: Cow::Borrowed(instance),
                    kind: crate::error::ValidationErrorKind::Schema,
                    instance_path,
                    schema_path,
                };
                return Box::new(Some(error).into_iter()); // Invalid schema
            }
            let mut errors = vec![];
            for (key, _value) in instance.as_object().unwrap() {
                if !key.is_ascii() {
                    let error = ValidationError {
                        instance: Cow::Borrowed(instance),
                        kind: crate::error::ValidationErrorKind::Format { format: "ASCII" },
                        instance_path: instance_path.clone(),
                        schema_path: schema_path.clone(),
                    };
                    errors.push(error);
                }
            }
            Box::new(errors.into_iter())
        }
        fn is_custom_object_valid(instance: &Value, schema: &Value, _: &Value) -> bool {
            if schema.as_str().map_or(true, |str| str != "ascii-keys") {
                return false; // Invalid schema
            }
            for (key, _value) in instance.as_object().unwrap() {
                if !key.is_ascii() {
                    return false;
                }
            }
            true
        }
        let definition = CustomKeywordDefinition::Validator {
            validate: custom_object_validator,
            is_valid: is_custom_object_valid,
        };

        // Define a JSON schema that enforces the top level object has ASCII keys and has at least 1 property
        let schema =
            json!({ "custom-object-type": "ascii-keys", "type": "object", "minProperties": 1 });
        let json_schema = JSONSchema::options()
            .with_custom_keyword("custom-object-type", definition)
            .compile(&schema)
            .unwrap();

        // Verify schema validation detects object with too few properties
        let instance_err_not_object = json!({});
        assert!(json_schema.validate(&instance_err_not_object).is_err());
        assert!(!json_schema.is_valid(&instance_err_not_object));

        // Verify validator succeeds on a valid custom-object-type
        let instance_ok = json!({ "a" : 1 });
        assert!(json_schema.validate(&instance_ok).is_ok());
        assert!(json_schema.is_valid(&instance_ok));

        // Verify validator detects invalid custom-object-type
        let instance_err_non_ascii_keys = json!({ "å" : 1 });
        assert!(json_schema.validate(&instance_err_non_ascii_keys).is_err());
        assert!(!json_schema.is_valid(&instance_err_non_ascii_keys));
    }

    #[test]
    fn custom_format_and_override_keyword() {
        // prepare a custom format checker
        // in this case, the format is "currency"
        // checks that a string has some number of digits followed by a dot followed by
        // exactly 2 digits.
        const CURRENCY_RE_STR: &str = "^(0|([1-9]+[0-9]*))(\\.[0-9]{2})$";
        fn currency_format_checker(s: &str) -> bool {
            Regex::new(CURRENCY_RE_STR).unwrap().is_match(s)
        }
        // Define a custom keyword validator that overrides "minimum"
        // so that "minimum" may apply to "currency"-formatted strings as well
        fn custom_minimum_validator(
            instance: &Value,
            instance_path: JSONPointer,
            subschema: Arc<Value>,
            subschema_path: JSONPointer,
            schema: Arc<Value>,
        ) -> ErrorIterator<'_> {
            let subschema: &Value = &subschema;
            let limit = match subschema {
                Value::Number(limit) => limit,
                _ => {
                    let error = ValidationError {
                        instance: Cow::Borrowed(instance),
                        kind: crate::error::ValidationErrorKind::Schema,
                        instance_path,
                        schema_path: subschema_path,
                    };
                    return Box::new(Some(error).into_iter()); // Invalid schema
                }
            };
            let mut errors = vec![];
            let valid = match instance {
                // numeric comparison should happen just like original behavior
                Value::Number(instance) => {
                    if let Some(item) = instance.as_u64() {
                        !NumCmp::num_lt(item, limit.as_f64().unwrap())
                    } else if let Some(item) = limit.as_i64() {
                        !NumCmp::num_lt(item, limit.as_f64().unwrap())
                    } else {
                        let item = instance.as_f64().expect("Always valid");
                        !NumCmp::num_lt(item, limit.as_f64().unwrap())
                    }
                }
                // string comparison should cast currency-formatted
                Value::String(instance) => {
                    let mut valid = true;
                    if let Some(schema) = schema.as_object() {
                        if let Some(format) = schema.get("format") {
                            if format == "currency" {
                                if currency_format_checker(instance) {
                                    // all preconditions for minimum applying are met
                                    let as_f64 = instance
                                        .parse::<f64>()
                                        .expect("format validated by regex checker");
                                    println!("1 {:#?} {:#?}", as_f64, limit.as_f64().unwrap());
                                    valid = !NumCmp::num_lt(as_f64, limit.as_f64().unwrap());
                                    println!("valid {:#?}", valid);
                                }
                            }
                        }
                    }
                    valid
                }
                // in all other cases, the "minimum" keyword should not apply
                _ => true,
            };
            if !valid {
                let error = ValidationError {
                    instance: Cow::Borrowed(instance),
                    kind: crate::error::ValidationErrorKind::Minimum {
                        limit: subschema.clone(),
                    },
                    instance_path: instance_path.clone(),
                    schema_path: subschema_path.clone(),
                };
                errors.push(error);
            }
            Box::new(errors.into_iter())
        }
        fn is_custom_minimum_valid(instance: &Value, subschema: &Value, schema: &Value) -> bool {
            let subschema: &Value = &subschema;
            let limit = match subschema {
                Value::Number(limit) => limit,
                _ => return false,
            };
            let valid = match instance {
                // numeric comparison should happen just like original behavior
                Value::Number(instance) => {
                    if let Some(item) = instance.as_u64() {
                        !NumCmp::num_lt(item, limit.as_f64().unwrap())
                    } else if let Some(item) = limit.as_i64() {
                        !NumCmp::num_lt(item, limit.as_f64().unwrap())
                    } else {
                        let item = instance.as_f64().expect("Always valid");
                        !NumCmp::num_lt(item, limit.as_f64().unwrap())
                    }
                }
                // string comparison should cast currency-formatted
                Value::String(instance) => {
                    let mut valid = true;
                    if let Some(schema) = schema.as_object() {
                        if let Some(format) = schema.get("format") {
                            if format == "currency" {
                                if currency_format_checker(instance) {
                                    // all preconditions for minimum applying are met
                                    let as_f64 = instance
                                        .parse::<f64>()
                                        .expect("format validated by regex checker");
                                    println!("1 {:#?} {:#?}", as_f64, limit.as_f64().unwrap());
                                    valid = !NumCmp::num_lt(as_f64, limit.as_f64().unwrap());
                                    println!("valid {:#?}", valid);
                                }
                            }
                        }
                    }
                    valid
                }
                // in all other cases, the "minimum" keyword should not apply
                _ => true,
            };
            return valid;
        }
        let definition = CustomKeywordDefinition::Validator {
            validate: custom_minimum_validator,
            is_valid: is_custom_minimum_valid,
        };

        // define compilation options that include the custom format and the overridden keyword
        let mut options = JSONSchema::options();
        let options = options
            .with_format("currency", currency_format_checker)
            .with_custom_keyword("minimum", definition);

        // Define a schema that includes both the custom format and the overridden keyword
        let schema = json!({ "minimum": 2, "type": "string", "format": "currency" });
        let compiled = options.compile(&schema).unwrap();

        // Control: verify schema validation rejects non-string types
        let instance_err_not_string = json!(15);
        assert!(compiled.validate(&instance_err_not_string).is_err());
        assert!(!compiled.is_valid(&instance_err_not_string));

        // Control: verify validator rejects ill-formatted strings
        let instance_ok = json!("not a currency");
        assert!(compiled.validate(&instance_ok).is_err());
        assert!(!compiled.is_valid(&instance_ok));

        // Verify validator allows properly formatted strings that conform to custom keyword
        let instance_err_non_ascii_keys = json!("3.00");
        assert!(compiled.validate(&instance_err_non_ascii_keys).is_ok());
        assert!(compiled.is_valid(&instance_err_non_ascii_keys));

        // Verify validator rejects properly formatted strings that do not conform to custom keyword
        let instance_err_non_ascii_keys = json!("1.99");
        assert!(compiled.validate(&instance_err_non_ascii_keys).is_err());
        assert!(!compiled.is_valid(&instance_err_non_ascii_keys));

        // Define another schema that applies "minimum" to an integer to ensure original behavior
        let schema = json!({ "minimum": 2, "type": "integer" });
        let compiled = options.compile(&schema).unwrap();

        // Verify schema allows integers greater than 2
        let instance_err_not_string = json!(3);
        assert!(compiled.validate(&instance_err_not_string).is_ok());
        assert!(compiled.is_valid(&instance_err_not_string));

        // Verify schema rejects integers less than 2
        let instance_ok = json!(1);
        assert!(compiled.validate(&instance_ok).is_err());
        assert!(!compiled.is_valid(&instance_ok));
    }
}
