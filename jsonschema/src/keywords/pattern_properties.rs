use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{no_error, CompilationError, ErrorIterator},
    keywords::{format_validators, CompilationResult, Validators},
    paths::InstancePath,
    validator::Validate,
};
use fancy_regex::Regex;
use serde_json::{Map, Value};

pub(crate) struct PatternPropertiesValidator {
    patterns: Vec<(Regex, Validators)>,
}

impl PatternPropertiesValidator {
    #[inline]
    pub(crate) fn compile(
        map: &Map<String, Value>,
        context: &CompilationContext,
    ) -> CompilationResult {
        let mut patterns = Vec::with_capacity(map.len());
        for (pattern, subschema) in map {
            patterns.push((
                Regex::new(pattern)?,
                compile_validators(subschema, context)?,
            ));
        }
        Ok(Box::new(PatternPropertiesValidator { patterns }))
    }
}

impl Validate for PatternPropertiesValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            self.patterns.iter().all(move |(re, validators)| {
                item.iter()
                    .filter(move |(key, _)| re.is_match(key).unwrap_or(false))
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

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            let errors: Vec<_> = self
                .patterns
                .iter()
                .flat_map(move |(re, validators)| {
                    item.iter()
                        .filter(move |(key, _)| re.is_match(key).unwrap_or(false))
                        .flat_map(move |(key, value)| {
                            let instance_path = instance_path.push(key.to_owned());
                            validators.iter().flat_map(move |validator| {
                                validator.validate(schema, value, &instance_path)
                            })
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

pub(crate) struct SingleValuePatternPropertiesValidator {
    pattern: Regex,
    validators: Validators,
}

impl SingleValuePatternPropertiesValidator {
    #[inline]
    pub(crate) fn compile(
        pattern: &str,
        schema: &Value,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(SingleValuePatternPropertiesValidator {
            pattern: Regex::new(pattern)?,
            validators: compile_validators(schema, context)?,
        }))
    }
}

impl Validate for SingleValuePatternPropertiesValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            item.iter()
                .filter(move |(key, _)| self.pattern.is_match(key).unwrap_or(false))
                .all(move |(_key, value)| {
                    self.validators
                        .iter()
                        .all(move |validator| validator.is_valid(schema, value))
                })
        } else {
            true
        }
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            let errors: Vec<_> = item
                .iter()
                .filter(move |(key, _)| self.pattern.is_match(key).unwrap_or(false))
                .flat_map(move |(key, value)| {
                    let instance_path = instance_path.push(key.to_owned());
                    self.validators.iter().flat_map(move |validator| {
                        validator.validate(schema, value, &instance_path)
                    })
                })
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl ToString for SingleValuePatternPropertiesValidator {
    fn to_string(&self) -> String {
        format!(
            "patternProperties: {{{}: {}}}",
            self.pattern,
            format_validators(&self.validators)
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
        _ => {
            if let Value::Object(map) = schema {
                if map.len() == 1 {
                    let (key, value) = map.iter().next().expect("Map is not empty");
                    Some(SingleValuePatternPropertiesValidator::compile(
                        key, value, context,
                    ))
                } else {
                    Some(PatternPropertiesValidator::compile(map, context))
                }
            } else {
                Some(Err(CompilationError::SchemaError))
            }
        }
    }
}
