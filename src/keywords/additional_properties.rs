use super::{CompilationResult, ValidationResult};
use super::{Validate, Validators};
use crate::context::CompilationContext;
use crate::error::CompilationError;
use crate::validator::compile_validators;
use crate::{JSONSchema, ValidationError};
use regex::Regex;
use serde_json::{Map, Value};

pub struct AdditionalPropertiesValidator<'a> {
    validators: Validators<'a>,
}

impl<'a> AdditionalPropertiesValidator<'a> {
    pub(crate) fn compile(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        Ok(Box::new(AdditionalPropertiesValidator {
            validators: compile_validators(schema, context)?,
        }))
    }
}

impl<'a> Validate<'a> for AdditionalPropertiesValidator<'a> {
    fn validate(&self, schema: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Object(item) = instance {
            for value in item.values() {
                for validator in self.validators.iter() {
                    validator.validate(schema, value)?
                }
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<additional properties: {:?}>", self.validators)
    }
}
pub struct AdditionalPropertiesFalseValidator {}

impl<'a> AdditionalPropertiesFalseValidator {
    pub(crate) fn compile() -> CompilationResult<'a> {
        Ok(Box::new(AdditionalPropertiesFalseValidator {}))
    }
}

impl<'a> Validate<'a> for AdditionalPropertiesFalseValidator {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Object(item) = instance {
            if let Some((_, value)) = item.iter().next() {
                return Err(ValidationError::false_schema(value.clone()));
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        "<additional properties: false>".to_string()
    }
}

pub struct AdditionalPropertiesNotEmptyFalseValidator<'a> {
    properties: &'a Map<String, Value>,
}

impl<'a> AdditionalPropertiesNotEmptyFalseValidator<'a> {
    pub(crate) fn compile(properties: &'a Value) -> CompilationResult<'a> {
        if let Value::Object(properties) = properties {
            return Ok(Box::new(AdditionalPropertiesNotEmptyFalseValidator {
                properties,
            }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl<'a> Validate<'a> for AdditionalPropertiesNotEmptyFalseValidator<'a> {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Object(item) = instance {
            for property in item.keys() {
                if !self.properties.contains_key(property) {
                    // No extra properties are allowed
                    return Err(ValidationError::false_schema(Value::String(
                        property.to_string(),
                    )));
                }
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        "<additional properties: false>".to_string()
    }
}

pub struct AdditionalPropertiesNotEmptyValidator<'a> {
    validators: Validators<'a>,
    properties: &'a Map<String, Value>,
}

impl<'a> AdditionalPropertiesNotEmptyValidator<'a> {
    pub(crate) fn compile(
        schema: &'a Value,
        properties: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        if let Value::Object(properties) = properties {
            return Ok(Box::new(AdditionalPropertiesNotEmptyValidator {
                properties,
                validators: compile_validators(schema, context)?,
            }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl<'a> Validate<'a> for AdditionalPropertiesNotEmptyValidator<'a> {
    fn validate(&self, schema: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Object(item) = instance {
            for (property, value) in item {
                if !self.properties.contains_key(property) {
                    for validator in self.validators.iter() {
                        validator.validate(schema, value)?
                    }
                }
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<additional properties: {:?}>", self.validators)
    }
}

pub struct AdditionalPropertiesWithPatternsValidator<'a> {
    validators: Validators<'a>,
    pattern: Regex,
}

impl<'a> AdditionalPropertiesWithPatternsValidator<'a> {
    pub(crate) fn compile(
        schema: &'a Value,
        pattern: Regex,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        Ok(Box::new(AdditionalPropertiesWithPatternsValidator {
            validators: compile_validators(schema, context)?,
            pattern,
        }))
    }
}

impl<'a> Validate<'a> for AdditionalPropertiesWithPatternsValidator<'a> {
    fn validate(&self, schema: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Object(item) = instance {
            for (property, value) in item {
                if !self.pattern.is_match(property) {
                    for validator in self.validators.iter() {
                        validator.validate(schema, value)?
                    }
                }
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<additional properties: {:?}>", self.validators)
    }
}

pub struct AdditionalPropertiesWithPatternsFalseValidator {
    pattern: Regex,
}

impl<'a> AdditionalPropertiesWithPatternsFalseValidator {
    pub(crate) fn compile(pattern: Regex) -> CompilationResult<'a> {
        Ok(Box::new(AdditionalPropertiesWithPatternsFalseValidator {
            pattern,
        }))
    }
}

impl<'a> Validate<'a> for AdditionalPropertiesWithPatternsFalseValidator {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Object(item) = instance {
            for (property, _) in item {
                if !self.pattern.is_match(property) {
                    return Err(ValidationError::false_schema(Value::String(
                        property.to_string(),
                    )));
                }
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        "<additional properties: false>".to_string()
    }
}

pub struct AdditionalPropertiesWithPatternsNotEmptyValidator<'a> {
    validators: Validators<'a>,
    properties: &'a Map<String, Value>,
    pattern: Regex,
}

impl<'a> AdditionalPropertiesWithPatternsNotEmptyValidator<'a> {
    pub(crate) fn compile(
        schema: &'a Value,
        properties: &'a Value,
        pattern: Regex,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        if let Value::Object(properties) = properties {
            return Ok(Box::new(
                AdditionalPropertiesWithPatternsNotEmptyValidator {
                    validators: compile_validators(schema, context)?,
                    properties,
                    pattern,
                },
            ));
        }
        Err(CompilationError::SchemaError)
    }
}

impl<'a> Validate<'a> for AdditionalPropertiesWithPatternsNotEmptyValidator<'a> {
    fn validate(&self, schema: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Object(item) = instance {
            for (property, value) in item {
                if !self.properties.contains_key(property) && !self.pattern.is_match(property) {
                    for validator in self.validators.iter() {
                        validator.validate(schema, value)?
                    }
                }
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<additional properties: {:?}>", self.validators)
    }
}

pub struct AdditionalPropertiesWithPatternsNotEmptyFalseValidator<'a> {
    properties: &'a Map<String, Value>,
    pattern: Regex,
}

impl<'a> AdditionalPropertiesWithPatternsNotEmptyFalseValidator<'a> {
    pub(crate) fn compile(properties: &'a Value, pattern: Regex) -> CompilationResult<'a> {
        if let Value::Object(properties) = properties {
            return Ok(Box::new(
                AdditionalPropertiesWithPatternsNotEmptyFalseValidator {
                    properties,
                    pattern,
                },
            ));
        }
        Err(CompilationError::SchemaError)
    }
}

impl<'a> Validate<'a> for AdditionalPropertiesWithPatternsNotEmptyFalseValidator<'a> {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Object(item) = instance {
            for property in item.keys() {
                if !self.properties.contains_key(property) && !self.pattern.is_match(property) {
                    return Err(ValidationError::false_schema(Value::String(
                        property.to_string(),
                    )));
                }
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        "<additional properties: false>".to_string()
    }
}

pub(crate) fn compile<'a>(
    parent: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
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
