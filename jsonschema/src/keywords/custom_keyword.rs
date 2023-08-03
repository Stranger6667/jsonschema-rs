use crate::compilation::context::CompilationContext;
use crate::compilation::options::CustomKeywordDefinition;
use crate::keywords::CompilationResult;
use crate::paths::{InstancePath, JSONPointer, PathChunk};
use crate::validator::Validate;
use crate::ErrorIterator;
use serde_json::{json, Value};
use std::fmt::{Display, Formatter};
use std::sync::Arc;

// An Arc<Value> is used so the borrow checker doesn't need explicit lifetime parameters.
// This would pollute dependents with lifetime parameters.
pub(crate) type CustomValidateFn =
    fn(&Value, JSONPointer, Arc<Value>, JSONPointer, Arc<Value>) -> ErrorIterator;
pub(crate) type CustomIsValidFn = fn(&Value, &Value, &Value) -> bool;

/// Custom keyword validation implemented by user provided validation functions.
pub(crate) struct CustomKeywordValidator {
    schema: Arc<Value>,
    subschema: Arc<Value>,
    subschema_path: JSONPointer,
    validate: CustomValidateFn,
    is_valid: CustomIsValidFn,
}

impl Display for CustomKeywordValidator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl Validate for CustomKeywordValidator {
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'instance> {
        (self.validate)(
            instance,
            instance_path.into(),
            self.subschema.clone(),
            self.subschema_path.clone(),
            self.schema.clone(),
        )
    }

    fn is_valid(&self, instance: &Value) -> bool {
        (self.is_valid)(instance, &self.subschema, &self.schema)
    }
}

pub(crate) fn compile_custom_keyword_validator<'a>(
    context: &CompilationContext,
    keyword: impl Into<PathChunk>,
    keyword_definition: &CustomKeywordDefinition,
    subschema: Value,
    schema: Value,
) -> CompilationResult<'a> {
    let subschema_path = context.as_pointer_with(keyword);
    match keyword_definition {
        CustomKeywordDefinition::Validator { validate, is_valid } => {
            Ok(Box::new(CustomKeywordValidator {
                schema: Arc::new(schema),
                subschema: Arc::new(subschema),
                subschema_path,
                validate: *validate,
                is_valid: *is_valid,
            }))
        }
    }
}
