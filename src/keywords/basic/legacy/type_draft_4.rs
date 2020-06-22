use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::{basic::type_, CompilationResult},
    primitive_type::{PrimitiveType, PrimitiveTypesBitMap},
    validator::Validate,
};
use serde_json::{Map, Value};
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
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::multiple_type_error(instance, self.types)
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, _: &[Value]) -> bool {
        self.types.contains_type(PrimitiveType::Array)
    }
    #[inline]
    fn is_valid_boolean(&self, _: &JSONSchema, _: &Value, _: bool) -> bool {
        self.types.contains_type(PrimitiveType::Boolean)
    }
    #[inline]
    fn is_valid_null(&self, _: &JSONSchema, _: &Value, _: ()) -> bool {
        self.types.contains_type(PrimitiveType::Null)
    }
    #[inline]
    fn is_valid_number(&self, _: &JSONSchema, _: &Value, _: f64) -> bool {
        self.types.contains_type(PrimitiveType::Number)
    }
    #[inline]
    fn is_valid_object(&self, _: &JSONSchema, _: &Value, _: &Map<String, Value>) -> bool {
        self.types.contains_type(PrimitiveType::Object)
    }
    #[inline]
    fn is_valid_signed_integer(&self, _: &JSONSchema, _: &Value, _: i64) -> bool {
        self.types.contains_type(PrimitiveType::Integer)
    }
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, _: &str) -> bool {
        self.types.contains_type(PrimitiveType::String)
    }
    #[inline]
    fn is_valid_unsigned_integer(&self, _: &JSONSchema, _: &Value, _: u64) -> bool {
        self.types.contains_type(PrimitiveType::Integer)
    }
}
impl ToString for MultipleTypesValidator {
    fn to_string(&self) -> String {
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
pub struct IntegerTypeValidator {}

impl IntegerTypeValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(IntegerTypeValidator {}))
    }
}

impl Validate for IntegerTypeValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::single_type_error(instance, PrimitiveType::Integer)
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, _: &[Value]) -> bool {
        false
    }
    #[inline]
    fn is_valid_boolean(&self, _: &JSONSchema, _: &Value, _: bool) -> bool {
        false
    }
    #[inline]
    fn is_valid_null(&self, _: &JSONSchema, _: &Value, _: ()) -> bool {
        false
    }
    #[inline]
    fn is_valid_number(&self, _: &JSONSchema, _: &Value, _: f64) -> bool {
        false
    }
    #[inline]
    fn is_valid_object(&self, _: &JSONSchema, _: &Value, _: &Map<String, Value>) -> bool {
        false
    }
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, _: &str) -> bool {
        false
    }
    #[inline]
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Number(instance_number) = instance {
            instance_number.is_u64() || instance_number.is_i64()
        } else {
            false
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Number(_) = instance {
            if self.is_valid(schema, instance) {
                no_error()
            } else {
                error(self.build_validation_error(instance))
            }
        } else {
            error(self.build_validation_error(instance))
        }
    }
}
impl ToString for IntegerTypeValidator {
    fn to_string(&self) -> String {
        "type: integer".to_string()
    }
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
