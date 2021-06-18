use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{format_vec_of_validators, Validators},
    paths::InstancePath,
    validator::Validate,
};
use serde_json::{Map, Value};

use super::CompilationResult;
use crate::paths::JSONPointer;

pub(crate) struct AnyOfValidator {
    schemas: Vec<Validators>,
    schema_path: JSONPointer,
}

impl AnyOfValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        if let Value::Array(items) = schema {
            let keyword_context = context.with_path("anyOf");
            let mut schemas = Vec::with_capacity(items.len());
            for (idx, item) in items.iter().enumerate() {
                let item_context = keyword_context.with_path(idx);
                let validators = compile_validators(item, &item_context)?;
                schemas.push(validators)
            }
            Ok(Box::new(AnyOfValidator {
                schemas,
                schema_path: keyword_context.into_pointer(),
            }))
        } else {
            Err(ValidationError::schema(schema))
        }
    }
}

impl Validate for AnyOfValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        for validators in &self.schemas {
            if validators
                .iter()
                .all(|validator| validator.is_valid(schema, instance))
            {
                return true;
            }
        }
        false
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::any_of(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
            ))
        }
    }
}

impl ToString for AnyOfValidator {
    fn to_string(&self) -> String {
        format!("anyOf: [{}]", format_vec_of_validators(&self.schemas))
    }
}
#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(AnyOfValidator::compile(schema, context))
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"anyOf": [{"type": "string"}]}), &json!(1), "/anyOf")]
    #[test_case(&json!({"anyOf": [{"type": "integer"}, {"type": "string"}]}), &json!({}), "/anyOf")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
