use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::ValidationResult,
    paths::InstancePath,
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct MaxPropertiesValidator {
    limit: u64,
}

impl MaxPropertiesValidator {
    #[inline]
    pub(crate) fn compile<'a>(schema: &'a Value) -> ValidationResult<'a> {
        if let Some(limit) = schema.as_u64() {
            Ok(Box::new(MaxPropertiesValidator { limit }))
        } else {
            Err(ValidationError::schema(schema))
        }
    }
}

impl Validate for MaxPropertiesValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            if (item.len() as u64) > self.limit {
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
        if let Value::Object(item) = instance {
            if (item.len() as u64) > self.limit {
                return error(ValidationError::max_properties(
                    instance_path.into(),
                    instance,
                    self.limit,
                ));
            }
        }
        no_error()
    }
}

impl ToString for MaxPropertiesValidator {
    fn to_string(&self) -> String {
        format!("maxProperties: {}", self.limit)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &'a CompilationContext,
) -> Option<ValidationResult<'a>> {
    Some(MaxPropertiesValidator::compile(schema))
}
