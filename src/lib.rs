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
