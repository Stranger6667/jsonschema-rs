//! Validator for `format` keyword.
use super::{CompilationResult, Validate};
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
};
use serde_json::{Map, Value};

pub struct FormatValidator {
    format: &'static str,
    check: fn(&str) -> bool,
}

impl FormatValidator {
    pub(crate) fn compile(format: &'static str, check: fn(&str) -> bool) -> CompilationResult {
        Ok(Box::new(FormatValidator { format, check }))
    }
}

impl Validate for FormatValidator {
    fn validate<'a>(&self, _: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::String(item) = instance {
            if !(self.check)(item) {
                return error(ValidationError::format(instance, self.format));
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

mod checks {
    use chrono::{DateTime, NaiveDate};
    use regex::Regex;
    use std::{net::IpAddr, str::FromStr};
    use url::Url;
    lazy_static! {
        static ref IRI_REFERENCE_RE: Regex =
            Regex::new(r"^(\w+:(/?/?))?[^#\\\s]*(#[^\\\s]*)?\z").unwrap();
        static ref JSON_POINTER_RE: Regex = Regex::new(r"^(/(([^/~])|(~[01]))*)*\z").unwrap();
        static ref RELATIVE_JSON_POINTER_RE: Regex =
            Regex::new(r"^(?:0|[1-9][0-9]*)(?:#|(?:/(?:[^~/]|~0|~1)*)*)\z").unwrap();
        static ref TIME_RE: Regex =
            Regex::new(
            r"^([01][0-9]|2[0-3]):([0-5][0-9]):([0-5][0-9])(\.[0-9]{6})?(([Zz])|([+|\-]([01][0-9]|2[0-3]):[0-5][0-9]))\z",
        ).unwrap();
        static ref URI_REFERENCE_RE: Regex =
            Regex::new(r"^(\w+:(/?/?))?[^#\\\s]*(#[^\\\s]*)?\z").unwrap();
        static ref URI_TEMPLATE_RE: Regex = Regex::new(
            r#"^(?:(?:[^\x00-\x20"'<>%\\^`{|}]|%[0-9a-f]{2})|\{[+#./;?&=,!@|]?(?:[a-z0-9_]|%[0-9a-f]{2})+(?::[1-9][0-9]{0,3}|\*)?(?:,(?:[a-z0-9_]|%[0-9a-f]{2})+(?::[1-9][0-9]{0,3}|\*)?)*})*\z"#
        )
        .unwrap();
    }

    pub(crate) fn date(instance: &str) -> bool {
        NaiveDate::parse_from_str(instance, "%Y-%m-%d").is_ok()
    }

    pub(crate) fn datetime(instance: &str) -> bool {
        DateTime::parse_from_rfc3339(instance).is_ok()
    }

    pub(crate) fn email(instance: &str) -> bool {
        instance.contains('@')
    }

    pub(crate) fn hostname(instance: &str) -> bool {
        !(instance.ends_with('-')
            || instance.starts_with('-')
            || instance.is_empty()
            || instance.chars().count() > 255
            || instance
                .chars()
                .any(|c| !(c.is_alphanumeric() || c == '-' || c == '.'))
            || instance.split('.').any(|part| part.chars().count() > 63))
    }

    pub(crate) fn ipv4(instance: &str) -> bool {
        match IpAddr::from_str(instance) {
            Ok(i) => match i {
                IpAddr::V4(_) => true,
                IpAddr::V6(_) => false,
            },
            Err(_) => false,
        }
    }

    pub(crate) fn ipv6(instance: &str) -> bool {
        match IpAddr::from_str(instance) {
            Ok(i) => match i {
                IpAddr::V4(_) => false,
                IpAddr::V6(_) => true,
            },
            Err(_) => false,
        }
    }

    pub(crate) fn iri(instance: &str) -> bool {
        Url::from_str(instance).is_ok()
    }

    pub(crate) fn iri_reference(instance: &str) -> bool {
        IRI_REFERENCE_RE.is_match(instance)
    }

    pub(crate) fn json_pointer(instance: &str) -> bool {
        JSON_POINTER_RE.is_match(instance)
    }

    pub(crate) fn regex(instance: &str) -> bool {
        Regex::new(instance).is_ok()
    }

    pub(crate) fn relative_json_pointer(instance: &str) -> bool {
        RELATIVE_JSON_POINTER_RE.is_match(instance)
    }

    pub(crate) fn time(instance: &str) -> bool {
        TIME_RE.is_match(instance)
    }

    pub(crate) fn uri_reference(instance: &str) -> bool {
        URI_REFERENCE_RE.is_match(instance)
    }

    pub(crate) fn uri_template(instance: &str) -> bool {
        URI_TEMPLATE_RE.is_match(instance)
    }
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    if let Value::String(format) = schema {
        match format.as_str() {
            "date" => Some(FormatValidator::compile("date", checks::date)),
            "date-time" => Some(FormatValidator::compile("date-time", checks::datetime)),
            "email" => Some(FormatValidator::compile("email", checks::email)),
            "hostname" => Some(FormatValidator::compile("hostname", checks::hostname)),
            "idn-email" => Some(FormatValidator::compile("idn-email", checks::email)),
            "idn-hostname" => Some(FormatValidator::compile("idn-hostname", checks::hostname)),
            "ipv4" => Some(FormatValidator::compile("ipv4", checks::ipv4)),
            "ipv6" => Some(FormatValidator::compile("ipv6", checks::ipv6)),
            "iri" => Some(FormatValidator::compile("iri", checks::iri)),
            "iri-reference" => Some(FormatValidator::compile(
                "iri-reference",
                checks::iri_reference,
            )),
            "json-pointer" => Some(FormatValidator::compile(
                "json-pointer",
                checks::json_pointer,
            )),
            "regex" => Some(FormatValidator::compile("regex", checks::regex)),
            "relative-json-pointer" => Some(FormatValidator::compile(
                "relative-json-pointer",
                checks::relative_json_pointer,
            )),
            "time" => Some(FormatValidator::compile("time", checks::time)),
            "uri" => Some(FormatValidator::compile("uri", checks::iri)),
            "uri-reference" => Some(FormatValidator::compile(
                "uri-reference",
                checks::uri_reference,
            )),
            "uri-template" => Some(FormatValidator::compile(
                "uri-template",
                checks::uri_template,
            )),
            _ => None,
        }
    } else {
        Some(Err(CompilationError::SchemaError))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ignored_format() {
        let schema = json!({"format": "custom", "type": "string"});
        let instance = json!("foo");
        let compiled = JSONSchema::compile(&schema, None).unwrap();
        assert!(compiled.is_valid(&instance))
    }
}
