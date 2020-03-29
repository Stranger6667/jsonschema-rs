//! Validator for `format` keyword.
use super::CompilationResult;
use super::Validate;
use crate::context::CompilationContext;
use crate::error::{error, no_error, CompilationError, ErrorIterator};
use crate::{checks, JSONSchema, ValidationError};
use serde_json::{Map, Value};

pub struct FormatValidator {
    format: String,
    check: fn(&str) -> bool,
}

impl FormatValidator {
    pub(crate) fn compile(format: &str, check: fn(&str) -> bool) -> CompilationResult {
        Ok(Box::new(FormatValidator {
            format: format.to_string(),
            check,
        }))
    }
}

impl Validate for FormatValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(item) = instance {
            if !(self.check)(item) {
                return error(ValidationError::format(
                    item.to_owned(),
                    self.format.clone(),
                ));
            }
        }
        no_error()
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            return (self.check)(item);
        }
        true
    }

    fn name(&self) -> String {
        format!("<format: {}>", self.format)
    }
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    match schema.as_str() {
        Some(format) => {
            let func = match format {
                "date" => checks::date,
                "date-time" => checks::datetime,
                "email" => checks::email,
                "hostname" => checks::hostname,
                "idn-email" => checks::email,
                "idn-hostname" => checks::hostname,
                "ipv4" => checks::ipv4,
                "ipv6" => checks::ipv6,
                "iri" => checks::iri,
                "iri-reference" => checks::iri_reference,
                "json-pointer" => checks::json_pointer,
                "regex" => checks::regex,
                "relative-json-pointer" => checks::relative_json_pointer,
                "time" => checks::time,
                "uri" => checks::iri,
                "uri-reference" => checks::uri_reference,
                "uri-template" => checks::uri_template,
                _ => return None,
            };
            Some(FormatValidator::compile(format, func))
        }
        None => Some(Err(CompilationError::SchemaError)),
    }
}
