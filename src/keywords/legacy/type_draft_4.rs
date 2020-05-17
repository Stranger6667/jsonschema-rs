use super::super::{type_, CompilationResult, Validate};
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, PrimitiveType, ValidationError},
};
use serde_json::{Map, Number, Value};
use std::convert::TryFrom;

pub struct MultipleTypesValidator {
    types: Vec<PrimitiveType>,
}

impl MultipleTypesValidator {
    pub(crate) fn compile(items: &[Value]) -> CompilationResult {
        let mut types = Vec::with_capacity(items.len());
        for item in items {
            match item {
                Value::String(string) => match PrimitiveType::try_from(string.as_str()) {
                    Ok(primitive_value) => types.push(primitive_value),
                    Err(_) => return Err(CompilationError::SchemaError),
                },
                _ => return Err(CompilationError::SchemaError),
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
        for type_ in self.types.iter() {
            match (type_, instance) {
                (PrimitiveType::Integer, Value::Number(num)) if is_integer(num) => return true,
                (PrimitiveType::Null, Value::Null)
                | (PrimitiveType::Boolean, Value::Bool(_))
                | (PrimitiveType::String, Value::String(_))
                | (PrimitiveType::Array, Value::Array(_))
                | (PrimitiveType::Object, Value::Object(_))
                | (PrimitiveType::Number, Value::Number(_)) => return true,
                (_, _) => continue,
            };
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
        if let Value::Number(num) = instance {
            return is_integer(num);
        }
        false
    }

    fn name(&self) -> String {
        "<type: integer>".to_string()
    }
}

fn is_integer(num: &Number) -> bool {
    num.is_u64() || num.is_i64()
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    match schema {
        Value::String(item) => compile_single_type(item.as_str()),
        Value::Array(items) => {
            if items.len() == 1 {
                if let Some(Value::String(item)) = items.iter().next() {
                    compile_single_type(item.as_str())
                } else {
                    Some(Err(CompilationError::SchemaError))
                }
            } else {
                Some(MultipleTypesValidator::compile(items))
            }
        }
        _ => Some(Err(CompilationError::SchemaError)),
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
