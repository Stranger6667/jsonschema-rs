use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::ValidationResult,
    paths::InstancePath,
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct MaxLengthValidator {
    limit: u64,
}

impl MaxLengthValidator {
    #[inline]
    pub(crate) fn compile<'a>(schema: &'a Value) -> ValidationResult<'a> {
        if let Some(limit) = schema.as_u64() {
            Ok(Box::new(MaxLengthValidator { limit }))
        } else {
            Err(ValidationError::schema(schema))
        }
    }
}

impl Validate for MaxLengthValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            if (item.chars().count() as u64) > self.limit {
                return false;
            }
        }
        true
    }

    fn validate<'a>(
        &self,
        _schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::String(item) = instance {
            if (item.chars().count() as u64) > self.limit {
                return error(ValidationError::max_length(
                    instance_path.into(),
                    instance,
                    self.limit,
                ));
            }
        }
        no_error()
    }
}

impl ToString for MaxLengthValidator {
    fn to_string(&self) -> String {
        format!("maxLength: {}", self.limit)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &'a CompilationContext,
) -> Option<ValidationResult<'a>> {
    Some(MaxLengthValidator::compile(schema))
}
