//! Validator for `format` keyword.
use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
    Draft,
};
use chrono::{DateTime, NaiveDate};
use regex::Regex;
use serde_json::{Map, Value};
use std::{net::IpAddr, str::FromStr};
use url::Url;

lazy_static::lazy_static! {
    static ref IRI_REFERENCE_RE: Regex =
        Regex::new(r"^(\w+:(/?/?))?[^#\\\s]*(#[^\\\s]*)?\z").expect("Is a valid regex");
    static ref JSON_POINTER_RE: Regex = Regex::new(r"^(/(([^/~])|(~[01]))*)*\z").expect("Is a valid regex");
    static ref RELATIVE_JSON_POINTER_RE: Regex =
        Regex::new(r"^(?:0|[1-9][0-9]*)(?:#|(?:/(?:[^~/]|~0|~1)*)*)\z").expect("Is a valid regex");
    static ref TIME_RE: Regex =
        Regex::new(
        r"^([01][0-9]|2[0-3]):([0-5][0-9]):([0-5][0-9])(\.[0-9]{6})?(([Zz])|([+|\-]([01][0-9]|2[0-3]):[0-5][0-9]))\z",
    ).expect("Is a valid regex");
    static ref URI_REFERENCE_RE: Regex =
        Regex::new(r"^(\w+:(/?/?))?[^#\\\s]*(#[^\\\s]*)?\z").expect("Is a valid regex");
    static ref URI_TEMPLATE_RE: Regex = Regex::new(
        r#"^(?:(?:[^\x00-\x20"'<>%\\^`{|}]|%[0-9a-f]{2})|\{[+#./;?&=,!@|]?(?:[a-z0-9_]|%[0-9a-f]{2})+(?::[1-9][0-9]{0,3}|\*)?(?:,(?:[a-z0-9_]|%[0-9a-f]{2})+(?::[1-9][0-9]{0,3}|\*)?)*})*\z"#
    )
    .expect("Is a valid regex");
}

macro_rules! generic_format_validator {
    ($validator:ident, $format_name:tt => $($validate_components_extra:tt)*) => {
        struct $validator {}
        impl $validator {
            pub(crate) fn compile() -> CompilationResult {
                Ok(Box::new($validator {}))
            }
        }
        impl Validate for $validator {
            #[inline]
            fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
                ValidationError::format(instance, $format_name)
            }
            $($validate_components_extra)*
        }
        impl ToString for $validator {
            fn to_string(&self) -> String {
                concat!("format: ", $format_name).to_string()
            }
        }
    };
}

macro_rules! string_format_validator {
    ($validator:ident, $format_name:tt, $check:expr) => {
        generic_format_validator!(
            $validator,
            $format_name =>
            #[inline]
            fn is_valid_string(&self, _: &JSONSchema, _: &Value, instance_string: &str) -> bool {
                $check(instance_string)
            }
            #[inline]
            fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
                if let Value::String(instance_string) = instance {
                    self.is_valid_string(schema, instance, instance_string)
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
        );
    };
}

#[inline]
fn is_valid_email(string: &str) -> bool {
    string.contains('@')
}
#[inline]
fn is_valid_hostname(string: &str) -> bool {
    !(string.ends_with('-')
        || string.starts_with('-')
        || string.is_empty()
        || string.chars().count() > 255
        || string
            .chars()
            .any(|c| !(c.is_alphanumeric() || c == '-' || c == '.'))
        || string.split('.').any(|part| part.chars().count() > 63))
}
#[inline]
fn is_valid_idn_hostname(string: &str) -> bool {
    is_valid_hostname(string) && idna::domain_to_unicode(string).1.is_ok()
}

string_format_validator!(DateValidator, "date", |instance_string| {
    NaiveDate::parse_from_str(instance_string, "%Y-%m-%d").is_ok()
});
string_format_validator!(DateTimeValidator, "date-time", |instance_string| {
    DateTime::parse_from_rfc3339(instance_string).is_ok()
});
string_format_validator!(EmailValidator, "email", is_valid_email);
string_format_validator!(IDNEmailValidator, "idn-email", is_valid_email);
string_format_validator!(HostnameValidator, "hostname", is_valid_hostname);
string_format_validator!(IDNHostnameValidator, "idn-hostname", is_valid_idn_hostname);
string_format_validator!(IpV4Validator, "ipv4", |instance_string| {
    if let Ok(IpAddr::V4(_)) = IpAddr::from_str(instance_string) {
        true
    } else {
        false
    }
});
string_format_validator!(IpV6Validator, "ipv6", |instance_string| {
    if let Ok(IpAddr::V6(_)) = IpAddr::from_str(instance_string) {
        true
    } else {
        false
    }
});
string_format_validator!(IRIValidator, "iri", |instance_string| {
    Url::from_str(instance_string).is_ok()
});
string_format_validator!(URIValidator, "uri", |instance_string| {
    Url::from_str(instance_string).is_ok()
});
string_format_validator!(IRIReferenceValidator, "iri-reference", |instance_value| {
    IRI_REFERENCE_RE.is_match(instance_value)
});
string_format_validator!(JSONPointerValidator, "json-pointer", |instance_value| {
    JSON_POINTER_RE.is_match(instance_value)
});
string_format_validator!(RegexValidator, "regex", |instance_value| {
    Regex::new(instance_value).is_ok()
});
string_format_validator!(
    RelativeJSONPointerValidator,
    "relative-json-pointer",
    |instance_value| RELATIVE_JSON_POINTER_RE.is_match(instance_value)
);
string_format_validator!(TimeValidator, "time", |instance_value| TIME_RE
    .is_match(instance_value));
string_format_validator!(URIReferenceValidator, "uri-reference", |instance_value| {
    URI_REFERENCE_RE.is_match(instance_value)
});
string_format_validator!(URITemplateValidator, "uri-template", |instance_value| {
    URI_TEMPLATE_RE.is_match(instance_value)
});

#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    if let Value::String(format) = schema {
        let draft_version = context.config.draft();
        match format.as_str() {
            "date-time" => Some(DateTimeValidator::compile()),
            "date" => Some(DateValidator::compile()),
            "email" => Some(EmailValidator::compile()),
            "hostname" => Some(HostnameValidator::compile()),
            "idn-email" => Some(IDNEmailValidator::compile()),
            "idn-hostname" if draft_version == Draft::Draft7 => {
                Some(IDNHostnameValidator::compile())
            }
            "ipv4" => Some(IpV4Validator::compile()),
            "ipv6" => Some(IpV6Validator::compile()),
            "iri-reference" if draft_version == Draft::Draft7 => {
                Some(IRIReferenceValidator::compile())
            }
            "iri" if draft_version == Draft::Draft7 => Some(IRIValidator::compile()),
            "json-pointer" if draft_version == Draft::Draft6 || draft_version == Draft::Draft7 => {
                Some(JSONPointerValidator::compile())
            }
            "regex" => Some(RegexValidator::compile()),
            "relative-json-pointer" if draft_version == Draft::Draft7 => {
                Some(RelativeJSONPointerValidator::compile())
            }
            "time" => Some(TimeValidator::compile()),
            "uri-reference" if draft_version == Draft::Draft6 || draft_version == Draft::Draft7 => {
                Some(URIReferenceValidator::compile())
            }
            "uri-template" if draft_version == Draft::Draft6 || draft_version == Draft::Draft7 => {
                Some(URITemplateValidator::compile())
            }
            "uri" => Some(URIValidator::compile()),
            _ => None,
        }
    } else {
        Some(Err(CompilationError::SchemaError))
    }
}

#[cfg(test)]
mod tests {
    use crate::compilation::JSONSchema;
    use serde_json::json;

    #[test]
    fn ignored_format() {
        let schema = json!({"format": "custom", "type": "string"});
        let instance = json!("foo");
        let compiled = JSONSchema::compile(&schema).unwrap();
        assert!(compiled.is_valid(&instance))
    }
}
