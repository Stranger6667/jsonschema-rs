use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{no_error, ErrorIterator, ValidationError},
    keywords::{format_validators, CompilationResult, Validators},
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct PropertyNamesObjectValidator {
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
    #[inline]
    fn is_valid_object(
        &self,
        schema: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        self.validators.iter().all(|validator| {
            instance_value
                .keys()
                .all(|key| validator.is_valid_string(schema, &Value::String(key.to_string()), key))
        })
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(instance_value) = instance {
            self.is_valid_object(schema, instance, instance_value)
        } else {
            true
        }
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
                .flat_map(|validator| {
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
    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(instance_value) = instance {
            self.validate_object(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}
impl ToString for PropertyNamesObjectValidator {
    fn to_string(&self) -> String {
        format!("propertyNames: {}", format_validators(&self.validators))
    }
}

pub(crate) struct PropertyNamesBooleanValidator {}

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

    #[inline]
    fn is_valid_object(
        &self,
        _: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        instance_value.is_empty()
    }
    #[inline]
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(instance_value) = instance {
            instance_value.is_empty()
        } else {
            true
        }
    }
}
impl ToString for PropertyNamesBooleanValidator {
    fn to_string(&self) -> String {
        "propertyNames: false".to_string()
    }
}

#[inline]
pub(crate) fn compile(
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
