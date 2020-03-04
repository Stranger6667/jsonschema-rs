use super::Validate;
use super::{CompilationResult, ValidationResult};
use crate::context::CompilationContext;
use crate::error::{CompilationError, ValidationError};
use crate::JSONSchema;
use regex::{Captures, Regex};
use serde_json::{Map, Value};
use std::ops::Index;

lazy_static! {
    static ref CONTROL_GROUPS_RE: Regex = Regex::new(r"\\c[A-Za-z]").unwrap();
}

pub struct PatternValidator<'a> {
    original: &'a String,
    pattern: Regex,
}

impl<'a> PatternValidator<'a> {
    pub(crate) fn compile(pattern: &'a Value) -> CompilationResult<'a> {
        match pattern {
            Value::String(item) => {
                let pattern = convert_regex(item)?;
                Ok(Box::new(PatternValidator {
                    original: item,
                    pattern,
                }))
            }
            _ => Err(CompilationError::SchemaError),
        }
    }
}

impl<'a> Validate<'a> for PatternValidator<'a> {
    fn validate(&self, _: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::String(item) = instance {
            if !self.pattern.is_match(item) {
                return Err(ValidationError::pattern(
                    item.clone(),
                    self.original.clone(),
                ));
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<pattern: {}>", self.pattern)
    }
}

// ECMA 262 has differences
fn convert_regex(pattern: &str) -> Result<Regex, regex::Error> {
    // replace control chars
    let new_pattern = CONTROL_GROUPS_RE.replace_all(pattern, replace_control_group);
    Regex::new(
        &new_pattern
            .replace(r"\d", "[0-9]")
            .replace(r"\D", "[^0-9]")
            .replace(r"\w", "[A-Za-z]")
            .replace(r"\W", "[^A-Za-z]")
            .replace(r"\s", "[ \t\n\r\x0b\x0c]")
            .replace(r"\S", "[^ \t\n\r\x0b\x0c]"),
    )
}

fn replace_control_group(captures: &Captures) -> String {
    ((captures
        .index(0)
        .trim_start_matches(r"\c")
        .chars()
        .next()
        .unwrap()
        .to_ascii_uppercase() as u8
        - 64) as char)
        .to_string()
}
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(PatternValidator::compile(schema))
}
