use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{ErrorIterator, ValidationError},
    keywords::{format_validators, format_vec_of_validators, Validators},
    paths::InstancePath,
    validator::Validate,
};
use serde_json::{Map, Value};

use super::ValidationResult;

pub(crate) struct AllOfValidator {
    schemas: Vec<Validators>,
}

impl AllOfValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        items: &'a [Value],
        context: &'a CompilationContext,
    ) -> ValidationResult<'a> {
        let mut schemas = Vec::with_capacity(items.len());
        for item in items {
            let validators = compile_validators(item, context)?;
            schemas.push(validators)
        }
        Ok(Box::new(AllOfValidator { schemas }))
    }
}

impl Validate for AllOfValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        self.schemas.iter().all(move |validators| {
            validators
                .iter()
                .all(move |validator| validator.is_valid(schema, instance))
        })
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        let errors: Vec<_> = self
            .schemas
            .iter()
            .flat_map(move |validators| {
                validators
                    .iter()
                    .flat_map(move |validator| validator.validate(schema, instance, instance_path))
            })
            .collect();
        Box::new(errors.into_iter())
    }
}

impl ToString for AllOfValidator {
    fn to_string(&self) -> String {
        format!("allOf: [{}]", format_vec_of_validators(&self.schemas))
    }
}
pub(crate) struct SingleValueAllOfValidator {
    validators: Validators,
}

impl SingleValueAllOfValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        context: &'a CompilationContext,
    ) -> ValidationResult<'a> {
        let validators = compile_validators(schema, context)?;
        Ok(Box::new(SingleValueAllOfValidator { validators }))
    }
}

impl Validate for SingleValueAllOfValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        self.validators
            .iter()
            .all(move |validator| validator.is_valid(schema, instance))
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        let errors: Vec<_> = self
            .validators
            .iter()
            .flat_map(move |validator| validator.validate(schema, instance, instance_path))
            .collect();
        Box::new(errors.into_iter())
    }
}

impl ToString for SingleValueAllOfValidator {
    fn to_string(&self) -> String {
        format!("allOf: [{}]", format_validators(&self.validators))
    }
}
#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &'a CompilationContext,
) -> Option<ValidationResult<'a>> {
    if let Value::Array(items) = schema {
        if items.len() == 1 {
            let value = items.iter().next().expect("Vec is not empty");
            Some(SingleValueAllOfValidator::compile(value, context))
        } else {
            Some(AllOfValidator::compile(items, context))
        }
    } else {
        Some(Err(ValidationError::schema(schema)))
    }
}
