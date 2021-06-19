use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    paths::{InstancePath, JSONPointer},
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct MinLengthValidator {
    limit: u64,
    schema_path: JSONPointer,
}

impl MinLengthValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, schema_path: JSONPointer) -> CompilationResult {
        if let Some(limit) = schema.as_u64() {
            Ok(Box::new(MinLengthValidator { limit, schema_path }))
        } else {
            Err(ValidationError::schema(schema))
        }
    }
}

impl Validate for MinLengthValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            if (item.chars().count() as u64) < self.limit {
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
        if let Value::String(item) = instance {
            if (item.chars().count() as u64) < self.limit {
                return error(ValidationError::min_length(
                    self.schema_path.clone(),
                    instance_path.into(),
                    instance,
                    self.limit,
                ));
            }
        }
        no_error()
    }
}

impl ToString for MinLengthValidator {
    fn to_string(&self) -> String {
        format!("minLength: {}", self.limit)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    let schema_path = context.as_pointer_with("minLength");
    Some(MinLengthValidator::compile(schema, schema_path))
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::json;

    #[test]
    fn schema_path() {
        tests_util::assert_schema_path(&json!({"minLength": 1}), &json!(""), "/minLength")
    }
}
