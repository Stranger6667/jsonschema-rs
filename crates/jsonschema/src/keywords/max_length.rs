use crate::{
    compilation::context::CompilationContext,
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{helpers::fail_on_non_positive_integer, CompilationResult},
    paths::{JSONPointer, JsonPointerNode},
    validator::Validate,
    Draft,
};
use serde_json::{Map, Value};

pub(crate) struct MaxLengthValidator {
    limit: u64,
    schema_path: JSONPointer,
}

impl MaxLengthValidator {
    #[inline]
    pub(crate) fn compile(
        schema: &Value,
        schema_path: JSONPointer,
        draft: Draft,
    ) -> CompilationResult {
        if let Some(limit) = schema.as_u64() {
            return Ok(Box::new(MaxLengthValidator { limit, schema_path }));
        }
        if !matches!(draft, Draft::Draft4) {
            if let Some(limit) = schema.as_f64() {
                if limit.trunc() == limit {
                    #[allow(clippy::cast_possible_truncation)]
                    return Ok(Box::new(MaxLengthValidator {
                        // NOTE: Imprecise cast as big integers are not supported yet
                        limit: limit as u64,
                        schema_path,
                    }));
                }
            }
        }
        Err(fail_on_non_positive_integer(schema, schema_path))
    }
}

impl Validate for MaxLengthValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            if (bytecount::num_chars(item.as_bytes()) as u64) > self.limit {
                return false;
            }
        }
        true
    }

    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if let Value::String(item) = instance {
            if (bytecount::num_chars(item.as_bytes()) as u64) > self.limit {
                return error(ValidationError::max_length(
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

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    let schema_path = context.as_pointer_with("maxLength");
    Some(MaxLengthValidator::compile(
        schema,
        schema_path,
        context.config.draft(),
    ))
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::json;

    #[test]
    fn schema_path() {
        tests_util::assert_schema_path(&json!({"maxLength": 1}), &json!("ab"), "/maxLength")
    }
}
