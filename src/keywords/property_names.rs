use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{ErrorIterator, ValidationError},
    keywords::{format_validators, CompilationResult, Validators},
    validator::Validate,
};
use serde_json::{Map, Value};

pub struct PropertyNamesObjectValidator {
    validators: Validators,
}

impl PropertyNamesObjectValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        Ok(Box::new(PropertyNamesObjectValidator {
            validators: compile_validators(schema, context)?,
        }))
    }
}

impl Validate for PropertyNamesObjectValidator {
    fn name(&self) -> String {
        format!("propertyNames: {}", format_validators(&self.validators))
    }

    #[inline]
    fn is_valid_object(
        &self,
        schema: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        self.validators.iter().all(move |validator| {
            instance_value.keys().all(move |key| {
                validator.is_valid_string(schema, &Value::String(key.to_string()), key)
            })
        })
    }

    #[inline]
    fn validate_object<'a>(
        &self,
        schema: &'a JSONSchema,
        _: &'a Value,
        instance_value: &Map<String, Value>,
    ) -> ErrorIterator<'a> {
        Box::new(
            self.validators
                .iter()
                .flat_map(move |validator| {
                    instance_value.keys().flat_map(move |key| {
                        let wrapper = Value::String(key.to_string());
                        let errors: Vec<_> = validator
                            .validate_string(schema, &wrapper, key)
                            .map(ValidationError::into_owned)
                            .collect();
                        errors.into_iter()
                    })
                })
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
}

pub struct PropertyNamesBooleanValidator {}

impl PropertyNamesBooleanValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(PropertyNamesBooleanValidator {}))
    }
}

impl Validate for PropertyNamesBooleanValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::false_schema(instance)
    }

    fn name(&self) -> String {
        "propertyNames: false".to_string()
    }

    #[inline]
    fn is_valid_object(
        &self,
        _: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        instance_value.is_empty()
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    match schema {
        Value::Object(_) => Some(PropertyNamesObjectValidator::compile(schema, context)),
        Value::Bool(false) => Some(PropertyNamesBooleanValidator::compile()),
        _ => None,
    }
}
