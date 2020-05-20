//! Validator for `format` keyword.
use super::{CompilationResult, Validate};
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
};
use chrono::{DateTime, NaiveDate};
use regex::Regex;
use serde_json::{Map, Value};
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

macro_rules! format_validator {
    ($name:ident) => {
        struct $name {}

        impl $name {
            pub(crate) fn compile() -> CompilationResult {
                Ok(Box::new($name {}))
            }
        }
    };
}

macro_rules! validate {
    ($format:expr) => {
        fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
            if let Value::String(_item) = instance {
                if !self.is_valid(schema, instance) {
                    return error(ValidationError::format(instance, $format));
                }
            }
            no_error()
        }

        fn name(&self) -> String {
            concat!("<format: ", $format, ">").to_string()
        }
    };
}

format_validator!(DateValidator);
impl Validate for DateValidator {
    validate!("date");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            return NaiveDate::parse_from_str(item, "%Y-%m-%d").is_ok();
        }
        true
    }
}
format_validator!(DateTimeValidator);
impl Validate for DateTimeValidator {
    validate!("date-time");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            return DateTime::parse_from_rfc3339(item).is_ok();
        }
        true
    }
}
format_validator!(EmailValidator);
impl Validate for EmailValidator {
    validate!("email");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            return item.contains('@');
        }
        true
    }
}
format_validator!(IDNEmailValidator);
impl Validate for IDNEmailValidator {
    validate!("idn-email");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            return item.contains('@');
        }
        true
    }
}
format_validator!(HostnameValidator);
impl Validate for HostnameValidator {
    validate!("hostname");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            return !(item.ends_with('-')
                || item.starts_with('-')
                || item.is_empty()
                || item.chars().count() > 255
                || item
                    .chars()
                    .any(|c| !(c.is_alphanumeric() || c == '-' || c == '.'))
                || item.split('.').any(|part| part.chars().count() > 63));
        }
        true
    }
}
format_validator!(IDNHostnameValidator);
impl Validate for IDNHostnameValidator {
    validate!("idn-hostname");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            return !(item.ends_with('-')
                || item.starts_with('-')
                || item.is_empty()
                || item.chars().count() > 255
                || item
                    .chars()
                    .any(|c| !(c.is_alphanumeric() || c == '-' || c == '.'))
                || item.split('.').any(|part| part.chars().count() > 63));
        }
        true
    }
}
format_validator!(IpV4Validator);
impl Validate for IpV4Validator {
    validate!("ipv4");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            return match IpAddr::from_str(item.as_str()) {
                Ok(i) => match i {
                    IpAddr::V4(_) => true,
                    IpAddr::V6(_) => false,
                },
                Err(_) => false,
            };
        }
        true
    }
}

format_validator!(IpV6Validator);
impl Validate for IpV6Validator {
    validate!("ipv6");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            return match IpAddr::from_str(item.as_str()) {
                Ok(i) => match i {
                    IpAddr::V4(_) => false,
                    IpAddr::V6(_) => true,
                },
                Err(_) => false,
            };
        }
        true
    }
}
format_validator!(IRIValidator);
impl Validate for IRIValidator {
    validate!("iri");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            Url::from_str(item).is_ok()
        } else {
            true
        }
    }
}
format_validator!(URIValidator);
impl Validate for URIValidator {
    validate!("uri");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            Url::from_str(item).is_ok()
        } else {
            true
        }
    }
}
format_validator!(IRIReferenceValidator);
impl Validate for IRIReferenceValidator {
    validate!("iri-reference");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            IRI_REFERENCE_RE.is_match(item)
        } else {
            true
        }
    }
}
format_validator!(JSONPointerValidator);
impl Validate for JSONPointerValidator {
    validate!("json-pointer");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            JSON_POINTER_RE.is_match(item)
        } else {
            true
        }
    }
}
format_validator!(RegexValidator);
impl Validate for RegexValidator {
    validate!("regex");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            Regex::new(item).is_ok()
        } else {
            true
        }
    }
}
format_validator!(RelativeJSONPointerValidator);
impl Validate for RelativeJSONPointerValidator {
    validate!("relative-json-pointer");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            RELATIVE_JSON_POINTER_RE.is_match(item)
        } else {
            true
        }
    }
}
format_validator!(TimeValidator);
impl Validate for TimeValidator {
    validate!("time");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            TIME_RE.is_match(item)
        } else {
            true
        }
    }
}
format_validator!(URIReferenceValidator);
impl Validate for URIReferenceValidator {
    validate!("uri-reference");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            URI_REFERENCE_RE.is_match(item)
        } else {
            true
        }
    }
}
format_validator!(URITemplateValidator);
impl Validate for URITemplateValidator {
    validate!("uri-template");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            URI_TEMPLATE_RE.is_match(item)
        } else {
            true
        }
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    if let Value::String(format) = schema {
        match format.as_str() {
            "date" => Some(DateValidator::compile()),
            "date-time" => Some(DateTimeValidator::compile()),
            "email" => Some(EmailValidator::compile()),
            "hostname" => Some(HostnameValidator::compile()),
            "idn-email" => Some(IDNEmailValidator::compile()),
            "idn-hostname" => Some(IDNHostnameValidator::compile()),
            "ipv4" => Some(IpV4Validator::compile()),
            "ipv6" => Some(IpV6Validator::compile()),
            "iri" => Some(IRIValidator::compile()),
            "iri-reference" => Some(IRIReferenceValidator::compile()),
            "json-pointer" => Some(JSONPointerValidator::compile()),
            "regex" => Some(RegexValidator::compile()),
            "relative-json-pointer" => Some(RelativeJSONPointerValidator::compile()),
            "time" => Some(TimeValidator::compile()),
            "uri" => Some(URIValidator::compile()),
            "uri-reference" => Some(URIReferenceValidator::compile()),
            "uri-template" => Some(URITemplateValidator::compile()),
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
