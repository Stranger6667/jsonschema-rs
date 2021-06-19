use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    paths::{InstancePath, JSONPointer},
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct MinItemsValidator {
    limit: u64,
    schema_path: JSONPointer,
}

impl MinItemsValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, schema_path: JSONPointer) -> CompilationResult {
        if let Some(limit) = schema.as_u64() {
            Ok(Box::new(MinItemsValidator { limit, schema_path }))
        } else {
            Err(ValidationError::schema(schema))
        }
    }
}

impl Validate for MinItemsValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            if (items.len() as u64) < self.limit {
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
            if (items.len() as u64) < self.limit {
                return error(ValidationError::min_items(
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

impl ToString for MinItemsValidator {
    fn to_string(&self) -> String {
        format!("minItems: {}", self.limit)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    let schema_path = context.as_pointer_with("minItems");
    Some(MinItemsValidator::compile(schema, schema_path))
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::json;

    #[test]
    fn schema_path() {
        tests_util::assert_schema_path(&json!({"minItems": 1}), &json!([]), "/minItems")
    }
}
