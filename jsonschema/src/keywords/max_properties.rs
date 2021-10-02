use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    paths::{InstancePath, JSONPointer},
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct MaxPropertiesValidator {
    limit: u64,
    schema_path: JSONPointer,
}

impl MaxPropertiesValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, schema_path: JSONPointer) -> CompilationResult {
        if let Some(limit) = schema.as_u64() {
            Ok(Box::new(MaxPropertiesValidator { limit, schema_path }))
        } else {
            Err(ValidationError::format(
                schema_path,
                JSONPointer::default(),
                schema,
                "max_properties int validation",
            ))
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

    fn validate<'a, 'b>(
        &self,
        _: &'a JSONSchema,
        instance: &'b Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'b> {
        if let Value::Object(item) = instance {
            if (item.len() as u64) > self.limit {
                return error(ValidationError::max_properties(
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

impl core::fmt::Display for MaxPropertiesValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "maxProperties: {}", self.limit)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    let schema_path = context.as_pointer_with("maxProperties");
    Some(MaxPropertiesValidator::compile(schema, schema_path))
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::json;

    #[test]
    fn schema_path() {
        tests_util::assert_schema_path(
            &json!({"maxProperties": 1}),
            &json!({"a": 1, "b": 2}),
            "/maxProperties",
        )
    }
}
