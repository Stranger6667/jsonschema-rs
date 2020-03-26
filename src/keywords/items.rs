use super::{CompilationResult, ValidationResult};
use super::{Validate, Validators};
use crate::context::CompilationContext;
use crate::error::ValidationError;
use crate::keywords::boolean::TrueValidator;
use crate::validator::compile_validators;
use crate::JSONSchema;
use rayon::prelude::*;
use serde_json::{Map, Value};

static PARALLEL_ITEMS_THRESHOLD: usize = 8;

pub struct ItemsArrayValidator<'a> {
    items: Vec<Validators<'a>>,
}

impl<'a> ItemsArrayValidator<'a> {
    pub(crate) fn compile(
        schemas: &'a [Value],
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let mut items = Vec::with_capacity(schemas.len());
        for item in schemas {
            let validators = compile_validators(item, context)?;
            items.push(validators)
        }
        Ok(Box::new(ItemsArrayValidator { items }))
    }
}

impl<'a> Validate<'a> for ItemsArrayValidator<'a> {
    fn validate(&self, schema: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Array(items) = instance {
            for (item, validators) in items.iter().zip(self.items.iter()) {
                for validator in validators {
                    validator.validate(schema, item)?
                }
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<items: {:?}>", self.items)
    }
}

pub struct ItemsObjectValidator<'a> {
    validators: Validators<'a>,
}

impl<'a> ItemsObjectValidator<'a> {
    pub(crate) fn compile(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let validators = compile_validators(schema, context)?;
        Ok(Box::new(ItemsObjectValidator { validators }))
    }
}

impl<'a> Validate<'a> for ItemsObjectValidator<'a> {
    fn validate(&self, schema: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Array(items) = instance {
            if items.len() > PARALLEL_ITEMS_THRESHOLD {
                let validate = |item| {
                    for validator in self.validators.iter() {
                        match validator.validate(schema, item) {
                            Ok(_) => continue,
                            Err(e) => return Err(e),
                        }
                    }
                    Ok(())
                };
                if items.par_iter().map(validate).any(|res| res.is_err()) {
                    // TODO. it should be propagated! not necessarily "schema" error
                    return Err(ValidationError::schema());
                }
            } else {
                for item in items {
                    for validator in self.validators.iter() {
                        validator.validate(schema, item)?
                    }
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> String {
        format!("<items: {:#?}>", self.validators)
    }
}

pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
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
