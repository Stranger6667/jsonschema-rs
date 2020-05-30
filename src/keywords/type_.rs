use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::CompilationResult,
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
    fn is_valid_number(&self, _: &JSONSchema, _: &Value, instance_value: f64) -> bool {
        self.types.contains_type(PrimitiveType::Number)
            || (self.types.contains_type(PrimitiveType::Integer) && instance_value.fract() == 0.)
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

pub struct NullTypeValidator {}

impl NullTypeValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(NullTypeValidator {}))
    }
}

impl Validate for NullTypeValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::single_type_error(instance, PrimitiveType::Null)
    }

    fn name(&self) -> String {
        "type: null".to_string()
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
    fn is_valid_object(&self, _: &JSONSchema, _: &Value, _: &Map<String, Value>) -> bool {
        false
    }
    #[inline]
    fn is_valid_number(&self, _: &JSONSchema, _: &Value, _: f64) -> bool {
        false
    }
    #[inline]
    fn is_valid_signed_integer(&self, _: &JSONSchema, _: &Value, _: i64) -> bool {
        false
    }
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, _: &str) -> bool {
        false
    }
    #[inline]
    fn is_valid_unsigned_integer(&self, _: &JSONSchema, _: &Value, _: u64) -> bool {
        false
    }
    #[inline]
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Null = instance {
            true
        } else {
            false
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Null = instance {
            self.validate_null(schema, instance, ())
        } else {
            error(self.build_validation_error(instance))
        }
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
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::single_type_error(instance, PrimitiveType::Boolean)
    }

    fn name(&self) -> String {
        "type: boolean".to_string()
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, _: &[Value]) -> bool {
        false
    }
    #[inline]
    fn is_valid_object(&self, _: &JSONSchema, _: &Value, _: &Map<String, Value>) -> bool {
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
    fn is_valid_signed_integer(&self, _: &JSONSchema, _: &Value, _: i64) -> bool {
        false
    }
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, _: &str) -> bool {
        false
    }
    #[inline]
    fn is_valid_unsigned_integer(&self, _: &JSONSchema, _: &Value, _: u64) -> bool {
        false
    }
    #[inline]
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Bool(_) = instance {
            true
        } else {
            false
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Bool(instance_value) = instance {
            self.validate_boolean(schema, instance, *instance_value)
        } else {
            error(self.build_validation_error(instance))
        }
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
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::single_type_error(instance, PrimitiveType::String)
    }

    fn name(&self) -> String {
        "type: string".to_string()
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
    fn is_valid_object(&self, _: &JSONSchema, _: &Value, _: &Map<String, Value>) -> bool {
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
    fn is_valid_signed_integer(&self, _: &JSONSchema, _: &Value, _: i64) -> bool {
        false
    }
    #[inline]
    fn is_valid_unsigned_integer(&self, _: &JSONSchema, _: &Value, _: u64) -> bool {
        false
    }
    #[inline]
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(_) = instance {
            true
        } else {
            false
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(instance_value) = instance {
            self.validate_string(schema, instance, instance_value)
        } else {
            error(self.build_validation_error(instance))
        }
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
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::single_type_error(instance, PrimitiveType::Array)
    }

    fn name(&self) -> String {
        "type: array".to_string()
    }

    #[inline]
    fn is_valid_boolean(&self, _: &JSONSchema, _: &Value, _: bool) -> bool {
        false
    }
    #[inline]
    fn is_valid_object(&self, _: &JSONSchema, _: &Value, _: &Map<String, Value>) -> bool {
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
    fn is_valid_signed_integer(&self, _: &JSONSchema, _: &Value, _: i64) -> bool {
        false
    }
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, _: &str) -> bool {
        false
    }
    #[inline]
    fn is_valid_unsigned_integer(&self, _: &JSONSchema, _: &Value, _: u64) -> bool {
        false
    }
    #[inline]
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(_) = instance {
            true
        } else {
            false
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Array(instance_value) = instance {
            self.validate_array(schema, instance, instance_value)
        } else {
            error(self.build_validation_error(instance))
        }
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
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::single_type_error(instance, PrimitiveType::Object)
    }

    fn name(&self) -> String {
        "type: object".to_string()
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
    fn is_valid_signed_integer(&self, _: &JSONSchema, _: &Value, _: i64) -> bool {
        false
    }
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, _: &str) -> bool {
        false
    }
    #[inline]
    fn is_valid_unsigned_integer(&self, _: &JSONSchema, _: &Value, _: u64) -> bool {
        false
    }
    #[inline]
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(_) = instance {
            true
        } else {
            false
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(instance_value) = instance {
            self.validate_object(schema, instance, instance_value)
        } else {
            error(self.build_validation_error(instance))
        }
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
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::single_type_error(instance, PrimitiveType::Number)
    }

    fn name(&self) -> String {
        "type: number".to_string()
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
    fn is_valid_object(&self, _: &JSONSchema, _: &Value, _: &Map<String, Value>) -> bool {
        false
    }
    #[inline]
    fn is_valid_null(&self, _: &JSONSchema, _: &Value, _: ()) -> bool {
        false
    }
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, _: &str) -> bool {
        false
    }
    #[inline]
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Number(_) = instance {
            true
        } else {
            false
        }
    }

    #[inline]
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Number(_) = instance {
            no_error()
        } else {
            error(self.build_validation_error(instance))
        }
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

    fn name(&self) -> String {
        "type: integer".to_string()
    }

    #[inline]
    fn is_valid_number(&self, _: &JSONSchema, _: &Value, instance_value: f64) -> bool {
        instance_value.fract() == 0.
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
    fn is_valid_object(&self, _: &JSONSchema, _: &Value, _: &Map<String, Value>) -> bool {
        false
    }
    #[inline]
    fn is_valid_null(&self, _: &JSONSchema, _: &Value, _: ()) -> bool {
        false
    }
    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, _: &str) -> bool {
        false
    }
    #[inline]
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Number(instance_number) = instance {
            instance_number.is_u64()
                || instance_number.is_i64()
                || instance_number.as_f64().map_or(false, |f| f.fract() == 0.)
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
