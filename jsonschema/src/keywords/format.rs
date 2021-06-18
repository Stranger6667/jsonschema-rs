//! Validator for `format` keyword.
use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{pattern, CompilationResult},
    paths::{InstancePath, JSONPointer},
    validator::Validate,
    Draft,
};
use chrono::{DateTime, NaiveDate};
use fancy_regex::Regex;
use serde_json::{Map, Value};

use std::{net::IpAddr, str::FromStr};
use url::Url;

lazy_static::lazy_static! {
    static ref DATE_RE: Regex =
        Regex::new(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}\z").expect("Is a valid regex");
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

macro_rules! format_validator {
    ($validator:ident, $format_name:tt) => {
        struct $validator {
            schema_path: JSONPointer,
        }
        impl $validator {
            pub(crate) fn compile<'a>(context: &CompilationContext) -> CompilationResult<'a> {
                let schema_path = context.as_pointer_with("format");
                Ok(Box::new($validator { schema_path }))
            }
        }

        impl ToString for $validator {
            fn to_string(&self) -> String {
                concat!("format: ", $format_name).to_string()
            }
        }
    };
}

macro_rules! validate {
    ($format:expr) => {
        fn validate<'a>(
            &self,
            schema: &'a JSONSchema,
            instance: &'a Value,
            instance_path: &InstancePath,
        ) -> ErrorIterator<'a> {
            if let Value::String(_item) = instance {
                if !self.is_valid(schema, instance) {
                    return error(ValidationError::format(
                        self.schema_path.clone(),
                        instance_path.into(),
                        instance,
                        $format,
                    ));
                }
            }
            no_error()
        }
    };
}

format_validator!(DateValidator, "date");
impl Validate for DateValidator {
    validate!("date");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            if NaiveDate::parse_from_str(item, "%Y-%m-%d").is_ok() {
                // Padding with zeroes is ignored by the underlying parser. The most efficient
                // way to check it will be to use a custom parser that won't ignore zeroes,
                // but this regex will do the trick and costs ~20% extra time in this validator.
                DATE_RE
                    .is_match(item.as_str())
                    .expect("Simple DATE_RE pattern")
            } else {
                false
            }
        } else {
            true
        }
    }
}
format_validator!(DateTimeValidator, "date-time");
impl Validate for DateTimeValidator {
    validate!("date-time");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            DateTime::parse_from_rfc3339(item).is_ok()
        } else {
            true
        }
    }
}
fn is_valid_email(email: &str) -> bool {
    if let Some('.') = email.chars().next() {
        // dot before local part is not valid
        return false;
    }
    // This loop exits early if it finds `@`.
    // Therefore, match arms examine only the local part
    for (a, b) in email.chars().zip(email.chars().skip(1)) {
        match (a, b) {
            // two subsequent dots inside local part are not valid
            // dot after local part is not valid
            ('.', '.') | ('.', '@') => return false,
            // The domain part is not validated for simplicity
            (_, '@') => return true,
            (_, _) => continue,
        }
    }
    false
}

format_validator!(EmailValidator, "email");
impl Validate for EmailValidator {
    validate!("email");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            is_valid_email(item)
        } else {
            true
        }
    }
}
format_validator!(IDNEmailValidator, "idn-email");
impl Validate for IDNEmailValidator {
    validate!("idn-email");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            is_valid_email(item)
        } else {
            true
        }
    }
}
format_validator!(HostnameValidator, "hostname");
impl Validate for HostnameValidator {
    validate!("hostname");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            !(item.ends_with('-')
                || item.starts_with('-')
                || item.is_empty()
                || item.chars().count() > 255
                || item
                    .chars()
                    .any(|c| !(c.is_alphanumeric() || c == '-' || c == '.'))
                || item.split('.').any(|part| part.chars().count() > 63))
        } else {
            true
        }
    }
}
format_validator!(IDNHostnameValidator, "idn-hostname");
impl Validate for IDNHostnameValidator {
    validate!("idn-hostname");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            !(item.ends_with('-')
                || item.starts_with('-')
                || item.is_empty()
                || item.chars().count() > 255
                || item
                    .chars()
                    .any(|c| !(c.is_alphanumeric() || c == '-' || c == '.'))
                || item.split('.').any(|part| part.chars().count() > 63))
        } else {
            true
        }
    }
}
format_validator!(IpV4Validator, "ipv4");
impl Validate for IpV4Validator {
    validate!("ipv4");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            if item.starts_with('0') {
                return false;
            }
            match IpAddr::from_str(item.as_str()) {
                Ok(i) => i.is_ipv4(),
                Err(_) => false,
            }
        } else {
            true
        }
    }
}

format_validator!(IpV6Validator, "ipv6");
impl Validate for IpV6Validator {
    validate!("ipv6");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            match IpAddr::from_str(item.as_str()) {
                Ok(i) => i.is_ipv6(),
                Err(_) => false,
            }
        } else {
            true
        }
    }
}
format_validator!(IRIValidator, "iri");
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
format_validator!(URIValidator, "uri");
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
format_validator!(IRIReferenceValidator, "iri-reference");
impl Validate for IRIReferenceValidator {
    validate!("iri-reference");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            IRI_REFERENCE_RE
                .is_match(item)
                .expect("Simple IRI_REFERENCE_RE pattern")
        } else {
            true
        }
    }
}
format_validator!(JSONPointerValidator, "json-pointer");
impl Validate for JSONPointerValidator {
    validate!("json-pointer");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            JSON_POINTER_RE
                .is_match(item)
                .expect("Simple JSON_POINTER_RE pattern")
        } else {
            true
        }
    }
}
format_validator!(RegexValidator, "regex");
impl Validate for RegexValidator {
    validate!("regex");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            pattern::convert_regex(item).is_ok()
        } else {
            true
        }
    }
}
format_validator!(RelativeJSONPointerValidator, "relative-json-pointer");
impl Validate for RelativeJSONPointerValidator {
    validate!("relative-json-pointer");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            RELATIVE_JSON_POINTER_RE
                .is_match(item)
                .expect("Simple RELATIVE_JSON_POINTER_RE pattern")
        } else {
            true
        }
    }
}
format_validator!(TimeValidator, "time");
impl Validate for TimeValidator {
    validate!("time");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            TIME_RE.is_match(item).expect("Simple TIME_RE pattern")
        } else {
            true
        }
    }
}
format_validator!(URIReferenceValidator, "uri-reference");
impl Validate for URIReferenceValidator {
    validate!("uri-reference");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            URI_REFERENCE_RE
                .is_match(item)
                .expect("Simple URI_REFERENCE_RE pattern")
        } else {
            true
        }
    }
}
format_validator!(URITemplateValidator, "uri-template");
impl Validate for URITemplateValidator {
    validate!("uri-template");
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            URI_TEMPLATE_RE
                .is_match(item)
                .expect("Simple URI_TEMPLATE_RE pattern")
        } else {
            true
        }
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    if let Value::String(format) = schema {
        let draft_version = context.config.draft();
        match format.as_str() {
            "date-time" => Some(DateTimeValidator::compile(context)),
            "date" => Some(DateValidator::compile(context)),
            "email" => Some(EmailValidator::compile(context)),
            "hostname" => Some(HostnameValidator::compile(context)),
            "idn-email" => Some(IDNEmailValidator::compile(context)),
            "idn-hostname" if draft_version == Draft::Draft7 => {
                Some(IDNHostnameValidator::compile(context))
            }
            "ipv4" => Some(IpV4Validator::compile(context)),
            "ipv6" => Some(IpV6Validator::compile(context)),
            "iri-reference" if draft_version == Draft::Draft7 => {
                Some(IRIReferenceValidator::compile(context))
            }
            "iri" if draft_version == Draft::Draft7 => Some(IRIValidator::compile(context)),
            "json-pointer" if draft_version == Draft::Draft6 || draft_version == Draft::Draft7 => {
                Some(JSONPointerValidator::compile(context))
            }
            "regex" => Some(RegexValidator::compile(context)),
            "relative-json-pointer" if draft_version == Draft::Draft7 => {
                Some(RelativeJSONPointerValidator::compile(context))
            }
            "time" => Some(TimeValidator::compile(context)),
            "uri-reference" if draft_version == Draft::Draft6 || draft_version == Draft::Draft7 => {
                Some(URIReferenceValidator::compile(context))
            }
            "uri-template" if draft_version == Draft::Draft6 || draft_version == Draft::Draft7 => {
                Some(URITemplateValidator::compile(context))
            }
            "uri" => Some(URIValidator::compile(context)),
            _ => None,
        }
    } else {
        Some(Err(ValidationError::schema(schema)))
    }
}

#[cfg(test)]
mod tests {
    use crate::{compilation::JSONSchema, tests_util};
    use serde_json::json;

    #[test]
    fn ignored_format() {
        let schema = json!({"format": "custom", "type": "string"});
        let instance = json!("foo");
        let compiled = JSONSchema::compile(&schema).unwrap();
        assert!(compiled.is_valid(&instance))
    }

    #[test]
    fn ecma_regex() {
        // See GH-230
        let schema = json!({"format": "regex", "type": "string"});
        let instance = json!("^\\cc$");
        let compiled = JSONSchema::compile(&schema).unwrap();
        assert!(compiled.is_valid(&instance))
    }

    #[test]
    fn schema_path() {
        tests_util::assert_schema_path(&json!({"format": "date"}), &json!("bla"), "/format")
    }
}
