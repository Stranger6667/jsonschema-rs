use super::Validate;
use super::{CompilationResult, ValidationResult};
use crate::context::CompilationContext;
use crate::error::ValidationError;
use crate::validator::JSONSchema;
use serde_json::{Map, Value};
use std::collections::HashSet;

pub struct UniqueItemsValidator {}

impl UniqueItemsValidator {
    pub(crate) fn compile<'a>() -> CompilationResult<'a> {
        Ok(Box::new(UniqueItemsValidator {}))
    }
}

impl<'a> Validate<'a> for UniqueItemsValidator {
    fn validate(&self, config: &JSONSchema, instance: &Value) -> ValidationResult {
        if !self.is_valid(config, instance) {
            return Err(ValidationError::unique_items(instance.clone()));
        }
        Ok(())
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            let mut seen = HashSet::with_capacity(items.len());
            for item in items {
                // TODO. Objects can be serialized differently, check `preserve_order` feature in serde_json
                if !seen.insert(item.to_string()) {
                    return false;
                }
            }
        }
        true
    }

    fn name(&self) -> String {
        "<unique items>".to_string()
    }
}
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    if let Value::Bool(value) = schema {
        if *value {
            Some(UniqueItemsValidator::compile())
        } else {
            None
        }
    } else {
        None
    }
}
