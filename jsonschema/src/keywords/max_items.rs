use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::ValidationResult,
    paths::InstancePath,
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct MaxItemsValidator {
    limit: u64,
}

impl MaxItemsValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value) -> ValidationResult {
        if let Some(limit) = schema.as_u64() {
            Ok(Box::new(MaxItemsValidator { limit }))
        } else {
            Err(ValidationError::schema(schema))
        }
    }
}

impl Validate for MaxItemsValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            if (items.len() as u64) > self.limit {
                return false;
            }
        }
        true
    }

    fn validate<'a>(
        &self,
        _: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::Array(items) = instance {
            if (items.len() as u64) > self.limit {
                return error(ValidationError::max_items(
                    instance_path.into(),
                    instance,
                    self.limit,
                ));
            }
        }
        no_error()
    }
}

impl ToString for MaxItemsValidator {
    fn to_string(&self) -> String {
        format!("maxItems: {}", self.limit)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &Map<String, Value>,
    schema: &'a Value,
    _: &CompilationContext,
) -> Option<ValidationResult<'a>> {
    Some(MaxItemsValidator::compile(schema))
}
