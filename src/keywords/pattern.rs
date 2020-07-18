use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use regex::{Captures, Regex};
use serde_json::{Map, Value};

use std::ops::Index;

lazy_static::lazy_static! {
    static ref CONTROL_GROUPS_RE: Regex = Regex::new(r"\\c[A-Za-z]").expect("Is a valid regex");
}

pub(crate) struct PatternValidator {
    original: String,
    pattern: Regex,
}

impl PatternValidator {
    #[inline]
    pub(crate) fn compile(pattern: &Value) -> CompilationResult {
        match pattern {
            Value::String(item) => {
                let pattern = convert_regex(item)?;
                Ok(Box::new(PatternValidator {
                    original: item.clone(),
                    pattern,
                }))
            }
            _ => Err(CompilationError::SchemaError),
        }
    }
}

impl Validate for PatternValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::pattern(instance, self.original.clone())
    }

    #[inline]
    fn is_valid_string(&self, _: &JSONSchema, _: &Value, instance_value: &str) -> bool {
        self.pattern.is_match(instance_value)
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(instance_value) = instance {
            self.is_valid_string(schema, instance, instance_value)
        } else {
            true
        }
    }

    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(instance_value) = instance {
            self.validate_string(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}
impl ToString for PatternValidator {
    fn to_string(&self) -> String {
        format!("pattern: {}", self.pattern)
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
            .replace(
                r"\s",
                "[ \t\n\r\u{000b}\u{000c}\u{2003}\u{feff}\u{2029}\u{00a0}]",
            )
            .replace(
                r"\S",
                "[^ \t\n\r\u{000b}\u{000c}\u{2003}\u{feff}\u{2029}\u{00a0}]",
            ),
    )
}

#[allow(clippy::integer_arithmetic)]
fn replace_control_group(captures: &Captures) -> String {
    // There will be no overflow, because the minimum value is 65 (char 'A')
    ((captures
        .index(0)
        .trim_start_matches(r"\c")
        .chars()
        .next()
        .expect("This is always present because of the regex rule. It has [A-Za-z] next")
        .to_ascii_uppercase() as u8
        - 64) as char)
        .to_string()
}

#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(PatternValidator::compile(schema))
}
