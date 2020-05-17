//! # jsonschema
//!
//! A crate for performing fast JSON Schema validation. It is fast due to schema compilation into
//! a validation tree, which reduces runtime costs for working with schema parameters.
//!
//! Supports:
//!   - JSON Schema drafts 6, 7 (all test cases);
//!   - Loading remote documents via HTTP(S);
//!
//! ## Example:
//!
//! ```rust
//! use jsonschema::{JSONSchema, Draft, CompilationError};
//! use serde_json::json;
//!
//!fn main() -> Result<(), CompilationError> {
//!    let schema = json!({"maxLength": 5});
//!    let instance = json!("foo");
//!    let compiled = JSONSchema::compile(&schema, Some(Draft::Draft7))?;
//!    let result = compiled.validate(&instance);
//!    if let Err(errors) = result {
//!        for error in errors {
//!            println!("Validation error: {}", error)
//!        }
//!    }
//!    Ok(())
//! }
//!
//! ```
#![warn(
    clippy::doc_markdown,
    clippy::redundant_closure,
    clippy::explicit_iter_loop,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::print_stdout,
    clippy::integer_arithmetic,
    clippy::cast_possible_truncation,
    clippy::result_unwrap_used,
    clippy::result_map_unwrap_or_else,
    clippy::option_unwrap_used,
    clippy::option_map_unwrap_or_else,
    clippy::option_map_unwrap_or,
    clippy::trivially_copy_pass_by_ref,
    clippy::needless_pass_by_value,
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]
mod compilation;
mod error;
mod keywords;
mod primitive_type;
mod resolver;
mod schemas;
mod validator;
pub use compilation::JSONSchema;
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
    let compiled = JSONSchema::compile(schema, None).expect("Invalid schema");
    compiled.is_valid(instance)
}

#[cfg(test)]
mod tests_util {
    use super::JSONSchema;
    use serde_json::Value;

    pub fn is_not_valid(schema: Value, instance: Value) {
        let compiled = JSONSchema::compile(&schema, None).unwrap();
        assert!(!compiled.is_valid(&instance), "{} should not be valid");
        assert!(
            compiled.validate(&instance).is_err(),
            "{} should not be valid"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
