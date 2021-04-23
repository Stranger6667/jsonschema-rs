use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    primitive_type::{PrimitiveType, PrimitiveTypesBitMap},
    validator::Validate,
};
use serde_json::{Map, Number, Value};
use std::convert::TryFrom;

use super::InstancePath;

pub(crate) struct MultipleTypesValidator {
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
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        match instance {
            Value::Array(_) => self.types.contains_type(PrimitiveType::Array),
            Value::Bool(_) => self.types.contains_type(PrimitiveType::Boolean),
            Value::Null => self.types.contains_type(PrimitiveType::Null),
            Value::Number(num) => {
                self.types.contains_type(PrimitiveType::Number)
                    || (self.types.contains_type(PrimitiveType::Integer) && is_integer(num))
            }
            Value::Object(_) => self.types.contains_type(PrimitiveType::Object),
            Value::String(_) => self.types.contains_type(PrimitiveType::String),
        }
    }
    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::multiple_type_error(
                instance_path.into(),
                instance,
                self.types,
            ))
        }
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

pub(crate) struct NullTypeValidator {}

impl NullTypeValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(NullTypeValidator {}))
    }
}

impl Validate for NullTypeValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_null()
    }
    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                instance_path.into(),
                instance,
                PrimitiveType::Null,
            ))
        }
    }
}

impl ToString for NullTypeValidator {
    fn to_string(&self) -> String {
        "type: null".to_string()
    }
}

pub(crate) struct BooleanTypeValidator {}

impl BooleanTypeValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(BooleanTypeValidator {}))
    }
}

impl Validate for BooleanTypeValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_boolean()
    }
    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                instance_path.into(),
                instance,
                PrimitiveType::Boolean,
            ))
        }
    }
}

impl ToString for BooleanTypeValidator {
    fn to_string(&self) -> String {
        "type: boolean".to_string()
    }
}

pub(crate) struct StringTypeValidator {}

impl StringTypeValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(StringTypeValidator {}))
    }
}

impl Validate for StringTypeValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_string()
    }

    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                instance_path.into(),
                instance,
                PrimitiveType::String,
            ))
        }
    }
}
impl ToString for StringTypeValidator {
    fn to_string(&self) -> String {
        "type: string".to_string()
    }
}

pub(crate) struct ArrayTypeValidator {}

impl ArrayTypeValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(ArrayTypeValidator {}))
    }
}

impl Validate for ArrayTypeValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_array()
    }

    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                instance_path.into(),
                instance,
                PrimitiveType::Array,
            ))
        }
    }
}

impl ToString for ArrayTypeValidator {
    fn to_string(&self) -> String {
        "type: array".to_string()
    }
}

pub(crate) struct ObjectTypeValidator {}

impl ObjectTypeValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(ObjectTypeValidator {}))
    }
}

impl Validate for ObjectTypeValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_object()
    }
    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                instance_path.into(),
                instance,
                PrimitiveType::Object,
            ))
        }
    }
}

impl ToString for ObjectTypeValidator {
    fn to_string(&self) -> String {
        "type: object".to_string()
    }
}

pub(crate) struct NumberTypeValidator {}

impl NumberTypeValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(NumberTypeValidator {}))
    }
}

impl Validate for NumberTypeValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        instance.is_number()
    }
    fn validate<'a, 'b>(
        &'b self,
        config: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if self.is_valid(config, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                instance_path.into(),
                instance,
                PrimitiveType::Number,
            ))
        }
    }
}
impl ToString for NumberTypeValidator {
    fn to_string(&self) -> String {
        "type: number".to_string()
    }
}
pub(crate) struct IntegerTypeValidator {}

impl IntegerTypeValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(IntegerTypeValidator {}))
    }
}

impl Validate for IntegerTypeValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Number(num) = instance {
            is_integer(num)
        } else {
            false
        }
    }
    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                instance_path.into(),
                instance,
                PrimitiveType::Integer,
            ))
        }
    }
}

impl ToString for IntegerTypeValidator {
    fn to_string(&self) -> String {
        "type: integer".to_string()
    }
}

fn is_integer(num: &Number) -> bool {
    num.is_u64() || num.is_i64() || num.as_f64().expect("Always valid").fract() == 0.
}

#[inline]
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
