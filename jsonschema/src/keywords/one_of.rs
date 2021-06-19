use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{format_vec_of_validators, CompilationResult, Validators},
    paths::{InstancePath, JSONPointer},
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct OneOfValidator {
    schemas: Vec<Validators>,
    schema_path: JSONPointer,
}

impl OneOfValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        if let Value::Array(items) = schema {
            let keyword_context = context.with_path("oneOf");
            let mut schemas = Vec::with_capacity(items.len());
            for item in items {
                schemas.push(compile_validators(item, &keyword_context)?)
            }
            Ok(Box::new(OneOfValidator {
                schemas,
                schema_path: keyword_context.into_pointer(),
            }))
        } else {
            Err(ValidationError::schema(schema))
        }
    }

    fn get_first_valid(&self, schema: &JSONSchema, instance: &Value) -> Option<usize> {
        let mut first_valid_idx = None;
        for (idx, validators) in self.schemas.iter().enumerate() {
            if validators
                .iter()
                .all(|validator| validator.is_valid(schema, instance))
            {
                first_valid_idx = Some(idx);
                break;
            }
        }
        first_valid_idx
    }

    #[allow(clippy::integer_arithmetic)]
    fn are_others_valid(&self, schema: &JSONSchema, instance: &Value, idx: usize) -> bool {
        // `idx + 1` will not overflow, because the maximum possible value there is `usize::MAX - 1`
        // For example we have `usize::MAX` schemas and only the last one is valid, then
        // in `get_first_valid` we enumerate from `0`, and on the last index will be `usize::MAX - 1`
        for validators in self.schemas.iter().skip(idx + 1) {
            if validators
                .iter()
                .all(|validator| validator.is_valid(schema, instance))
            {
                return true;
            }
        }
        false
    }
}

impl Validate for OneOfValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        let first_valid_idx = self.get_first_valid(schema, instance);
        first_valid_idx.map_or(false, |idx| !self.are_others_valid(schema, instance, idx))
    }
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        let first_valid_idx = self.get_first_valid(schema, instance);
        if let Some(idx) = first_valid_idx {
            if self.are_others_valid(schema, instance, idx) {
                return error(ValidationError::one_of_multiple_valid(
                    self.schema_path.clone(),
                    instance_path.into(),
                    instance,
                ));
            }
            no_error()
        } else {
            error(ValidationError::one_of_not_valid(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
            ))
        }
    }
}

impl ToString for OneOfValidator {
    fn to_string(&self) -> String {
        format!("oneOf: [{}]", format_vec_of_validators(&self.schemas))
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(OneOfValidator::compile(schema, context))
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"oneOf": [{"type": "string"}]}), &json!(0), "/oneOf")]
    #[test_case(&json!({"oneOf": [{"type": "string"}, {"maxLength": 3}]}), &json!(""), "/oneOf")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
