use super::{CompilationResult, ValidationResult};
use super::{Validate, Validators};
use crate::context::CompilationContext;
use crate::error::CompilationError;
use crate::validator::compile_validators;
use crate::JSONSchema;
use regex::Regex;
use serde_json::{Map, Value};

pub struct PatternPropertiesValidator<'a> {
    patterns: Vec<(Regex, Validators<'a>)>,
}

impl<'a> PatternPropertiesValidator<'a> {
    pub(crate) fn compile(
        properties: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        match properties.as_object() {
            Some(map) => {
                let mut patterns = Vec::with_capacity(map.len());
                for (pattern, subschema) in map {
                    patterns.push((
                        Regex::new(pattern)?,
                        compile_validators(subschema, context)?,
                    ));
                }
                Ok(Box::new(PatternPropertiesValidator { patterns }))
            }
            None => Err(CompilationError::SchemaError),
        }
    }
}

impl<'a> Validate<'a> for PatternPropertiesValidator<'a> {
    fn validate(&self, config: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Object(item) = instance {
            for (re, validators) in self.patterns.iter() {
                for (key, value) in item {
                    if re.is_match(key) {
                        for validator in validators.iter() {
                            validator.validate(config, value)?
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<pattern properties: {:?}>", self.patterns)
    }
}

pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(PatternPropertiesValidator::compile(schema, context))
}
