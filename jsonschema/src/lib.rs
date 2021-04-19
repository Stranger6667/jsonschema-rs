//! # jsonschema
//!
//! A crate for performing fast JSON Schema validation. It is fast due to schema compilation into
//! a validation tree, which reduces runtime costs for working with schema parameters.
//!
//! Supports:
//!   - JSON Schema drafts 4, 6, 7 (except some optional test cases);
//!   - Loading remote documents via HTTP(S);
//!
//! ## Usage Examples:
//! A schema can be compiled with two main flavours:
//!  * using default configurations
//! ```rust
//! # use jsonschema::{CompilationError, Draft, JSONSchema};
//! # use serde_json::json;
//! # fn foo() -> Result<(), CompilationError> {
//! # let schema = json!({"maxLength": 5});
//! let compiled_schema = JSONSchema::compile(&schema)?;
//! # Ok(())
//! # }
//! ```
//!  * using custom configurations (such as define a Draft version)
//! ```rust
//! # use jsonschema::{CompilationError, Draft, JSONSchema};
//! # use serde_json::json;
//! # fn foo() -> Result<(), CompilationError> {
//! # let schema = json!({"maxLength": 5});
//! let compiled_schema = JSONSchema::options()
//!     .with_draft(Draft::Draft7)
//!     .compile(&schema)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Example (CLI tool to highlight print errors)
//! ```rust
//! use jsonschema::{CompilationError, Draft, JSONSchema};
//! use serde_json::json;
//!
//! fn main() -> Result<(), CompilationError> {
//!     let schema = json!({"maxLength": 5});
//!     let instance = json!("foo");
//!     let compiled = JSONSchema::options()
//!         .with_draft(Draft::Draft7)
//!         .compile(&schema)?;
//!     let result = compiled.validate(&instance);
//!     if let Err(errors) = result {
//!         for error in errors {
//!             println!("Validation error: {}", error)
//!         }
//!     }
//!     Ok(())
//! }
//! ```
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
#![allow(clippy::unnecessary_wraps, clippy::upper_case_acronyms)]
#![cfg_attr(not(test), allow(clippy::integer_arithmetic, clippy::unwrap_used))]
mod compilation;
mod content_encoding;
mod content_media_type;
pub mod error;
mod keywords;
pub mod primitive_type;
mod resolver;
mod schemas;
mod validator;
pub use compilation::{options::CompilationOptions, JSONSchema};
pub use error::{CompilationError, ErrorIterator, ValidationError};
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
    }
}

#[cfg(test)]
mod tests {
    use super::is_valid;
    use serde_json::json;

    #[test]
    fn test_is_valid() {
        let schema = json!({"minLength": 5});
        let valid = json!("foobar");
        let invalid = json!("foo");
        assert!(is_valid(&schema, &valid));
        assert!(!is_valid(&schema, &invalid));
    }
}
