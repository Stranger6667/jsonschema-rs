use crate::{compilation::JSONSchema, error::ErrorIterator};
use serde_json::Value;
use std::fmt;

pub trait Validate: Send + Sync {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a>;
    // The same as above, but does not construct ErrorIterator.
    // It is faster for cases when the result is not needed (like anyOf), since errors are
    // not constructed
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool;
    fn name(&self) -> String;
}

impl fmt::Debug for dyn Validate + Send + Sync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}
