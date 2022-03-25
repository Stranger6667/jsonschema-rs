//! # jsonschema
//!
//! A crate for performing fast JSON Schema validation. It is fast due to schema compilation into
//! a validation tree, which reduces runtime costs for working with schema parameters.
//!
//! Supports:
//!   - JSON Schema drafts 4, 6, 7 (except some optional test cases);
//!   - Loading remote documents via HTTP(S);
//!
//! This library is functional and ready for use, but its API is still evolving to the 1.0 API.
//!
//! ## Usage Examples:
//! A schema can be compiled with two main flavours:
//!  * using default configurations
//! ```rust
//! # use jsonschema::JSONSchema;
//! # use serde_json::json;
//! # fn foo() {
//! # let schema = json!({"maxLength": 5});
//! let compiled_schema = JSONSchema::compile(&schema).expect("A valid schema");
//! # }
//! ```
//!  * using custom configurations (such as define a Draft version)
//! ```rust
//! # use jsonschema::{Draft, JSONSchema};
//! # use serde_json::json;
//! # fn foo() {
//! # let schema = json!({"maxLength": 5});
//! let compiled_schema = JSONSchema::options()
//!     .with_draft(Draft::Draft7)
//!     .compile(&schema)
//!     .expect("A valid schema");
//! # }
//! ```
//!
//! ## Example (CLI tool to highlight print errors)
//! ```rust
//! use jsonschema::{Draft, JSONSchema};
//! use serde_json::json;
//!
//! let schema = json!({"maxLength": 5});
//! let instance = json!("foo");
//! let compiled = JSONSchema::options()
//!     .with_draft(Draft::Draft7)
//!     .compile(&schema)
//!     .expect("A valid schema");
//! let result = compiled.validate(&instance);
//! if let Err(errors) = result {
//!     for error in errors {
//!         println!("Validation error: {}", error);
//!         println!("Instance path: {}", error.instance_path);
//!     }
//! }
//! ```
//! Each error has an `instance_path` attribute that indicates the path to the erroneous part within the validated instance.
//! It could be transformed to JSON Pointer via `.to_string()` or to `Vec<String>` via `.into_vec()`.
#![warn(
    clippy::cast_possible_truncation,
    clippy::doc_markdown,
    clippy::explicit_iter_loop,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::needless_pass_by_value,
    clippy::print_stdout,
    clippy::redundant_closure,
    clippy::trivially_copy_pass_by_ref,
    clippy::missing_const_for_fn,
    clippy::unseparated_literal_suffix,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unreachable_pub,
    variant_size_differences
)]
#![allow(
    clippy::unnecessary_wraps,
    clippy::upper_case_acronyms,
    clippy::needless_collect
)]
#![cfg_attr(not(test), allow(clippy::integer_arithmetic, clippy::unwrap_used))]
mod compilation;
mod content_encoding;
mod content_media_type;
pub mod error;
mod keywords;
pub mod output;
pub mod paths;
pub mod primitive_type;
mod resolver;
mod schema_node;
mod schemas;
mod validator;

pub use compilation::{options::CompilationOptions, JSONSchema, options::KeywordDefinition};
pub use error::{ErrorIterator, ValidationError};
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

#[cfg(test)]
pub(crate) mod tests_util {
    use super::JSONSchema;
    use crate::ValidationError;
    use serde_json::Value;

    pub(crate) fn is_not_valid(schema: &Value, instance: &Value) {
        let compiled = JSONSchema::compile(schema).unwrap();
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

    pub(crate) fn expect_errors(schema: &Value, instance: &Value, errors: &[&str]) {
        assert_eq!(
            JSONSchema::compile(schema)
                .expect("Should be a valid schema")
                .validate(instance)
                .expect_err(format!("{} should not be valid", instance).as_str())
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>(),
            errors
        )
    }

    pub(crate) fn is_valid(schema: &Value, instance: &Value) {
        let compiled = JSONSchema::compile(schema).unwrap();
        assert!(
            compiled.is_valid(instance),
            "{} should be valid (via is_valid)",
            instance
        );
        assert!(
            compiled.validate(instance).is_ok(),
            "{} should be valid (via validate)",
            instance
        );
        assert!(
            compiled.apply(instance).basic().is_valid(),
            "{} should be valid (via apply)",
            instance
        );
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
