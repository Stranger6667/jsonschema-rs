use super::{CompilationResult, ValidationResult};
use super::{Validate, Validators};
use crate::context::CompilationContext;
use crate::error::{CompilationError, ValidationError};
use crate::keywords::boolean::TrueValidator;
use crate::validator::compile_validators;
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct AdditionalItemsObjectValidator<'a> {
    validators: Validators<'a>,
    items_count: usize,
}
pub struct AdditionalItemsBooleanValidator {
    items_count: usize,
}

impl<'a> AdditionalItemsObjectValidator<'a> {
    pub(crate) fn compile(
        schema: &'a Value,
        items_count: usize,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let validators = compile_validators(schema, context)?;
        Ok(Box::new(AdditionalItemsObjectValidator {
            validators,
            items_count,
        }))
    }
}

impl<'a> AdditionalItemsBooleanValidator {
    pub(crate) fn compile(items_count: usize) -> CompilationResult<'a> {
        Ok(Box::new(AdditionalItemsBooleanValidator { items_count }))
    }
}

impl<'a> Validate<'a> for AdditionalItemsObjectValidator<'a> {
    fn validate(&self, schema: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Array(items) = instance {
            for item in items.iter().skip(self.items_count) {
                for validator in self.validators.iter() {
                    validator.validate(schema, item)?
                }
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!(
            "<additional items ({}): {:?}>",
            self.items_count, self.validators
        )
    }
}

impl<'a> Validate<'a> for AdditionalItemsBooleanValidator {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Array(items) = instance {
            if items.len() > self.items_count {
                return Err(ValidationError::additional_items(
                    items.clone(),
                    self.items_count,
                ));
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<additional items: {}>", self.items_count)
    }
}

pub(crate) fn compile<'a>(
    parent: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
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
