use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{CompilationError, ErrorIterator},
    keywords::{format_validators, CompilationResult, Validators},
    validator::Validate,
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
    fn name(&self) -> String {
        format!(
            "patternProperties: {{{}}}",
            self.patterns
                .iter()
                .map(|(key, validators)| { format!("{}: {}", key, format_validators(validators)) })
                .collect::<Vec<String>>()
                .join(", ")
        )
    }

    #[inline]
    fn is_valid_object(
        &self,
        schema: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        self.patterns.iter().all(|(re, validators)| {
            instance_value
                .iter()
                .filter(|(key, _)| re.is_match(key))
                .all(|(_key, value)| {
                    validators
                        .iter()
                        .all(|validator| validator.is_valid(schema, value))
                })
        })
    }

    #[inline]
    fn validate_object<'a>(
        &self,
        schema: &'a JSONSchema,
        _: &'a Value,
        instance_value: &'a Map<String, Value>,
    ) -> ErrorIterator<'a> {
        Box::new(
            self.patterns
                .iter()
                .flat_map(|(re, validators)| {
                    instance_value
                        .iter()
                        .filter(move |(key, _)| re.is_match(key))
                        .flat_map(move |(_key, value)| {
                            validators
                                .iter()
                                .flat_map(move |validator| validator.validate(schema, value))
                        })
                })
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(PatternPropertiesValidator::compile(schema, context))
}
