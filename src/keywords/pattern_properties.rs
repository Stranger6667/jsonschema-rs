use super::{CompilationResult, Validate, Validators};
use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{no_error, CompilationError, ErrorIterator},
};
use regex::Regex;
use serde_json::{Map, Value};

pub struct PatternPropertiesValidator {
    patterns: Vec<(Regex, Validators)>,
}

impl PatternPropertiesValidator {
    #[inline]
    pub(crate) fn compile(properties: &Value, context: &CompilationContext) -> CompilationResult {
        if let Value::Object(map) = properties {
            let mut patterns = Vec::with_capacity(map.len());
            for (pattern, subschema) in map {
                patterns.push((
                    Regex::new(pattern)?,
                    compile_validators(subschema, context)?,
                ));
            }
            return Ok(Box::new(PatternPropertiesValidator { patterns }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for PatternPropertiesValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            let errors: Vec<_> = self
                .patterns
                .iter()
                .flat_map(move |(re, validators)| {
                    item.iter()
                        .filter(move |(key, _)| re.is_match(key))
                        .flat_map(move |(_key, value)| {
                            validators
                                .iter()
                                .flat_map(move |validator| validator.validate(schema, value))
                        })
                })
                .collect();
            return Box::new(errors.into_iter());
        }
        no_error()
    }

    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            return self.patterns.iter().all(move |(re, validators)| {
                item.iter()
                    .filter(move |(key, _)| re.is_match(key))
                    .all(move |(_key, value)| {
                        validators
                            .iter()
                            .all(move |validator| validator.is_valid(schema, value))
                    })
            });
        }
        true
    }

    fn name(&self) -> String {
        format!("<pattern properties: {:?}>", self.patterns)
    }
}

#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(PatternPropertiesValidator::compile(schema, context))
}
