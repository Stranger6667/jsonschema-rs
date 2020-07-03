use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::{
        boolean::{FalseValidator, TrueValidator},
        format_validators, CompilationResult, Validators,
    },
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct AdditionalItemsObjectValidator {
    validators: Validators,
    items_count: usize,
}
impl AdditionalItemsObjectValidator {
    #[inline]
    pub(crate) fn compile(
        schema: &Value,
        items_count: usize,
        context: &CompilationContext,
    ) -> CompilationResult {
        let validators = compile_validators(schema, context)?;
        Ok(Box::new(AdditionalItemsObjectValidator {
            validators,
            items_count,
        }))
    }
}
impl Validate for AdditionalItemsObjectValidator {
    #[inline]
    fn is_valid_array(&self, schema: &JSONSchema, _: &Value, instance_array: &[Value]) -> bool {
        instance_array.iter().skip(self.items_count).all(|item| {
            self.validators
                .iter()
                .all(move |validator| validator.is_valid(schema, item))
        })
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(instance_value) = instance {
            self.is_valid_array(schema, instance, instance_value)
        } else {
            true
        }
    }

    #[inline]
    fn validate_array<'a>(
        &self,
        schema: &'a JSONSchema,
        _: &'a Value,
        instance_array: &'a [Value],
    ) -> ErrorIterator<'a> {
        Box::new(
            instance_array
                .iter()
                .skip(self.items_count)
                .flat_map(|item| {
                    self.validators
                        .iter()
                        .flat_map(move |validator| validator.validate(schema, item))
                })
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Array(instance_value) = instance {
            self.validate_array(schema, instance, instance_value)
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
    pub(crate) fn compile(items_count: usize) -> CompilationResult {
        Ok(Box::new(AdditionalItemsBooleanValidator { items_count }))
    }
}
impl Validate for AdditionalItemsBooleanValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::additional_items(instance, self.items_count)
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, instance_array: &[Value]) -> bool {
        instance_array.len() <= self.items_count
    }
    #[inline]
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(instance_array) = instance {
            instance_array.len() <= self.items_count
        } else {
            true
        }
    }
}
impl ToString for AdditionalItemsBooleanValidator {
    fn to_string(&self) -> String {
        "additionalItems: false".to_string()
    }
}

#[inline]
pub(crate) fn compile(
    parent: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    if let Some(items) = parent.get("items") {
        match items {
            Value::Object(_) => Some(TrueValidator::compile()),
            Value::Array(items) => {
                let items_count = items.len();
                match schema {
                    Value::Object(_) => Some(AdditionalItemsObjectValidator::compile(
                        schema,
                        items_count,
                        context,
                    )),
                    Value::Bool(true) => Some(TrueValidator::compile()),
                    Value::Bool(false) => {
                        Some(AdditionalItemsBooleanValidator::compile(items_count))
                    }
                    _ => None,
                }
            }
            Value::Bool(value) => {
                if *value {
                    Some(TrueValidator::compile())
                } else {
                    Some(FalseValidator::compile())
                }
            }
            _ => Some(Err(CompilationError::SchemaError)),
        }
    } else {
        Some(TrueValidator::compile())
    }
}
