use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{type_, CompilationResult},
    paths::{InstancePath, JSONPointer},
    primitive_type::{PrimitiveType, PrimitiveTypesBitMap},
    validator::Validate,
};
use serde_json::{Map, Number, Value};
use std::convert::TryFrom;

pub(crate) struct MultipleTypesValidator {
    types: PrimitiveTypesBitMap,
    schema_path: JSONPointer,
}

impl MultipleTypesValidator {
    #[inline]
    pub(crate) fn compile(items: &[Value], schema_path: JSONPointer) -> CompilationResult {
        let mut types = PrimitiveTypesBitMap::new();
        for item in items {
            match item {
                Value::String(string) => {
                    if let Ok(primitive_type) = PrimitiveType::try_from(string.as_str()) {
                        types |= primitive_type;
                    } else {
                        return Err(ValidationError::schema(item));
                    }
                }
                _ => return Err(ValidationError::schema(item)),
            }
        }
        Ok(Box::new(MultipleTypesValidator { types, schema_path }))
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
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::multiple_type_error(
                self.schema_path.clone(),
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

pub(crate) struct IntegerTypeValidator {
    schema_path: JSONPointer,
}

impl IntegerTypeValidator {
    #[inline]
    pub(crate) fn compile<'a>(schema_path: JSONPointer) -> CompilationResult<'a> {
        Ok(Box::new(IntegerTypeValidator { schema_path }))
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

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::single_type_error(
                self.schema_path.clone(),
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
    num.is_u64() || num.is_i64()
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    let schema_path = context.as_pointer_with("type");
    match schema {
        Value::String(item) => compile_single_type(item.as_str(), schema_path),
        Value::Array(items) => {
            if items.len() == 1 {
                if let Some(Value::String(item)) = items.iter().next() {
                    compile_single_type(item.as_str(), schema_path)
                } else {
                    Some(Err(ValidationError::schema(schema)))
                }
            } else {
                Some(MultipleTypesValidator::compile(items, schema_path))
            }
        }
        _ => Some(Err(ValidationError::schema(schema))),
    }
}

fn compile_single_type<'a>(item: &str, schema_path: JSONPointer) -> Option<CompilationResult<'a>> {
    match PrimitiveType::try_from(item) {
        Ok(PrimitiveType::Array) => Some(type_::ArrayTypeValidator::compile(schema_path)),
        Ok(PrimitiveType::Boolean) => Some(type_::BooleanTypeValidator::compile(schema_path)),
        Ok(PrimitiveType::Integer) => Some(IntegerTypeValidator::compile(schema_path)),
        Ok(PrimitiveType::Null) => Some(type_::NullTypeValidator::compile(schema_path)),
        Ok(PrimitiveType::Number) => Some(type_::NumberTypeValidator::compile(schema_path)),
        Ok(PrimitiveType::Object) => Some(type_::ObjectTypeValidator::compile(schema_path)),
        Ok(PrimitiveType::String) => Some(type_::StringTypeValidator::compile(schema_path)),
        Err(()) => Some(Err(ValidationError::null_schema())),
    }
}
