//! A high-performance JSON Schema validator for Rust.
//!
//! - 📚 Support for popular JSON Schema drafts
//! - 🔧 Custom keywords and format validators
//! - 🌐 Remote reference fetching (network/file)
//! - 🎨 `Basic` output style as per JSON Schema spec
//!
//! ## Supported drafts
//!
//! Compliance levels vary across drafts, with newer versions having some unimplemented keywords.
//!
//! - ![Draft 2020-12](https://img.shields.io/endpoint?url=https%3A%2F%2Fbowtie.report%2Fbadges%2Frust-jsonschema%2Fcompliance%2Fdraft2020-12.json)
//! - ![Draft 2019-09](https://img.shields.io/endpoint?url=https%3A%2F%2Fbowtie.report%2Fbadges%2Frust-jsonschema%2Fcompliance%2Fdraft2019-09.json)
//! - ![Draft 7](https://img.shields.io/endpoint?url=https%3A%2F%2Fbowtie.report%2Fbadges%2Frust-jsonschema%2Fcompliance%2Fdraft7.json)
//! - ![Draft 6](https://img.shields.io/endpoint?url=https%3A%2F%2Fbowtie.report%2Fbadges%2Frust-jsonschema%2Fcompliance%2Fdraft6.json)
//! - ![Draft 4](https://img.shields.io/endpoint?url=https%3A%2F%2Fbowtie.report%2Fbadges%2Frust-jsonschema%2Fcompliance%2Fdraft4.json)
//!
//! # Validation
//!
//! The `jsonschema` crate offers two main approaches to validation: one-off validation and reusable validators.
//!
//! ## One-off Validation
//!
//! For simple use cases where you need to validate an instance against a schema once, use the `is_valid` function:
//!
//! ```rust
//! use serde_json::json;
//!
//! let schema = json!({"type": "string"});
//! let instance = json!("Hello, world!");
//!
//! assert!(jsonschema::is_valid(&schema, &instance));
//! ```
//!
//! ## Reusable Validators
//!
//! For better performance, especially when validating multiple instances against the same schema, build a validator once and reuse it:
//!
//! ```rust
//! use serde_json::json;
//!
//! let schema = json!({"type": "string"});
//! let validator = jsonschema::compile(&schema)
//!     .expect("Invalid schema");
//!
//! assert!(validator.is_valid(&json!("Hello, world!")));
//! assert!(!validator.is_valid(&json!(42)));
//!
//! // Iterate over all errors
//! let instance = json!(42);
//! let result = validator.validate(&instance);
//! if let Err(errors) = result {
//!     for error in errors {
//!         eprintln!("Error: {}", error);
//!         eprintln!("Location: {}", error.instance_path);
//!     }
//! }
//! ```
//!
//! # Configuration
//!
//! `jsonschema` provides a builder for configuration options via `JSONSchema::options()`.
//!
//! Here is how you can explicitly set the JSON Schema draft version:
//!
//! ```rust
//! use jsonschema::{JSONSchema, Draft};
//! use serde_json::json;
//!
//! let schema = json!({"type": "string"});
//! let validator = JSONSchema::options()
//!     .with_draft(Draft::Draft7)
//!     .compile(&schema)
//!     .expect("Invalid schema");
//! ```
//!
//! For a complete list of configuration options and their usage, please refer to the [`CompilationOptions`] struct.
//!
//! # Reference Resolving
//!
//! By default, `jsonschema` resolves HTTP references using `reqwest` and file references from the local file system.
//!
//! To enable HTTPS support, add the `rustls-tls` feature to `reqwest` in your `Cargo.toml`:
//!
//! ```toml
//! reqwest = { version = "*", features = ["rustls-tls"] }
//! ```
//!
//! You can disable the default behavior using crate features:
//!
//! - Disable HTTP resolving: `default-features = false, features = ["resolve-file"]`
//! - Disable file resolving: `default-features = false, features = ["resolve-http"]`
//! - Disable both: `default-features = false`
//!
//! You can implement a custom resolver to handle external references. Here's an example that uses a static map of schemas:
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use std::{collections::HashMap, sync::Arc};
//! use anyhow::anyhow;
//! use jsonschema::{JSONSchema, SchemaResolver, SchemaResolverError};
//! use serde_json::{json, Value};
//! use url::Url;
//!
//! struct StaticSchemaResolver {
//!     schemas: HashMap<String, Arc<Value>>,
//! }
//!
//! impl SchemaResolver for StaticSchemaResolver {
//!     fn resolve(
//!         &self,
//!         _root_schema: &serde_json::Value,
//!         url: &Url,
//!         _original_reference: &str
//!     ) -> Result<Arc<Value>, SchemaResolverError> {
//!         self.schemas
//!             .get(url.as_str())
//!             .cloned()
//!             .ok_or_else(|| anyhow!("Schema not found: {}", url))
//!     }
//! }
//!
//! let mut schemas = HashMap::new();
//! schemas.insert(
//!     "https://example.com/person.json".to_string(),
//!     Arc::new(json!({
//!         "type": "object",
//!         "properties": {
//!             "name": { "type": "string" },
//!             "age": { "type": "integer" }
//!         },
//!         "required": ["name", "age"]
//!     })),
//! );
//!
//! let resolver = StaticSchemaResolver { schemas };
//!
//! let schema = json!({
//!     "$ref": "https://example.com/person.json"
//! });
//!
//! let validator = JSONSchema::options()
//!     .with_resolver(resolver)
//!     .compile(&schema)
//!     .expect("Invalid schema");
//!
//! assert!(validator.is_valid(&json!({
//!     "name": "Alice",
//!     "age": 30
//! })));
//!
//! assert!(!validator.is_valid(&json!({
//!     "name": "Bob"
//! })));
//! #    Ok(())
//! # }
//! ```
//! # Output Styles
//!
//! `jsonschema` supports the `basic` output style as defined in JSON Schema Draft 2019-09.
//! This styles allow you to serialize validation results in a standardized format using `serde`.
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use jsonschema::BasicOutput;
//! use serde_json::json;
//!
//! let schema_json = json!({
//!     "title": "string value",
//!     "type": "string"
//! });
//! let instance = json!("some string");
//! let schema = jsonschema::compile(&schema_json)
//!     .expect("Invalid schema");
//!
//! let output: BasicOutput = schema.apply(&instance).basic();
//! let output_json = serde_json::to_value(output)?;
//!
//! assert_eq!(
//!     output_json,
//!     json!({
//!         "valid": true,
//!         "annotations": [
//!             {
//!                 "keywordLocation": "",
//!                 "instanceLocation": "",
//!                 "annotations": {
//!                     "title": "string value"
//!                 }
//!             }
//!         ]
//!     })
//! );
//! #    Ok(())
//! # }
//! ```
//!
//! # Custom Keywords
//!
//! `jsonschema` allows you to extend its functionality by implementing custom validation logic through custom keywords.
//! This feature is particularly useful when you need to validate against domain-specific rules that aren't covered by the standard JSON Schema keywords.
//!
//! To implement a custom keyword, you need to:
//! 1. Create a struct that implements the `Keyword` trait
//! 2. Create a factory function or closure that produces instances of your custom keyword
//! 3. Register the custom keyword with the `JSONSchema` instance using the `with_keyword` method
//!
//! Here's a complete example:
//!
//! ```rust
//! use jsonschema::{
//!     paths::{JSONPointer, JsonPointerNode},
//!     ErrorIterator, JSONSchema, Keyword, ValidationError,
//! };
//! use serde_json::{json, Map, Value};
//! use std::iter::once;
//!
//! // Step 1: Implement the Keyword trait
//! struct EvenNumberValidator;
//!
//! impl Keyword for EvenNumberValidator {
//!     fn validate<'instance>(
//!         &self,
//!         instance: &'instance Value,
//!         instance_path: &JsonPointerNode,
//!     ) -> ErrorIterator<'instance> {
//!         if let Value::Number(n) = instance {
//!             if n.as_u64().map_or(false, |n| n % 2 == 0) {
//!                 Box::new(None.into_iter())
//!             } else {
//!                 let error = ValidationError::custom(
//!                     JSONPointer::default(),
//!                     instance_path.into(),
//!                     instance,
//!                     "Number must be even",
//!                 );
//!                 Box::new(once(error))
//!             }
//!         } else {
//!             let error = ValidationError::custom(
//!                 JSONPointer::default(),
//!                 instance_path.into(),
//!                 instance,
//!                 "Value must be a number",
//!             );
//!             Box::new(once(error))
//!         }
//!     }
//!
//!     fn is_valid(&self, instance: &Value) -> bool {
//!         instance.as_u64().map_or(false, |n| n % 2 == 0)
//!     }
//! }
//!
//! // Step 2: Create a factory function
//! fn even_number_validator_factory<'a>(
//!     _parent: &'a Map<String, Value>,
//!     value: &'a Value,
//!     _path: JSONPointer,
//! ) -> Result<Box<dyn Keyword>, ValidationError<'a>> {
//!     // You can use the `value` parameter to configure your validator if needed
//!     if value.as_bool() == Some(true) {
//!         Ok(Box::new(EvenNumberValidator))
//!     } else {
//!         Err(ValidationError::custom(
//!             JSONPointer::default(),
//!             JSONPointer::default(),
//!             value,
//!             "The 'even-number' keyword must be set to true",
//!         ))
//!     }
//! }
//!
//! // Step 3: Use the custom keyword
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let schema = json!({"even-number": true, "type": "integer"});
//!     let validator = JSONSchema::options()
//!         .with_keyword("even-number", even_number_validator_factory)
//!         .compile(&schema)
//!         .expect("Invalid schema");
//!
//!     assert!(validator.is_valid(&json!(2)));
//!     assert!(!validator.is_valid(&json!(3)));
//!     assert!(!validator.is_valid(&json!("not a number")));
//!
//!     Ok(())
//! }
//! ```
//!
//! In this example, we've created a custom `even-number` keyword that validates whether a number is even.
//! The `EvenNumberValidator` implements the actual validation logic, while the `even_number_validator_factory`
//! creates instances of the validator and allows for additional configuration based on the keyword's value in the schema.
//!
//! You can also use a closure instead of a factory function for simpler cases:
//!
//! ```rust
//! # use jsonschema::{
//! #     paths::{JSONPointer, JsonPointerNode},
//! #     ErrorIterator, JSONSchema, Keyword, ValidationError,
//! # };
//! # use serde_json::{json, Map, Value};
//! # use std::iter::once;
//! #
//! # struct EvenNumberValidator;
//! #
//! # impl Keyword for EvenNumberValidator {
//! #     fn validate<'instance>(
//! #         &self,
//! #         instance: &'instance Value,
//! #         instance_path: &JsonPointerNode,
//! #     ) -> ErrorIterator<'instance> {
//! #         Box::new(None.into_iter())
//! #     }
//! #
//! #     fn is_valid(&self, instance: &Value) -> bool {
//! #         true
//! #     }
//! # }
//! let schema = json!({"even-number": true, "type": "integer"});
//! let validator = JSONSchema::options()
//!     .with_keyword("even-number", |_, _, _| {
//!         Ok(Box::new(EvenNumberValidator))
//!     })
//!     .compile(&schema)
//!     .expect("Invalid schema");
//! ```
//!
//! # Custom Formats
//!
//! JSON Schema allows for format validation through the `format` keyword. While `jsonschema`
//! provides built-in validators for standard formats, you can also define custom format validators
//! for domain-specific string formats.
//!
//! To implement a custom format validator:
//!
//! 1. Define a function or a closure that takes a `&str` and returns a `bool`.
//! 2. Register the function with `JSONSchema::options().with_format()`.
//!
//! ```rust
//! use jsonschema::JSONSchema;
//! use serde_json::json;
//!
//! // Step 1: Define the custom format validator function
//! fn ends_with_42(s: &str) -> bool {
//!     s.ends_with("42!")
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Step 2: Create a schema using the custom format
//! let schema = json!({
//!     "type": "string",
//!     "format": "ends-with-42"
//! });
//!
//! // Step 3: Compile the schema with the custom format
//! let validator = JSONSchema::options()
//!     .with_format("ends-with-42", ends_with_42)
//!     .with_format("ends-with-43", |s: &str| s.ends_with("43!"))
//!     .compile(&schema)
//!     .expect("Invalid schema");
//!
//! // Step 4: Validate instances
//! assert!(validator.is_valid(&json!("Hello42!")));
//! assert!(!validator.is_valid(&json!("Hello43!")));
//! assert!(!validator.is_valid(&json!(42))); // Not a string
//! #    Ok(())
//! # }
//! ```
//!
//! ### Notes on Custom Format Validators
//!
//! - Custom format validators are only called for string instances.
//! - Format validation can be disabled globally or per-draft using `CompilationOptions`.
//!   Ensure format validation is enabled if you're using custom formats.
mod compilation;
mod content_encoding;
mod content_media_type;
pub mod error;
mod keywords;
pub mod output;
pub mod paths;
pub mod primitive_type;
pub(crate) mod properties;
mod resolver;
mod schema_node;
mod schemas;
mod validator;

pub use compilation::{options::CompilationOptions, JSONSchema};
pub use error::{ErrorIterator, ValidationError};
pub use keywords::custom::Keyword;
pub use output::BasicOutput;
pub use resolver::{SchemaResolver, SchemaResolverError};
pub use schemas::Draft;

use serde_json::Value;

/// A shortcut for validating `instance` against `schema`. Draft version is detected automatically.
/// ```rust
/// use jsonschema::is_valid;
/// use serde_json::json;
///
/// let schema = json!({"maxLength": 5});
/// let instance = json!("foo");
/// assert!(is_valid(&schema, &instance));
/// ```
///
/// This function panics if an invalid schema is passed.
#[must_use]
#[inline]
pub fn is_valid(schema: &Value, instance: &Value) -> bool {
    let compiled = JSONSchema::compile(schema).expect("Invalid schema");
    compiled.is_valid(instance)
}

/// Compile the input schema for faster validation.
pub fn compile(schema: &Value) -> Result<JSONSchema, ValidationError> {
    JSONSchema::compile(schema)
}

#[cfg(test)]
pub(crate) mod tests_util {
    use super::JSONSchema;
    use crate::ValidationError;
    use serde_json::Value;

    pub(crate) fn is_not_valid_with(compiled: &JSONSchema, instance: &Value) {
        assert!(
            !compiled.is_valid(instance),
            "{} should not be valid (via is_valid)",
            instance
        );
        assert!(
            compiled.validate(instance).is_err(),
            "{} should not be valid (via validate)",
            instance
        );
        assert!(
            !compiled.apply(instance).basic().is_valid(),
            "{} should not be valid (via apply)",
            instance
        );
    }

    pub(crate) fn is_not_valid(schema: &Value, instance: &Value) {
        let compiled = JSONSchema::compile(schema).unwrap();
        is_not_valid_with(&compiled, instance)
    }

    pub(crate) fn is_not_valid_with_draft(draft: crate::Draft, schema: &Value, instance: &Value) {
        let compiled = JSONSchema::options()
            .with_draft(draft)
            .compile(schema)
            .unwrap();
        is_not_valid_with(&compiled, instance)
    }

    pub(crate) fn expect_errors(schema: &Value, instance: &Value, errors: &[&str]) {
        assert_eq!(
            JSONSchema::compile(schema)
                .expect("Should be a valid schema")
                .validate(instance)
                .expect_err(format!("{} should not be valid", instance).as_str())
                .map(|e| e.to_string())
                .collect::<Vec<String>>(),
            errors
        )
    }

    pub(crate) fn is_valid_with(compiled: &JSONSchema, instance: &Value) {
        if let Err(mut errors) = compiled.validate(instance) {
            let first = errors.next().expect("Errors iterator is empty");
            panic!(
                "{} should be valid (via validate). Error: {} at {}",
                instance, first, first.instance_path
            );
        }
        assert!(
            compiled.is_valid(instance),
            "{} should be valid (via is_valid)",
            instance
        );
        assert!(
            compiled.apply(instance).basic().is_valid(),
            "{} should be valid (via apply)",
            instance
        );
    }

    pub(crate) fn is_valid(schema: &Value, instance: &Value) {
        let compiled = JSONSchema::compile(schema).unwrap();
        is_valid_with(&compiled, instance);
    }

    pub(crate) fn is_valid_with_draft(draft: crate::Draft, schema: &Value, instance: &Value) {
        let compiled = JSONSchema::options()
            .with_draft(draft)
            .compile(schema)
            .unwrap();
        is_valid_with(&compiled, instance)
    }

    pub(crate) fn validate(schema: &Value, instance: &Value) -> ValidationError<'static> {
        let compiled = JSONSchema::compile(schema).unwrap();
        let err = compiled
            .validate(instance)
            .expect_err("Should be an error")
            .next()
            .expect("Should be an error")
            .into_owned();
        err
    }

    pub(crate) fn assert_schema_path(schema: &Value, instance: &Value, expected: &str) {
        let error = validate(schema, instance);
        assert_eq!(error.schema_path.to_string(), expected)
    }

    pub(crate) fn assert_schema_paths(schema: &Value, instance: &Value, expected: &[&str]) {
        let compiled = JSONSchema::compile(schema).unwrap();
        let errors = compiled.validate(instance).expect_err("Should be an error");
        for (error, schema_path) in errors.zip(expected) {
            assert_eq!(error.schema_path.to_string(), *schema_path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_valid, Draft, JSONSchema};
    use serde_json::json;
    use test_case::test_case;

    #[test]
    fn test_is_valid() {
        let schema = json!({"minLength": 5});
        let valid = json!("foobar");
        let invalid = json!("foo");
        assert!(is_valid(&schema, &valid));
        assert!(!is_valid(&schema, &invalid));
    }

    #[test_case(Draft::Draft4)]
    #[test_case(Draft::Draft6)]
    #[test_case(Draft::Draft7)]
    fn meta_schemas(draft: Draft) {
        // See GH-258
        for schema in [json!({"enum": [0, 0.0]}), json!({"enum": []})] {
            assert!(JSONSchema::options()
                .with_draft(draft)
                .compile(&schema)
                .is_ok())
        }
    }

    #[test]
    fn incomplete_escape_in_pattern() {
        // See GH-253
        let schema = json!({"pattern": "\\u"});
        assert!(JSONSchema::compile(&schema).is_err())
    }
}
