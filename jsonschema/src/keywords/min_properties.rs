use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    paths::InstancePath,
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct MinPropertiesValidator {
    limit: u64,
}

impl MinPropertiesValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Some(limit) = schema.as_u64() {
            Ok(Box::new(MinPropertiesValidator { limit }))
        } else {
            Err(ValidationError::schema(schema))
        }
    }
}

impl Validate for MinPropertiesValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            if (item.len() as u64) < self.limit {
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
            if (item.len() as u64) < self.limit {
                return error(ValidationError::min_properties(
                    instance_path.into(),
                    instance,
                    self.limit,
                ));
            }
        }
        no_error()
    }
}

impl ToString for MinPropertiesValidator {
    fn to_string(&self) -> String {
        format!("minProperties: {}", self.limit)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &mut CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(MinPropertiesValidator::compile(schema))
}
