use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{format_validators, CompilationResult, Validators},
    paths::{InstancePath, JSONPointer},
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct NotValidator {
    // needed only for error representation
    original: Value,
    validators: Validators,
    schema_path: JSONPointer,
}

impl NotValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let keyword_context = context.with_path("not");
        Ok(Box::new(NotValidator {
            original: schema.clone(),
            validators: compile_validators(schema, &keyword_context)?,
            schema_path: keyword_context.into_pointer(),
        }))
    }
}

impl Validate for NotValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        !self
            .validators
            .iter()
            .all(|validator| validator.is_valid(schema, instance))
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
            error(ValidationError::not(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
                self.original.clone(),
            ))
        }
    }
}

impl ToString for NotValidator {
    fn to_string(&self) -> String {
        format!("not: {}", format_validators(&self.validators))
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(NotValidator::compile(schema, context))
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::json;

    #[test]
    fn schema_path() {
        tests_util::assert_schema_path(&json!({"not": {"type": "string"}}), &json!("foo"), "/not")
    }
}
