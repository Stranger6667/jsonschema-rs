use super::super::{type_, CompilationResult, Validate};
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, PrimitiveType, ValidationError},
};
use serde_json::{Map, Value};
use std::convert::TryFrom;

pub struct MultipleTypesValidator {
    types: Vec<PrimitiveType>,
}

impl MultipleTypesValidator {
    pub(crate) fn compile(items: &[Value]) -> CompilationResult {
        let mut types = Vec::with_capacity(items.len());
        for item in items {
            if let Some(string) = item.as_str() {
                match PrimitiveType::try_from(string) {
                    Ok(primitive_value) => types.push(primitive_value),
                    Err(_) => return Err(CompilationError::SchemaError),
                }
            } else {
                return Err(CompilationError::SchemaError);
            }
        }
        Ok(Box::new(MultipleTypesValidator { types }))
    }
}

impl Validate for MultipleTypesValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::multiple_type_error(
                instance,
                self.types.clone(),
            ))
        }
    }
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        let instance_primitive_type = PrimitiveType::from(instance);
        for type_ in self.types.iter() {
            if type_ == &instance_primitive_type {
                return true;
            }
        }
        false
    }

    fn name(&self) -> String {
        format!("<type: {:?}>", self.types)
    }
}

pub struct IntegerTypeValidator {}

impl IntegerTypeValidator {
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(IntegerTypeValidator {}))
    }
}

impl Validate for IntegerTypeValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                instance,
                PrimitiveType::Integer,
            ))
        }
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_u64() || instance.is_i64()
    }

    fn name(&self) -> String {
        "<type: integer>".to_string()
    }
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    if let Some(item) = schema.as_str() {
        compile_single_type(item)
    } else if let Some(items) = schema.as_array() {
        if items.len() == 1 {
            // Unwrap is safe as we checked that we have exactly one item
            if let Some(string) = items.iter().next().unwrap().as_str() {
                compile_single_type(string)
            } else {
                Some(Err(CompilationError::SchemaError))
            }
        } else {
            Some(MultipleTypesValidator::compile(items))
        }
    } else {
        Some(Err(CompilationError::SchemaError))
    }
}

fn compile_single_type(item: &str) -> Option<CompilationResult> {
    match item {
        "integer" => Some(IntegerTypeValidator::compile()),
        "null" => Some(type_::NullTypeValidator::compile()),
        "boolean" => Some(type_::BooleanTypeValidator::compile()),
        "string" => Some(type_::StringTypeValidator::compile()),
        "array" => Some(type_::ArrayTypeValidator::compile()),
        "object" => Some(type_::ObjectTypeValidator::compile()),
        "number" => Some(type_::NumberTypeValidator::compile()),
        _ => Some(Err(CompilationError::SchemaError)),
    }
}
