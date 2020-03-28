use super::CompilationResult;
use super::{Validate, Validators};
use crate::context::CompilationContext;
use crate::error::{no_error, CompilationError, ErrorIterator, ValidationError};
use crate::keywords::boolean::TrueValidator;
use crate::validator::compile_validators;
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct AdditionalItemsObjectValidator {
    validators: Validators,
    items_count: usize,
}
pub struct AdditionalItemsBooleanValidator {
    items_count: usize,
}

impl AdditionalItemsObjectValidator {
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

impl<'a> AdditionalItemsBooleanValidator {
    pub(crate) fn compile(items_count: usize) -> CompilationResult {
        Ok(Box::new(AdditionalItemsBooleanValidator { items_count }))
    }
}

impl Validate for AdditionalItemsObjectValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Array(items) = instance {
            let errors: Vec<_> = items
                .iter()
                .skip(self.items_count)
                .flat_map(|item| {
                    self.validators
                        .iter()
                        .flat_map(move |validator| validator.validate(schema, item))
                })
                .collect();
            return Box::new(errors.into_iter());
        }
        no_error()
    }
    fn name(&self) -> String {
        format!(
            "<additional items ({}): {:?}>",
            self.items_count, self.validators
        )
    }
}

impl Validate for AdditionalItemsBooleanValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Array(items) = instance {
            if items.len() > self.items_count {
                return ValidationError::additional_items(items.clone(), self.items_count);
            }
        }
        no_error()
    }
    fn name(&self) -> String {
        format!("<additional items: {}>", self.items_count)
    }
}

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
            _ => Some(Err(CompilationError::SchemaError)),
        }
    } else {
        Some(TrueValidator::compile())
    }
}
