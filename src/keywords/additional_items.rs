use super::{boolean::TrueValidator, CompilationResult, Validate, Validators};
use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
};
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

impl AdditionalItemsBooleanValidator {
    pub(crate) fn compile(items_count: usize) -> CompilationResult {
        Ok(Box::new(AdditionalItemsBooleanValidator { items_count }))
    }
}

impl Validate for AdditionalItemsObjectValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Some(items) = instance.as_array() {
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

    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Some(items) = instance.as_array() {
            return items.iter().skip(self.items_count).all(|item| {
                self.validators
                    .iter()
                    .all(move |validator| validator.is_valid(schema, item))
            });
        }
        true
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
        if let Some(items) = instance.as_array() {
            if items.len() > self.items_count {
                return error(ValidationError::additional_items(
                    instance,
                    self.items_count,
                ));
            }
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Some(items) = instance.as_array() {
            if items.len() > self.items_count {
                return false;
            }
        }
        true
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
        if items.is_object() {
            Some(TrueValidator::compile())
        } else if let Some(items) = items.as_array() {
            if schema.is_object() {
                Some(AdditionalItemsObjectValidator::compile(
                    schema,
                    items.len(),
                    context,
                ))
            } else if let Some(boolean) = schema.as_bool() {
                if boolean {
                    Some(TrueValidator::compile())
                } else {
                    Some(AdditionalItemsBooleanValidator::compile(items.len()))
                }
            } else {
                None
            }
        } else {
            Some(Err(CompilationError::SchemaError))
        }
    } else {
        Some(TrueValidator::compile())
    }
}
