use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{format_vec_of_validators, Validators},
    paths::InstancePath,
    validator::Validate,
};
use serde_json::{Map, Value};

use super::CompilationResult;

pub(crate) struct AnyOfValidator {
    schemas: Vec<Validators>,
}

impl AnyOfValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        context: &mut CompilationContext,
    ) -> CompilationResult<'a> {
        if let Value::Array(items) = schema {
            let mut schemas = Vec::with_capacity(items.len());
            for item in items {
                context.schema_path.push(item.to_string());
                let validators = compile_validators(item, context)?;
                context.schema_path.pop();
                schemas.push(validators)
            }
            Ok(Box::new(AnyOfValidator { schemas }))
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
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &mut CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(AnyOfValidator::compile(schema, context))
}
