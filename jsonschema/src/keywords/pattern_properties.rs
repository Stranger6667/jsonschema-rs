use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{no_error, CompilationError, ErrorIterator},
    keywords::{format_validators, CompilationResult, Validators},
    validator::Validate,
};
use regex::Regex;
use serde_json::{Map, Value};

pub(crate) struct PatternPropertiesValidator {
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
            Ok(Box::new(PatternPropertiesValidator { patterns }))
        } else {
            Err(CompilationError::SchemaError)
        }
    }
}

impl Validate for PatternPropertiesValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            self.patterns.iter().all(move |(re, validators)| {
                item.iter()
                    .filter(move |(key, _)| re.is_match(key))
                    .all(move |(_key, value)| {
                        validators
                            .iter()
                            .all(move |validator| validator.is_valid(schema, value))
                    })
            })
        } else {
            true
        }
    }

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
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl ToString for PatternPropertiesValidator {
    fn to_string(&self) -> String {
        format!(
            "patternProperties: {{{}}}",
            self.patterns
                .iter()
                .map(|(key, validators)| { format!("{}: {}", key, format_validators(validators)) })
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[inline]
pub(crate) fn compile(
    parent: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    match parent.get("additionalProperties") {
        // This type of `additionalProperties` validator handles `patternProperties` logic
        Some(Value::Bool(false)) | Some(Value::Object(_)) => None,
        _ => Some(PatternPropertiesValidator::compile(schema, context)),
    }
}
