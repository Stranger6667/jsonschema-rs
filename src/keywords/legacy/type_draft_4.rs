use super::super::{type_, CompilationResult, Validate};
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    primitive_type::PrimitiveType,
};
use serde_json::{Map, Number, Value};
use std::convert::TryFrom;

pub struct MultipleTypesValidator {
    types: Vec<PrimitiveType>,
}

impl MultipleTypesValidator {
    #[inline]
    pub(crate) fn compile(items: &[Value]) -> CompilationResult {
        let mut types = Vec::with_capacity(items.len());
        for item in items {
            match item {
                Value::String(string) => {
                    if let Ok(primitive_type) = PrimitiveType::try_from(string.as_str()) {
                        types.push(primitive_type)
                    } else {
                        return Err(CompilationError::SchemaError);
                    }
                }
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
        for type_ in &self.types {
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
        format!(
            "type: [{}]",
            self.types
                .iter()
                .map(|type_| format!("{}", type_))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

pub struct IntegerTypeValidator {}

impl IntegerTypeValidator {
    #[inline]
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
        "type: integer".to_string()
    }
}

fn is_integer(num: &Number) -> bool {
    num.is_u64() || num.is_i64()
}

#[inline]
pub fn compile(
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
    match PrimitiveType::try_from(item) {
        Ok(PrimitiveType::Array) => Some(type_::ArrayTypeValidator::compile()),
        Ok(PrimitiveType::Boolean) => Some(type_::BooleanTypeValidator::compile()),
        Ok(PrimitiveType::Integer) => Some(IntegerTypeValidator::compile()),
        Ok(PrimitiveType::Null) => Some(type_::NullTypeValidator::compile()),
        Ok(PrimitiveType::Number) => Some(type_::NumberTypeValidator::compile()),
        Ok(PrimitiveType::Object) => Some(type_::ObjectTypeValidator::compile()),
        Ok(PrimitiveType::String) => Some(type_::StringTypeValidator::compile()),
        Err(()) => Some(Err(CompilationError::SchemaError)),
    }
}
