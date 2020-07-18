use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{no_error, ErrorIterator},
    keywords::{
        boolean::TrueValidator, format_validators, format_vec_of_validators, CompilationResult,
        Validators,
    },
    validator::Validate,
};
use rayon::prelude::*;
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
    #[inline]
    fn is_valid_array(&self, schema: &JSONSchema, _: &Value, instance_value: &[Value]) -> bool {
        instance_value
            .iter()
            .zip(self.items.iter())
            .all(move |(item, validators)| {
                validators
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
        instance_value: &'a [Value],
    ) -> ErrorIterator<'a> {
        Box::new(
            instance_value
                .iter()
                .zip(self.items.iter())
                .flat_map(move |(item, validators)| {
                    validators
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
    #[inline]
    fn is_valid_array(&self, schema: &JSONSchema, _: &Value, instance_value: &[Value]) -> bool {
        if instance_value.len() > 8 {
            instance_value.par_iter().all(|item| {
                self.validators
                    .iter()
                    .all(|validator| validator.is_valid(schema, item))
            })
        } else {
            self.validators.iter().all(|validator| {
                instance_value
                    .iter()
                    .all(|item| validator.is_valid(schema, item))
            })
        }
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
        instance_value: &'a [Value],
    ) -> ErrorIterator<'a> {
        let errors: Vec<_> = if instance_value.len() > 8 {
            instance_value
                .par_iter()
                .flat_map(|item| {
                    self.validators
                        .iter()
                        .flat_map(|validator| validator.validate(schema, item))
                        .collect::<Vec<_>>()
                })
                .collect()
        } else {
            self.validators
                .iter()
                .flat_map(move |validator| {
                    instance_value
                        .iter()
                        .flat_map(move |item| validator.validate(schema, item))
                })
                .collect()
        };
        Box::new(errors.into_iter())
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
