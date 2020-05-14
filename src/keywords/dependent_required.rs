use super::{CompilationResult, Validate};
use crate::compilation::{CompilationContext, JSONSchema};
use crate::error::{no_error, CompilationError, ErrorIterator, ValidationError};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

pub struct DependentRequiredValidator {
    dependent: HashMap<String, Vec<String>>,
}

impl DependentRequiredValidator {
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        match schema {
            Value::Object(items) => {
                let mut dependent = HashMap::with_capacity(items.len());
                for (key, value) in items {
                    match value {
                        Value::Array(required) => {
                            let capacity = required.len();
                            let dependent_required = dependent
                                .entry(key.clone())
                                .or_insert_with(|| Vec::with_capacity(capacity));
                            let mut seen = HashSet::with_capacity(capacity);
                            for item in required {
                                match item {
                                    Value::String(string) => {
                                        if seen.insert(string) {
                                            dependent_required.push(string.clone())
                                        } else {
                                            return Err(CompilationError::SchemaError);
                                        }
                                    }
                                    _ => return Err(CompilationError::SchemaError),
                                }
                            }
                        }
                        _ => return Err(CompilationError::SchemaError),
                    }
                }
                Ok(Box::new(DependentRequiredValidator { dependent }))
            }
            _ => Err(CompilationError::SchemaError),
        }
    }
}

impl Validate for DependentRequiredValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            for (property_name, dependent) in self.dependent.iter() {
                if item.contains_key(property_name) {
                    for required in dependent.iter() {
                        if !item.contains_key(required) {
                            // Might be more verbose and specify "why" it is required
                            return ValidationError::required(required.clone());
                        }
                    }
                }
            }
            return no_error();
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            return self.dependent.iter().all(|(property_name, dependent)| {
                // Seems like it could be done with `filter`
                if item.contains_key(property_name) {
                    dependent.iter().all(|required| item.contains_key(required))
                } else {
                    true
                }
            });
        }
        true
    }

    fn name(&self) -> String {
        format!("<required: {:?}>", self.dependent)
    }
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(DependentRequiredValidator::compile(schema))
}
