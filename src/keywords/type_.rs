use super::{CompilationResult, Validate};
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
        let instance_primtive_type = PrimitiveType::from(instance);
        for type_ in self.types.iter() {
            if type_ == &instance_primtive_type {
                return true;
            }
        }
        false
    }

    fn name(&self) -> String {
        format!("<type: {:?}>", self.types)
    }
}

pub struct NullTypeValidator {}

impl NullTypeValidator {
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
        "<type: null>".to_string()
    }
}

pub struct BooleanTypeValidator {}

impl BooleanTypeValidator {
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
        "<type: boolean>".to_string()
    }
}

pub struct StringTypeValidator {}

impl StringTypeValidator {
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
        "<type: string>".to_string()
    }
}

pub struct ArrayTypeValidator {}

impl ArrayTypeValidator {
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
        "<type: array>".to_string()
    }
}

pub struct ObjectTypeValidator {}

impl ObjectTypeValidator {
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
        "<type: object>".to_string()
    }
}

pub struct NumberTypeValidator {}

impl NumberTypeValidator {
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
        "<type: number>".to_string()
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
        instance.is_u64()
            || instance.is_i64()
            || instance.as_f64().map(|f| f.fract() == 0.).unwrap_or(false)
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
            if let Some(item) = items.iter().next().unwrap().as_str() {
                compile_single_type(item)
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
    match PrimitiveType::try_from(item) {
        Ok(PrimitiveType::Integer) => Some(IntegerTypeValidator::compile()),
        Ok(PrimitiveType::Null) => Some(NullTypeValidator::compile()),
        Ok(PrimitiveType::Boolean) => Some(BooleanTypeValidator::compile()),
        Ok(PrimitiveType::String) => Some(StringTypeValidator::compile()),
        Ok(PrimitiveType::Array) => Some(ArrayTypeValidator::compile()),
        Ok(PrimitiveType::Object) => Some(ObjectTypeValidator::compile()),
        Ok(PrimitiveType::Number) => Some(NumberTypeValidator::compile()),
        Err(_) => Some(Err(CompilationError::SchemaError)),
    }
}
