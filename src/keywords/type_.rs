use super::{CompilationResult, Validate};
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    primitive_type::{PrimitiveType, PrimitiveTypesBitMap},
};
use serde_json::{Map, Number, Value};
use std::convert::TryFrom;

pub struct MultipleTypesValidator {
    types: PrimitiveTypesBitMap,
}

impl MultipleTypesValidator {
    #[inline]
    pub(crate) fn compile(items: &[Value]) -> CompilationResult {
        let mut types = PrimitiveTypesBitMap::new();
        for item in items {
            match item {
                Value::String(string) => {
                    if let Ok(primitive_type) = PrimitiveType::try_from(string.as_str()) {
                        types |= primitive_type;
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
            error(ValidationError::multiple_type_error(instance, self.types))
        }
    }
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        match instance {
            Value::Array(_) => self.types.contains_type(&PrimitiveType::Array),
            Value::Bool(_) => self.types.contains_type(&PrimitiveType::Boolean),
            Value::Null => self.types.contains_type(&PrimitiveType::Null),
            Value::Number(num) => {
                self.types.contains_type(&PrimitiveType::Number)
                    || (self.types.contains_type(&PrimitiveType::Integer) && is_integer(num))
            }
            Value::Object(_) => self.types.contains_type(&PrimitiveType::Object),
            Value::String(_) => self.types.contains_type(&PrimitiveType::String),
        }
    }

    fn name(&self) -> String {
        format!(
            "type: [{}]",
            self.types
                .into_iter()
                .map(|type_| format!("{}", type_))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

pub struct NullTypeValidator {}

impl NullTypeValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(NullTypeValidator {}))
    }
}

impl Validate for NullTypeValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                instance,
                PrimitiveType::Null,
            ))
        }
    }
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_null()
    }

    fn name(&self) -> String {
        "type: null".to_string()
    }
}

pub struct BooleanTypeValidator {}

impl BooleanTypeValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(BooleanTypeValidator {}))
    }
}

impl Validate for BooleanTypeValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                instance,
                PrimitiveType::Boolean,
            ))
        }
    }
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_boolean()
    }

    fn name(&self) -> String {
        "type: boolean".to_string()
    }
}

pub struct StringTypeValidator {}

impl StringTypeValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(StringTypeValidator {}))
    }
}

impl Validate for StringTypeValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                instance,
                PrimitiveType::String,
            ))
        }
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_string()
    }

    fn name(&self) -> String {
        "type: string".to_string()
    }
}

pub struct ArrayTypeValidator {}

impl ArrayTypeValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(ArrayTypeValidator {}))
    }
}

impl Validate for ArrayTypeValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                instance,
                PrimitiveType::Array,
            ))
        }
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_array()
    }

    fn name(&self) -> String {
        "type: array".to_string()
    }
}

pub struct ObjectTypeValidator {}

impl ObjectTypeValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(ObjectTypeValidator {}))
    }
}

impl Validate for ObjectTypeValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                instance,
                PrimitiveType::Object,
            ))
        }
    }
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_object()
    }

    fn name(&self) -> String {
        "type: object".to_string()
    }
}

pub struct NumberTypeValidator {}

impl NumberTypeValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(NumberTypeValidator {}))
    }
}

impl Validate for NumberTypeValidator {
    fn validate<'a>(&self, config: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self.is_valid(config, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                instance,
                PrimitiveType::Number,
            ))
        }
    }
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_number()
    }

    fn name(&self) -> String {
        "type: number".to_string()
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
    num.is_u64() || num.is_i64() || num.as_f64().expect("Always valid").fract() == 0.
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
        Ok(PrimitiveType::Array) => Some(ArrayTypeValidator::compile()),
        Ok(PrimitiveType::Boolean) => Some(BooleanTypeValidator::compile()),
        Ok(PrimitiveType::Integer) => Some(IntegerTypeValidator::compile()),
        Ok(PrimitiveType::Null) => Some(NullTypeValidator::compile()),
        Ok(PrimitiveType::Number) => Some(NumberTypeValidator::compile()),
        Ok(PrimitiveType::Object) => Some(ObjectTypeValidator::compile()),
        Ok(PrimitiveType::String) => Some(StringTypeValidator::compile()),
        Err(()) => Some(Err(CompilationError::SchemaError)),
    }
}
