use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{CompilationError, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::{Map, Value};

pub struct MinPropertiesValidator {
    limit: u64,
}

impl MinPropertiesValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Some(limit) = schema.as_u64() {
            return Ok(Box::new(MinPropertiesValidator { limit }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for MinPropertiesValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::min_properties(instance, self.limit)
    }

    fn name(&self) -> String {
        format!("minProperties: {}", self.limit)
    }

    #[inline]
    fn is_valid_object(
        &self,
        _: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        instance_value.len() as u64 >= self.limit
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(MinPropertiesValidator::compile(schema))
}
