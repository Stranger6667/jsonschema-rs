use super::Validate;
use super::{CompilationResult, ValidationResult};
use crate::context::CompilationContext;
use crate::error::{CompilationError, PrimitiveType, ValidationError};
use crate::validator::JSONSchema;
use serde_json::{Map, Number, Value};

pub struct MultipleTypesValidator {
    types: Vec<PrimitiveType>,
}

impl MultipleTypesValidator {
    pub(crate) fn compile<'a>(items: &[Value]) -> CompilationResult<'a> {
        let mut types = Vec::with_capacity(items.len());
        for item in items {
            match item {
                Value::String(string) => match string.as_str() {
                    "integer" => types.push(PrimitiveType::Integer),
                    "null" => types.push(PrimitiveType::Null),
                    "boolean" => types.push(PrimitiveType::Boolean),
                    "string" => types.push(PrimitiveType::String),
                    "array" => types.push(PrimitiveType::Array),
                    "object" => types.push(PrimitiveType::Object),
                    "number" => types.push(PrimitiveType::Number),
                    _ => return Err(CompilationError::SchemaError),
                },
                _ => return Err(CompilationError::SchemaError),
            }
        }
        Ok(Box::new(MultipleTypesValidator { types }))
    }
}

impl<'a> Validate<'a> for MultipleTypesValidator {
    fn validate(&self, schema: &JSONSchema, instance: &Value) -> ValidationResult {
        if !self.is_valid(schema, instance) {
            return Err(ValidationError::multiple_type_error(
                instance.clone(),
                self.types.clone(),
            ));
        }
        Ok(())
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

pub struct NullTypeValidator {}

impl NullTypeValidator {
    pub(crate) fn compile<'a>() -> CompilationResult<'a> {
        Ok(Box::new(NullTypeValidator {}))
    }
}

impl<'a> Validate<'a> for NullTypeValidator {
    fn validate(&self, schema: &JSONSchema, instance: &Value) -> ValidationResult {
        if !self.is_valid(schema, instance) {
            return Err(ValidationError::single_type_error(
                instance.clone(),
                PrimitiveType::Null,
            ));
        }
        Ok(())
    }
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_null()
    }

    fn name(&self) -> String {
        "<type: null>".to_string()
    }
}

pub struct BooleanTypeValidator {}

impl BooleanTypeValidator {
    pub(crate) fn compile<'a>() -> CompilationResult<'a> {
        Ok(Box::new(BooleanTypeValidator {}))
    }
}

impl<'a> Validate<'a> for BooleanTypeValidator {
    fn validate(&self, schema: &JSONSchema, instance: &Value) -> ValidationResult {
        if !self.is_valid(schema, instance) {
            return Err(ValidationError::single_type_error(
                instance.clone(),
                PrimitiveType::Boolean,
            ));
        }
        Ok(())
    }
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_boolean()
    }

    fn name(&self) -> String {
        "<type: boolean>".to_string()
    }
}

pub struct StringTypeValidator {}

impl StringTypeValidator {
    pub(crate) fn compile<'a>() -> CompilationResult<'a> {
        Ok(Box::new(StringTypeValidator {}))
    }
}

impl<'a> Validate<'a> for StringTypeValidator {
    fn validate(&self, schema: &JSONSchema, instance: &Value) -> ValidationResult {
        if !self.is_valid(schema, instance) {
            return Err(ValidationError::single_type_error(
                instance.clone(),
                PrimitiveType::String,
            ));
        }
        Ok(())
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_string()
    }

    fn name(&self) -> String {
        "<type: string>".to_string()
    }
}

pub struct ArrayTypeValidator {}

impl ArrayTypeValidator {
    pub(crate) fn compile<'a>() -> CompilationResult<'a> {
        Ok(Box::new(ArrayTypeValidator {}))
    }
}

impl<'a> Validate<'a> for ArrayTypeValidator {
    fn validate(&self, schema: &JSONSchema, instance: &Value) -> ValidationResult {
        if !self.is_valid(schema, instance) {
            return Err(ValidationError::single_type_error(
                instance.clone(),
                PrimitiveType::Array,
            ));
        }
        Ok(())
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_array()
    }

    fn name(&self) -> String {
        "<type: array>".to_string()
    }
}

pub struct ObjectTypeValidator {}

impl ObjectTypeValidator {
    pub(crate) fn compile<'a>() -> CompilationResult<'a> {
        Ok(Box::new(ObjectTypeValidator {}))
    }
}

impl<'a> Validate<'a> for ObjectTypeValidator {
    fn validate(&self, schema: &JSONSchema, instance: &Value) -> ValidationResult {
        if !self.is_valid(schema, instance) {
            return Err(ValidationError::single_type_error(
                instance.clone(),
                PrimitiveType::Object,
            ));
        }
        Ok(())
    }
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_object()
    }

    fn name(&self) -> String {
        "<type: object>".to_string()
    }
}

pub struct NumberTypeValidator {}

impl NumberTypeValidator {
    pub(crate) fn compile<'a>() -> CompilationResult<'a> {
        Ok(Box::new(NumberTypeValidator {}))
    }
}

impl<'a> Validate<'a> for NumberTypeValidator {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if !instance.is_number() {
            return Err(ValidationError::single_type_error(
                instance.clone(),
                PrimitiveType::Number,
            ));
        }
        Ok(())
    }
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_number()
    }

    fn name(&self) -> String {
        "<type: number>".to_string()
    }
}

pub struct IntegerTypeValidator {}

impl IntegerTypeValidator {
    pub(crate) fn compile<'a>() -> CompilationResult<'a> {
        Ok(Box::new(IntegerTypeValidator {}))
    }
}

impl<'a> Validate<'a> for IntegerTypeValidator {
    fn validate(&self, schema: &JSONSchema, instance: &Value) -> ValidationResult {
        if !self.is_valid(schema, instance) {
            return Err(ValidationError::single_type_error(
                instance.clone(),
                PrimitiveType::Integer,
            ));
        }
        Ok(())
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
    num.is_u64() || num.is_i64() || num.as_f64().unwrap().fract() == 0.
}

pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &CompilationContext,
) -> Option<CompilationResult<'a>> {
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

fn compile_single_type<'a>(item: &str) -> Option<CompilationResult<'a>> {
    match item {
        "integer" => Some(IntegerTypeValidator::compile()),
        "null" => Some(NullTypeValidator::compile()),
        "boolean" => Some(BooleanTypeValidator::compile()),
        "string" => Some(StringTypeValidator::compile()),
        "array" => Some(ArrayTypeValidator::compile()),
        "object" => Some(ObjectTypeValidator::compile()),
        "number" => Some(NumberTypeValidator::compile()),
        _ => Some(Err(CompilationError::SchemaError)),
    }
}
