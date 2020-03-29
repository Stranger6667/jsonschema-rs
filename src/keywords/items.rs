use super::boolean::TrueValidator;
use super::{CompilationResult, Validate, Validators};
use crate::compilation::{compile_validators, CompilationContext, JSONSchema};
use crate::error::{no_error, ErrorIterator};
use serde_json::{Map, Value};

pub struct ItemsArrayValidator {
    items: Vec<Validators>,
}

impl ItemsArrayValidator {
    pub(crate) fn compile(schemas: &[Value], context: &CompilationContext) -> CompilationResult {
        let mut items = Vec::with_capacity(schemas.len());
        for item in schemas {
            let validators = compile_validators(item, context)?;
            items.push(validators)
        }
        Ok(Box::new(ItemsArrayValidator { items }))
    }
}

impl Validate for ItemsArrayValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Array(items) = instance {
            let errors: Vec<_> = items
                .iter()
                .zip(self.items.iter())
                .flat_map(move |(item, validators)| {
                    validators
                        .iter()
                        .flat_map(move |validator| validator.validate(schema, item))
                })
                .collect();
            return Box::new(errors.into_iter());
        }
        no_error()
    }

    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            return items
                .iter()
                .zip(self.items.iter())
                .all(move |(item, validators)| {
                    validators
                        .iter()
                        .all(move |validator| validator.is_valid(schema, item))
                });
        }
        true
    }

    fn name(&self) -> String {
        format!("<items: {:?}>", self.items)
    }
}

pub struct ItemsObjectValidator {
    validators: Validators,
}

impl ItemsObjectValidator {
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        let validators = compile_validators(schema, context)?;
        Ok(Box::new(ItemsObjectValidator { validators }))
    }
}

impl Validate for ItemsObjectValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Array(items) = instance {
            // TODO. make parallel
            let errors: Vec<_> = self
                .validators
                .iter()
                .flat_map(move |validator| {
                    items
                        .iter()
                        .flat_map(move |item| validator.validate(schema, item))
                })
                .collect();
            return Box::new(errors.into_iter());
        }
        no_error()
    }

    fn name(&self) -> String {
        format!("<items: {:#?}>", self.validators)
    }
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    match schema {
        Value::Array(items) => Some(ItemsArrayValidator::compile(&items, &context)),
        Value::Object(_) => Some(ItemsObjectValidator::compile(schema, &context)),
        Value::Bool(value) => {
            if *value {
                Some(TrueValidator::compile())
            } else {
                Some(ItemsObjectValidator::compile(schema, &context))
            }
        }
        _ => None,
    }
}
