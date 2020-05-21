use super::{CompilationResult, Validate, Validators};
use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::format_validators,
};
use regex::Regex;
use serde_json::{Map, Value};

pub struct AdditionalPropertiesValidator {
    validators: Validators,
}

impl AdditionalPropertiesValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        Ok(Box::new(AdditionalPropertiesValidator {
            validators: compile_validators(schema, context)?,
        }))
    }
}

impl Validate for AdditionalPropertiesValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            let errors: Vec<_> = self
                .validators
                .iter()
                .flat_map(move |validator| {
                    item.values()
                        .flat_map(move |value| validator.validate(schema, value))
                })
                .collect();
            return Box::new(errors.into_iter());
        }
        no_error()
    }

    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            return self.validators.iter().all(move |validator| {
                item.values()
                    .all(move |value| validator.is_valid(schema, value))
            });
        }
        true
    }

    fn name(&self) -> String {
        format!(
            "additionalProperties: {}",
            format_validators(&self.validators)
        )
    }
}
pub struct AdditionalPropertiesFalseValidator {}

impl AdditionalPropertiesFalseValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(AdditionalPropertiesFalseValidator {}))
    }
}

impl Validate for AdditionalPropertiesFalseValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            if let Some((_, value)) = item.iter().next() {
                return error(ValidationError::false_schema(value));
            }
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            return item.iter().next().is_none();
        }
        true
    }

    fn name(&self) -> String {
        "additionalProperties: false".to_string()
    }
}

pub struct AdditionalPropertiesNotEmptyFalseValidator {
    properties: Map<String, Value>,
}

impl AdditionalPropertiesNotEmptyFalseValidator {
    #[inline]
    pub(crate) fn compile(properties: &Value) -> CompilationResult {
        if let Value::Object(properties) = properties {
            return Ok(Box::new(AdditionalPropertiesNotEmptyFalseValidator {
                properties: properties.clone(),
            }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for AdditionalPropertiesNotEmptyFalseValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            for property in item.keys() {
                if !self.properties.contains_key(property) {
                    // No extra properties are allowed
                    let property_value = Value::String(property.to_string());
                    return error(ValidationError::false_schema(&property_value).into_owned());
                }
            }
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            for property in item.keys() {
                if !self.properties.contains_key(property) {
                    // No extra properties are allowed
                    return false;
                }
            }
        }
        true
    }

    fn name(&self) -> String {
        "additionalProperties: false".to_string()
    }
}

pub struct AdditionalPropertiesNotEmptyValidator {
    validators: Validators,
    properties: Map<String, Value>,
}

impl AdditionalPropertiesNotEmptyValidator {
    #[inline]
    pub(crate) fn compile(
        schema: &Value,
        properties: &Value,
        context: &CompilationContext,
    ) -> CompilationResult {
        if let Value::Object(properties) = properties {
            return Ok(Box::new(AdditionalPropertiesNotEmptyValidator {
                properties: properties.clone(),
                validators: compile_validators(schema, context)?,
            }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for AdditionalPropertiesNotEmptyValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(ref item) = instance {
            let errors: Vec<_> = self
                .validators
                .iter()
                .flat_map(move |validator| {
                    item.iter()
                        .filter(move |(property, _)| !self.properties.contains_key(*property))
                        .flat_map(move |(_, value)| validator.validate(schema, value))
                })
                .collect();
            return Box::new(errors.into_iter());
        }
        no_error()
    }

    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(ref item) = instance {
            return self.validators.iter().all(move |validator| {
                item.iter()
                    .filter(move |(property, _)| !self.properties.contains_key(*property))
                    .all(move |(_, value)| validator.is_valid(schema, value))
            });
        }
        true
    }

    fn name(&self) -> String {
        format!(
            "additionalProperties: {}",
            format_validators(&self.validators)
        )
    }
}

pub struct AdditionalPropertiesWithPatternsValidator {
    validators: Validators,
    pattern: Regex,
}

impl AdditionalPropertiesWithPatternsValidator {
    #[inline]
    pub(crate) fn compile(
        schema: &Value,
        pattern: Regex,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(AdditionalPropertiesWithPatternsValidator {
            validators: compile_validators(schema, context)?,
            pattern,
        }))
    }
}

impl Validate for AdditionalPropertiesWithPatternsValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            let errors: Vec<_> = self
                .validators
                .iter()
                .flat_map(move |validator| {
                    item.iter()
                        .filter(move |(property, _)| !self.pattern.is_match(property))
                        .flat_map(move |(_, value)| validator.validate(schema, value))
                })
                .collect();
            return Box::new(errors.into_iter());
        }
        no_error()
    }

    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            return self.validators.iter().all(move |validator| {
                item.iter()
                    .filter(move |(property, _)| !self.pattern.is_match(property))
                    .all(move |(_, value)| validator.is_valid(schema, value))
            });
        }
        true
    }

    fn name(&self) -> String {
        format!(
            "additionalProperties: {}",
            format_validators(&self.validators)
        )
    }
}

pub struct AdditionalPropertiesWithPatternsFalseValidator {
    pattern: Regex,
}

impl AdditionalPropertiesWithPatternsFalseValidator {
    #[inline]
    pub(crate) fn compile(pattern: Regex) -> CompilationResult {
        Ok(Box::new(AdditionalPropertiesWithPatternsFalseValidator {
            pattern,
        }))
    }
}

impl Validate for AdditionalPropertiesWithPatternsFalseValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            for (property, _) in item {
                if !self.pattern.is_match(property) {
                    let property_value = Value::String(property.to_string());
                    return error(ValidationError::false_schema(&property_value).into_owned());
                }
            }
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            for (property, _) in item {
                if !self.pattern.is_match(property) {
                    return false;
                }
            }
        }
        true
    }

    fn name(&self) -> String {
        "additionalProperties: false".to_string()
    }
}

pub struct AdditionalPropertiesWithPatternsNotEmptyValidator {
    validators: Validators,
    properties: Map<String, Value>,
    pattern: Regex,
}

impl AdditionalPropertiesWithPatternsNotEmptyValidator {
    #[inline]
    pub(crate) fn compile(
        schema: &Value,
        properties: &Value,
        pattern: Regex,
        context: &CompilationContext,
    ) -> CompilationResult {
        if let Value::Object(properties) = properties {
            return Ok(Box::new(
                AdditionalPropertiesWithPatternsNotEmptyValidator {
                    validators: compile_validators(schema, context)?,
                    properties: properties.clone(),
                    pattern,
                },
            ));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for AdditionalPropertiesWithPatternsNotEmptyValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            let errors: Vec<_> = self
                .validators
                .iter()
                .flat_map(move |validator| {
                    item.iter()
                        .filter(move |(property, _)| {
                            !self.properties.contains_key(*property)
                                && !self.pattern.is_match(property)
                        })
                        .flat_map(move |(_, value)| validator.validate(schema, value))
                })
                .collect();
            return Box::new(errors.into_iter());
        }
        no_error()
    }

    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            return self.validators.iter().all(move |validator| {
                item.iter()
                    .filter(move |(property, _)| {
                        !self.properties.contains_key(*property) && !self.pattern.is_match(property)
                    })
                    .all(move |(_, value)| validator.is_valid(schema, value))
            });
        }
        true
    }

    fn name(&self) -> String {
        format!(
            "additionalProperties: {}",
            format_validators(&self.validators)
        )
    }
}

pub struct AdditionalPropertiesWithPatternsNotEmptyFalseValidator {
    properties: Map<String, Value>,
    pattern: Regex,
}

impl AdditionalPropertiesWithPatternsNotEmptyFalseValidator {
    #[inline]
    pub(crate) fn compile(properties: &Value, pattern: Regex) -> CompilationResult {
        if let Value::Object(properties) = properties {
            return Ok(Box::new(
                AdditionalPropertiesWithPatternsNotEmptyFalseValidator {
                    properties: properties.clone(),
                    pattern,
                },
            ));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for AdditionalPropertiesWithPatternsNotEmptyFalseValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            for property in item.keys() {
                if !self.properties.contains_key(property) && !self.pattern.is_match(property) {
                    let property_value = Value::String(property.to_string());
                    return error(ValidationError::false_schema(&property_value).into_owned());
                }
            }
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            for property in item.keys() {
                if !self.properties.contains_key(property) && !self.pattern.is_match(property) {
                    return false;
                }
            }
        }
        true
    }

    fn name(&self) -> String {
        "additionalProperties: false".to_string()
    }
}

#[inline]
pub fn compile(
    parent: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    let properties = parent.get("properties");
    if let Some(patterns) = parent.get("patternProperties") {
        if let Value::Object(obj) = patterns {
            let pattern = obj.keys().cloned().collect::<Vec<String>>().join("|");
            return match Regex::new(&pattern) {
                Ok(re) => {
                    match schema {
                        Value::Bool(true) => None, // "additionalProperties" are "true" by default
                        Value::Bool(false) => match properties {
                            Some(properties) => Some(
                                AdditionalPropertiesWithPatternsNotEmptyFalseValidator::compile(
                                    properties, re,
                                ),
                            ),
                            None => {
                                Some(AdditionalPropertiesWithPatternsFalseValidator::compile(re))
                            }
                        },
                        _ => match properties {
                            Some(properties) => {
                                Some(AdditionalPropertiesWithPatternsNotEmptyValidator::compile(
                                    schema, properties, re, context,
                                ))
                            }
                            None => Some(AdditionalPropertiesWithPatternsValidator::compile(
                                schema, re, context,
                            )),
                        },
                    }
                }
                Err(_) => Some(Err(CompilationError::SchemaError)),
            };
        }
        Some(Err(CompilationError::SchemaError))
    } else {
        match schema {
            Value::Bool(true) => None, // "additionalProperties" are "true" by default
            Value::Bool(false) => match properties {
                Some(properties) => Some(AdditionalPropertiesNotEmptyFalseValidator::compile(
                    properties,
                )),
                None => Some(AdditionalPropertiesFalseValidator::compile()),
            },
            _ => match properties {
                Some(properties) => Some(AdditionalPropertiesNotEmptyValidator::compile(
                    schema, properties, context,
                )),
                None => Some(AdditionalPropertiesValidator::compile(schema, context)),
            },
        }
    }
}
