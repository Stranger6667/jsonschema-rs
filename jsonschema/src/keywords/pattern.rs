use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    paths::InstancePath,
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
    fn validate<'a>(
        &self,
        _: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::String(item) = instance {
            if !self.pattern.is_match(item) {
                return error(ValidationError::pattern(
                    instance_path.into(),
                    instance,
                    self.original.clone(),
                ));
            }
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            if !self.pattern.is_match(item) {
                return false;
            }
        }
        true
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
    let mut out = String::with_capacity(new_pattern.len());
    let mut chars = new_pattern.chars().peekable();
    // To convert character group we need to iterate over chars and in case of `\` take a look
    // at the next char to detect whether this group should be converted
    while let Some(current) = chars.next() {
        if current == '\\' {
            // Possible character group
            if let Some(next) = chars.next() {
                match next {
                    'd' => out.push_str("[0-9]"),
                    'D' => out.push_str("[^0-9]"),
                    'w' => out.push_str("[A-Za-z0-9_]"),
                    'W' => out.push_str("[^A-Za-z0-9_]"),
                    's' => {
                        out.push_str("[ \t\n\r\u{000b}\u{000c}\u{2003}\u{feff}\u{2029}\u{00a0}]")
                    }
                    'S' => {
                        out.push_str("[^ \t\n\r\u{000b}\u{000c}\u{2003}\u{feff}\u{2029}\u{00a0}]")
                    }
                    _ => {
                        // Nothing interesting, push as is
                        out.push(current);
                        out.push(next)
                    }
                }
            } else {
                // End of the string, push the last char.
                // Note that it is an incomplete escape sequence and will lead to an error on
                // the next step
                out.push(current);
            }
        } else {
            // Regular character
            out.push(current);
        }
    }
    Regex::new(&out)
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(r"^[\w\-\.\+]+$", "CC-BY-4.0", true)]
    #[test_case(r"^[\w\-\.\+]+$", "CC-BY-!", false)]
    #[test_case(r"^\W+$", "1_0", false)]
    #[test_case(r"\\w", r"\w", true)]
    fn regex_matches(pattern: &str, text: &str, is_matching: bool) {
        let compiled = convert_regex(pattern).expect("A valid regex");
        assert_eq!(compiled.is_match(text), is_matching);
    }

    #[test_case(r"\")]
    #[test_case(r"\d\")]
    fn invalid_escape_sequences(pattern: &str) {
        assert!(convert_regex(pattern).is_err())
    }
}
