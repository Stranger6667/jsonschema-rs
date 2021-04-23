use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::{format_vec_of_validators, CompilationResult, InstancePath, Validators},
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct AnyOfValidator {
    schemas: Vec<Validators>,
}

impl AnyOfValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        if let Value::Array(items) = schema {
            let mut schemas = Vec::with_capacity(items.len());
            for item in items {
                let validators = compile_validators(item, context)?;
                schemas.push(validators)
            }
            Ok(Box::new(AnyOfValidator { schemas }))
        } else {
            Err(CompilationError::SchemaError)
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

    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::any_of(instance_path.into(), instance))
        }
    }
}

impl ToString for AnyOfValidator {
    fn to_string(&self) -> String {
        format!("anyOf: [{}]", format_vec_of_validators(&self.schemas))
    }
}
#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(AnyOfValidator::compile(schema, context))
}
