//! Schema compilation.
//! The main idea is to compile the input JSON Schema to a validators tree that will contain
//! everything needed to perform such validation in runtime.
pub(crate) mod context;
pub(crate) mod options;

use crate::{
    error::ErrorIterator,
    keywords::{self, custom::CustomKeyword, BoxedValidator},
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
    Lazy::new(|| Url::parse(DEFAULT_ROOT_URL).expect("Is a valid URL"));

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
                    // Check if this keyword is overridden, then check the standard definitions
                    if let Some(factory) = context.config.get_keyword_factory(keyword) {
                        let path = context.as_pointer_with(keyword.as_str());
                        let validator = CustomKeyword::new(factory.init(object, subschema, path)?);
                        let validator: BoxedValidator = Box::new(validator);
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
        error::{self, no_error, ValidationError},
        keywords::custom::Keyword,
        paths::{JSONPointer, JsonPointerNode},
        primitive_type::PrimitiveType,
        ErrorIterator,
    };
    use num_cmp::NumCmp;
    use once_cell::sync::Lazy;
    use regex::Regex;
    use serde_json::{from_str, json, Map, Value};
    use std::{fs::File, io::Read, path::Path};

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
        /// Define a custom validator that verifies the object's keys consist of
        /// only ASCII representable characters.
        /// NOTE: This could be done with `propertyNames` + `pattern` but will be slower due to
        /// regex usage.
        struct CustomObjectValidator;
        impl Keyword for CustomObjectValidator {
            fn validate<'instance>(
                &self,
                instance: &'instance Value,
                instance_path: &JsonPointerNode,
            ) -> ErrorIterator<'instance> {
                let mut errors = vec![];
                for key in instance.as_object().unwrap().keys() {
                    if !key.is_ascii() {
                        let error = ValidationError::custom(
                            JSONPointer::default(),
                            instance_path.into(),
                            instance,
                            "Key is not ASCII",
                        );
                        errors.push(error);
                    }
                }
                Box::new(errors.into_iter())
            }

            fn is_valid(&self, instance: &Value) -> bool {
                for (key, _value) in instance.as_object().unwrap() {
                    if !key.is_ascii() {
                        return false;
                    }
                }
                true
            }
        }

        fn custom_object_type_factory<'a>(
            _: &'a Map<String, Value>,
            schema: &'a Value,
            path: JSONPointer,
        ) -> Result<Box<dyn Keyword>, ValidationError<'a>> {
            const EXPECTED: &str = "ascii-keys";
            if schema.as_str().map_or(true, |key| key != EXPECTED) {
                Err(ValidationError::constant_string(
                    JSONPointer::default(),
                    path,
                    schema,
                    EXPECTED,
                ))
            } else {
                Ok(Box::new(CustomObjectValidator))
            }
        }

        // Define a JSON schema that enforces the top level object has ASCII keys and has at least 1 property
        let schema =
            json!({ "custom-object-type": "ascii-keys", "type": "object", "minProperties": 1 });
        let compiled = JSONSchema::options()
            .with_keyword("custom-object-type", custom_object_type_factory)
            .compile(&schema)
            .unwrap();

        // Verify schema validation detects object with too few properties
        let instance = json!({});
        assert!(compiled.validate(&instance).is_err());
        assert!(!compiled.is_valid(&instance));

        // Verify validator succeeds on a valid custom-object-type
        let instance = json!({ "a" : 1 });
        assert!(compiled.validate(&instance).is_ok());
        assert!(compiled.is_valid(&instance));

        // Verify validator detects invalid custom-object-type
        let instance = json!({ "å" : 1 });
        let error = compiled
            .validate(&instance)
            .expect_err("Should fail")
            .next()
            .expect("Not empty");
        assert_eq!(error.to_string(), "Key is not ASCII");
        assert!(!compiled.is_valid(&instance));
    }

    #[test]
    fn custom_format_and_override_keyword() {
        /// Check that a string has some number of digits followed by a dot followed by exactly 2 digits.
        fn currency_format_checker(s: &str) -> bool {
            static CURRENCY_RE: Lazy<Regex> = Lazy::new(|| {
                Regex::new("^(0|([1-9]+[0-9]*))(\\.[0-9]{2})$").expect("Invalid regex")
            });
            CURRENCY_RE.is_match(s)
        }
        /// A custom keyword validator that overrides "minimum"
        /// so that "minimum" may apply to "currency"-formatted strings as well.
        struct CustomMinimumValidator {
            limit: f64,
            limit_val: Value,
            with_currency_format: bool,
            schema_path: JSONPointer,
        }

        impl Keyword for CustomMinimumValidator {
            fn validate<'instance>(
                &self,
                instance: &'instance Value,
                instance_path: &JsonPointerNode,
            ) -> ErrorIterator<'instance> {
                if self.is_valid(instance) {
                    no_error()
                } else {
                    error::error(ValidationError::minimum(
                        self.schema_path.clone(),
                        instance_path.into(),
                        instance,
                        self.limit_val.clone(),
                    ))
                }
            }

            fn is_valid(&self, instance: &Value) -> bool {
                match instance {
                    // Numeric comparison should happen just like original behavior
                    Value::Number(instance) => {
                        if let Some(item) = instance.as_u64() {
                            !NumCmp::num_lt(item, self.limit)
                        } else if let Some(item) = instance.as_i64() {
                            !NumCmp::num_lt(item, self.limit)
                        } else {
                            let item = instance.as_f64().expect("Always valid");
                            !NumCmp::num_lt(item, self.limit)
                        }
                    }
                    // String comparison should cast currency-formatted
                    Value::String(instance) => {
                        if self.with_currency_format && currency_format_checker(instance) {
                            // all preconditions for minimum applying are met
                            let value = instance
                                .parse::<f64>()
                                .expect("format validated by regex checker");
                            !NumCmp::num_lt(value, self.limit)
                        } else {
                            true
                        }
                    }
                    // In all other cases, the "minimum" keyword should not apply
                    _ => true,
                }
            }
        }

        /// Build a validator that overrides the standard `minimum` keyword
        fn custom_minimum_factory<'a>(
            parent: &'a Map<String, Value>,
            schema: &'a Value,
            schema_path: JSONPointer,
        ) -> Result<Box<dyn Keyword>, ValidationError<'a>> {
            let limit = if let Value::Number(limit) = schema {
                limit.as_f64().expect("Always valid")
            } else {
                return Err(ValidationError::single_type_error(
                    // There is no metaschema definition for a custom keyword, hence empty `schema` pointer
                    JSONPointer::default(),
                    schema_path,
                    schema,
                    PrimitiveType::Number,
                ));
            };
            let with_currency_format = parent
                .get("format")
                .map_or(false, |format| format == "currency");
            Ok(Box::new(CustomMinimumValidator {
                limit,
                limit_val: schema.clone(),
                with_currency_format,
                schema_path,
            }))
        }

        // Schema includes both the custom format and the overridden keyword
        let schema = json!({ "minimum": 2, "type": "string", "format": "currency" });
        let compiled = JSONSchema::options()
            .with_format("currency", currency_format_checker)
            .with_keyword("minimum", custom_minimum_factory)
            .with_keyword("minimum-2", |_, _, _| todo!())
            .compile(&schema)
            .expect("Invalid schema");

        // Control: verify schema validation rejects non-string types
        let instance = json!(15);
        assert!(compiled.validate(&instance).is_err());
        assert!(!compiled.is_valid(&instance));

        // Control: verify validator rejects ill-formatted strings
        let instance = json!("not a currency");
        assert!(compiled.validate(&instance).is_err());
        assert!(!compiled.is_valid(&instance));

        // Verify validator allows properly formatted strings that conform to custom keyword
        let instance = json!("3.00");
        assert!(compiled.validate(&instance).is_ok());
        assert!(compiled.is_valid(&instance));

        // Verify validator rejects properly formatted strings that do not conform to custom keyword
        let instance = json!("1.99");
        assert!(compiled.validate(&instance).is_err());
        assert!(!compiled.is_valid(&instance));

        // Define another schema that applies "minimum" to an integer to ensure original behavior
        let schema = json!({ "minimum": 2, "type": "integer" });
        let compiled = JSONSchema::options()
            .with_format("currency", currency_format_checker)
            .with_keyword("minimum", custom_minimum_factory)
            .compile(&schema)
            .expect("Invalid schema");

        // Verify schema allows integers greater than 2
        let instance = json!(3);
        assert!(compiled.validate(&instance).is_ok());
        assert!(compiled.is_valid(&instance));

        // Verify schema rejects integers less than 2
        let instance = json!(1);
        assert!(compiled.validate(&instance).is_err());
        assert!(!compiled.is_valid(&instance));

        // Invalid `minimum` value
        let schema = json!({ "minimum": "foo" });
        let error = JSONSchema::options()
            .with_keyword("minimum", custom_minimum_factory)
            .compile(&schema)
            .expect_err("Should fail");
        assert_eq!(error.to_string(), "\"foo\" is not of type \"number\"");
    }
}
