use crate::{
    compiler,
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{type_, CompilationResult},
    paths::{JsonPointer, JsonPointerNode},
    primitive_type::{PrimitiveType, PrimitiveTypesBitMap},
    validator::Validate,
};
use serde_json::{json, Map, Number, Value};
use std::convert::TryFrom;

pub(crate) struct MultipleTypesValidator {
    types: PrimitiveTypesBitMap,
    schema_path: JsonPointer,
}

impl MultipleTypesValidator {
    #[inline]
    pub(crate) fn compile(items: &[Value], schema_path: JsonPointer) -> CompilationResult {
        let mut types = PrimitiveTypesBitMap::new();
        for item in items {
            match item {
                Value::String(string) => {
                    if let Ok(primitive_type) = PrimitiveType::try_from(string.as_str()) {
                        types |= primitive_type;
                    } else {
                        return Err(ValidationError::enumeration(
                            JsonPointer::default(),
                            schema_path,
                            item,
                            &json!([
                                "array", "boolean", "integer", "null", "number", "object", "string"
                            ]),
                        ));
                    }
                }
                _ => {
                    return Err(ValidationError::single_type_error(
                        JsonPointer::default(),
                        schema_path,
                        item,
                        PrimitiveType::String,
                    ))
                }
            }
        }
        Ok(Box::new(MultipleTypesValidator { types, schema_path }))
    }
}

impl Validate for MultipleTypesValidator {
    fn is_valid(&self, instance: &Value) -> bool {
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
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if self.is_valid(instance) {
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

pub(crate) struct IntegerTypeValidator {
    schema_path: JsonPointer,
}

impl IntegerTypeValidator {
    #[inline]
    pub(crate) fn compile<'a>(schema_path: JsonPointer) -> CompilationResult<'a> {
        Ok(Box::new(IntegerTypeValidator { schema_path }))
    }
}

impl Validate for IntegerTypeValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Number(num) = instance {
            is_integer(num)
        } else {
            false
        }
    }

    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if self.is_valid(instance) {
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

fn is_integer(num: &Number) -> bool {
    num.is_u64() || num.is_i64()
}

#[inline]
pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    _: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    let schema_path = ctx.as_pointer_with("type");
    match schema {
        Value::String(item) => compile_single_type(item.as_str(), schema_path),
        Value::Array(items) => {
            if items.len() == 1 {
                let item = &items[0];
                if let Value::String(item) = item {
                    compile_single_type(item.as_str(), schema_path)
                } else {
                    Some(Err(ValidationError::single_type_error(
                        JsonPointer::default(),
                        schema_path,
                        item,
                        PrimitiveType::String,
                    )))
                }
            } else {
                Some(MultipleTypesValidator::compile(items, schema_path))
            }
        }
        _ => Some(Err(ValidationError::multiple_type_error(
            JsonPointer::default(),
            ctx.clone().into_pointer(),
            schema,
            PrimitiveTypesBitMap::new()
                .add_type(PrimitiveType::String)
                .add_type(PrimitiveType::Array),
        ))),
    }
}

fn compile_single_type<'a>(item: &str, schema_path: JsonPointer) -> Option<CompilationResult<'a>> {
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
