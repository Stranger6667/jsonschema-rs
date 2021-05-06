use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::{helpers, CompilationResult},
    paths::InstancePath,
    primitive_type::{PrimitiveType, PrimitiveTypesBitMap},
    validator::Validate,
};
use serde_json::{Map, Value};

#[derive(Debug)]
pub(crate) struct EnumValidator {
    options: Value,
    // Types that occur in items
    types: PrimitiveTypesBitMap,
    items: Vec<Value>,
}

impl EnumValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, items: &[Value]) -> CompilationResult {
        let mut types = PrimitiveTypesBitMap::new();
        for item in items.iter() {
            types |= PrimitiveType::from(item);
        }
        Ok(Box::new(EnumValidator {
            options: schema.clone(),
            items: items.to_vec(),
            types,
        }))
    }
}

impl Validate for EnumValidator {
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::enumeration(
                instance_path.into(),
                instance,
                &self.options,
            ))
        }
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        // If the input value type is not in the types present among the enum options, then there
        // is no reason to compare it against all items - we know that
        // there are no items with such type at all
        if self.types.contains_type(PrimitiveType::from(instance)) {
            self.items.iter().any(|item| helpers::equal(instance, item))
        } else {
            false
        }
    }
}

impl ToString for EnumValidator {
    fn to_string(&self) -> String {
        format!(
            "enum: [{}]",
            self.items
                .iter()
                .map(Value::to_string)
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Debug)]
pub(crate) struct SingleValueEnumValidator {
    value: Value,
    options: Value,
}

impl SingleValueEnumValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, value: &Value) -> CompilationResult {
        Ok(Box::new(SingleValueEnumValidator {
            options: schema.clone(),
            value: value.clone(),
        }))
    }
}

impl Validate for SingleValueEnumValidator {
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::enumeration(
                instance_path.into(),
                instance,
                &self.options,
            ))
        }
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        helpers::equal(&self.value, instance)
    }
}

impl ToString for SingleValueEnumValidator {
    fn to_string(&self) -> String {
        format!("enum: [{}]", self.value)
    }
}

#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    if let Value::Array(items) = schema {
        if items.len() == 1 {
            let value = items.iter().next().expect("Vec is not empty");
            Some(SingleValueEnumValidator::compile(schema, value))
        } else {
            Some(EnumValidator::compile(schema, items))
        }
    } else {
        Some(Err(CompilationError::SchemaError))
    }
}
