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
//! use jsonschema::{JSONSchema, Draft};
//! use serde_json::json;
//!
//!
//! let schema = json!({"maxLength": 5});
//! let instance = json!("foo");
//! let compiled = JSONSchema::compile(&schema, Some(Draft::Draft7)).unwrap();
//! let result = compiled.validate(&instance);
//! if let Err(errors) = result {
//!     for error in errors {
//!         println!("Validation error: {}", error)
//!     }   
//! }
//!
//! ```
mod checks;
mod context;
mod error;
mod helpers;
mod keywords;
mod resolver;
mod schemas;
mod validator;
pub use error::{ErrorIterator, ValidationError};
pub use schemas::Draft;
use serde_json::Value;
pub use validator::JSONSchema;

#[macro_use]
extern crate lazy_static;

/// Validates `instance` against `schema`. Draft version is detected automatically.
/// ```rust
/// use jsonschema::is_valid;
/// use serde_json::json;
///
///
/// let schema = json!({"maxLength": 5});
/// let instance = json!("foo");
/// assert!(is_valid(&schema, &instance));
/// ```
pub fn is_valid(schema: &Value, instance: &Value) -> bool {
    let compiled = JSONSchema::compile(schema, None).expect("Invalid schema");
    compiled.is_valid(instance)
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
