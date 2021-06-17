use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{boolean::FalseValidator, format_validators, ValidationResult, Validators},
    paths::InstancePath,
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct AdditionalItemsObjectValidator {
    validators: Validators,
    items_count: usize,
}
impl AdditionalItemsObjectValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        items_count: usize,
        context: &CompilationContext,
    ) -> ValidationResult<'a> {
        let validators = compile_validators(schema, context)?;
        Ok(Box::new(AdditionalItemsObjectValidator {
            validators,
            items_count,
        }))
    }
}
impl Validate for AdditionalItemsObjectValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            items.iter().skip(self.items_count).all(|item| {
                self.validators
                    .iter()
                    .all(move |validator| validator.is_valid(schema, item))
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
        if let Value::Array(items) = instance {
            let errors: Vec<_> = items
                .iter()
                .enumerate()
                .skip(self.items_count)
                .flat_map(|(idx, item)| {
                    self.validators.iter().flat_map(move |validator| {
                        validator.validate(schema, item, &instance_path.push(idx))
                    })
                })
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}
impl ToString for AdditionalItemsObjectValidator {
    fn to_string(&self) -> String {
        format!("additionalItems: {}", format_validators(&self.validators))
    }
}

pub(crate) struct AdditionalItemsBooleanValidator {
    items_count: usize,
}
impl AdditionalItemsBooleanValidator {
    #[inline]
    pub(crate) fn compile<'a>(items_count: usize) -> ValidationResult<'a> {
        Ok(Box::new(AdditionalItemsBooleanValidator { items_count }))
    }
}
impl Validate for AdditionalItemsBooleanValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            if items.len() > self.items_count {
                return false;
            }
        }
        true
    }

    fn validate<'a>(
        &self,
        _: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::Array(items) = instance {
            if items.len() > self.items_count {
                return error(ValidationError::additional_items(
                    instance_path.into(),
                    instance,
                    self.items_count,
                ));
            }
        }
        no_error()
    }
}
impl ToString for AdditionalItemsBooleanValidator {
    fn to_string(&self) -> String {
        "additionalItems: false".to_string()
    }
}

#[inline]
pub(crate) fn compile<'a>(
    parent: &Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<ValidationResult<'a>> {
    if let Some(items) = parent.get("items") {
        match items {
            Value::Object(_) => None,
            Value::Array(items) => {
                let items_count = items.len();
                match schema {
                    Value::Object(_) => Some(AdditionalItemsObjectValidator::compile(
                        schema,
                        items_count,
                        context,
                    )),
                    Value::Bool(false) => {
                        Some(AdditionalItemsBooleanValidator::compile(items_count))
                    }
                    _ => None,
                }
            }
            Value::Bool(value) => {
                if *value {
                    None
                } else {
                    Some(FalseValidator::compile())
                }
            }
            _ => Some(Err(ValidationError::schema(schema))),
        }
    } else {
        None
    }
}
