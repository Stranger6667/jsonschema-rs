use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{format_validators, CompilationResult, Validators},
    paths::{InstancePath, JSONPointer},
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct PropertyNamesObjectValidator {
    validators: Validators,
}

impl PropertyNamesObjectValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let keyword_context = context.with_path("propertyNames");
        Ok(Box::new(PropertyNamesObjectValidator {
            validators: compile_validators(schema, &keyword_context)?,
        }))
    }
}

impl Validate for PropertyNamesObjectValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = &instance {
            self.validators.iter().all(|validator| {
                item.keys().all(move |key| {
                    let wrapper = Value::String(key.to_string());
                    validator.is_valid(schema, &wrapper)
                })
            })
        } else {
            true
        }
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::Object(item) = &instance {
            let errors: Vec<_> = self
                .validators
                .iter()
                .flat_map(|validator| {
                    item.keys().flat_map(move |key| {
                        let wrapper = Value::String(key.to_string());
                        let errors: Vec<_> = validator
                            .validate(schema, &wrapper, instance_path)
                            .map(|error| {
                                ValidationError::property_names(
                                    error.schema_path.clone(),
                                    instance_path.into(),
                                    instance,
                                    error.into_owned(),
                                )
                            })
                            .collect();
                        errors.into_iter()
                    })
                })
                .collect();
            Box::new(errors.into_iter())
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

pub(crate) struct PropertyNamesBooleanValidator {
    schema_path: JSONPointer,
}

impl PropertyNamesBooleanValidator {
    #[inline]
    pub(crate) fn compile<'a>(context: &CompilationContext) -> CompilationResult<'a> {
        let schema_path = context.as_pointer_with("propertyNames");
        Ok(Box::new(PropertyNamesBooleanValidator { schema_path }))
    }
}

impl Validate for PropertyNamesBooleanValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            if !item.is_empty() {
                return false;
            }
        }
        true
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
            error(ValidationError::false_schema(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
            ))
        }
    }
}

impl ToString for PropertyNamesBooleanValidator {
    fn to_string(&self) -> String {
        "propertyNames: false".to_string()
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    match schema {
        Value::Object(_) => Some(PropertyNamesObjectValidator::compile(schema, context)),
        Value::Bool(false) => Some(PropertyNamesBooleanValidator::compile(context)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"propertyNames": false}), &json!({"foo": 1}), "/propertyNames")]
    #[test_case(&json!({"propertyNames": {"minLength": 2}}), &json!({"f": 1}), "/propertyNames/minLength")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
