use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{no_error, ErrorIterator},
    keywords::{
        boolean::TrueValidator, format_validators, format_vec_of_validators, CompilationResult,
        Validators,
    },
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct ItemsArrayValidator {
    items: Vec<Validators>,
}
impl ItemsArrayValidator {
    #[inline]
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
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            items
                .iter()
                .zip(self.items.iter())
                .all(move |(item, validators)| {
                    validators
                        .iter()
                        .all(move |validator| validator.is_valid(schema, item))
                })
        } else {
            true
        }
    }

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
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl ToString for ItemsArrayValidator {
    fn to_string(&self) -> String {
        format!("items: [{}]", format_vec_of_validators(&self.items))
    }
}

pub(crate) struct ItemsObjectValidator {
    validators: Validators,
}
impl ItemsObjectValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        let validators = compile_validators(schema, context)?;
        Ok(Box::new(ItemsObjectValidator { validators }))
    }
}
impl Validate for ItemsObjectValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            self.validators.iter().all(move |validator| {
                items
                    .iter()
                    .all(move |item| validator.is_valid(schema, item))
            })
        } else {
            true
        }
    }

    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Array(items) = instance {
            let errors: Vec<_> = self
                .validators
                .iter()
                .flat_map(move |validator| {
                    items
                        .iter()
                        .flat_map(move |item| validator.validate(schema, item))
                })
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl ToString for ItemsObjectValidator {
    fn to_string(&self) -> String {
        format!("items: {}", format_validators(&self.validators))
    }
}

#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    match schema {
        Value::Array(items) => Some(ItemsArrayValidator::compile(items, context)),
        Value::Object(_) => Some(ItemsObjectValidator::compile(schema, context)),
        Value::Bool(value) => {
            if *value {
                Some(TrueValidator::compile())
            } else {
                Some(ItemsObjectValidator::compile(schema, context))
            }
        }
        _ => None,
    }
}
