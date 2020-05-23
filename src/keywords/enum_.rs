use super::{helpers, CompilationResult, Validate};
use crate::primitive_type::{PrimitiveType, PrimitiveTypesBitMap};
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
};
use serde_json::{Map, Value};

pub struct NumberEnumValidator {
    options: Value,
    variants: Vec<f64>,
}

impl NumberEnumValidator {
    #[inline]
    pub(crate) fn compile(options: &Value, variants: Vec<f64>) -> CompilationResult {
        Ok(Box::new(NumberEnumValidator {
            options: options.clone(),
            variants,
        }))
    }
}

impl Validate for NumberEnumValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if !self.is_valid(schema, instance) {
            return error(ValidationError::enumeration(instance, &self.options));
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Some(value) = instance.as_f64() {
            self.variants.iter().any(|variant| *variant == value)
        } else {
            false
        }
    }

    fn name(&self) -> String {
        format!(
            "enum: [{}]",
            self.options
                .as_array()
                .expect("Always array")
                .iter()
                .map(|item| format!("{}", item))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

pub struct StringEnumValidator {
    options: Value,
    variants: Vec<String>,
}

impl StringEnumValidator {
    #[inline]
    pub(crate) fn compile(options: &Value, variants: Vec<String>) -> CompilationResult {
        Ok(Box::new(StringEnumValidator {
            options: options.clone(),
            variants,
        }))
    }
}

impl Validate for StringEnumValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if !self.is_valid(schema, instance) {
            return error(ValidationError::enumeration(instance, &self.options));
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Some(value) = instance.as_str() {
            self.variants.iter().any(|variant| variant == value)
        } else {
            false
        }
    }

    fn name(&self) -> String {
        format!(
            "enum: [{}]",
            self.options
                .as_array()
                .expect("Always array")
                .iter()
                .map(|item| format!("{}", item))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

pub struct EnumValidator {
    options: Value,
    items: Vec<Value>,
}

impl EnumValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Value::Array(items) = schema {
            return Ok(Box::new(EnumValidator {
                options: schema.clone(),
                items: items.clone(),
            }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for EnumValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if !self.is_valid(schema, instance) {
            return error(ValidationError::enumeration(instance, &self.options));
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        self.items.iter().any(|item| helpers::equal(instance, item))
    }

    fn name(&self) -> String {
        format!(
            "enum: [{}]",
            self.items
                .iter()
                .map(|item| format!("{}", item))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    if let Value::Array(items) = schema {
        let mut types = PrimitiveTypesBitMap::new();
        let mut numbers = vec![];
        let mut strings = vec![];
        for item in items {
            match item {
                Value::Array(_) => types |= PrimitiveType::Array,
                Value::Bool(_) => types |= PrimitiveType::Boolean,
                Value::Null => types |= PrimitiveType::Null,
                Value::Number(value) => {
                    types |= PrimitiveType::Number;
                    numbers.push(value.as_f64().expect("Always can be f64"))
                }
                Value::Object(_) => types |= PrimitiveType::Object,
                Value::String(value) => {
                    types |= PrimitiveType::String;
                    strings.push(value.clone())
                }
            }
        }
        let t: Vec<PrimitiveType> = types.into_iter().collect();
        return if t.len() == 1 {
            match t.iter().next().expect("This vector has one element") {
                PrimitiveType::Number => Some(NumberEnumValidator::compile(schema, numbers)),
                PrimitiveType::String => Some(StringEnumValidator::compile(schema, strings)),
                _ => Some(EnumValidator::compile(schema)),
            }
        } else {
            Some(EnumValidator::compile(schema))
        };
    };
    Some(Err(CompilationError::SchemaError))
}
