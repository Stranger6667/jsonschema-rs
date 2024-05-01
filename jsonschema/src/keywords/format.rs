//! Validator for `format` keyword.
use std::{net::IpAddr, str::FromStr, sync::Arc};

use fancy_regex::Regex;
use once_cell::sync::Lazy;
use serde_json::{Map, Value};
use url::Url;
use uuid::Uuid;

use crate::{
    compilation::context::CompilationContext,
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{pattern, CompilationResult},
    paths::{JSONPointer, JsonPointerNode},
    primitive_type::PrimitiveType,
    validator::Validate,
    Draft,
};

static DATE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}\z").expect("Is a valid regex"));
static IRI_REFERENCE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\w+:(/?/?))?[^#\\\s]*(#[^\\\s]*)?\z").expect("Is a valid regex"));
static JSON_POINTER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(/(([^/~])|(~[01]))*)*\z").expect("Is a valid regex"));
static RELATIVE_JSON_POINTER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:0|[1-9][0-9]*)(?:#|(?:/(?:[^~/]|~0|~1)*)*)\z").expect("Is a valid regex")
});
static TIME_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^([01][0-9]|2[0-3]):([0-5][0-9]):([0-5][0-9])(\.[0-9]{6})?(([Zz])|([+|\-]([01][0-9]|2[0-3]):[0-5][0-9]))\z",
    ).expect("Is a valid regex")
});
static URI_REFERENCE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\w+:(/?/?))?[^#\\\s]*(#[^\\\s]*)?\z").expect("Is a valid regex"));
static URI_TEMPLATE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"^(?:(?:[^\x00-\x20"'<>%\\^`{|}]|%[0-9a-f]{2})|\{[+#./;?&=,!@|]?(?:[a-z0-9_]|%[0-9a-f]{2})+(?::[1-9][0-9]{0,3}|\*)?(?:,(?:[a-z0-9_]|%[0-9a-f]{2})+(?::[1-9][0-9]{0,3}|\*)?)*})*\z"#
    )
    .expect("Is a valid regex")
});

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

        impl core::fmt::Display for $validator {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                concat!("format: ", $format_name).fmt(f)
            }
        }
    };
}

macro_rules! validate {
    ($format:expr) => {
        fn validate<'instance>(
            &self,
            instance: &'instance Value,
            instance_path: &JsonPointerNode,
        ) -> ErrorIterator<'instance> {
            if let Value::String(_item) = instance {
                if !self.is_valid(instance) {
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
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            if time::Date::parse(
                item,
                &time::macros::format_description!("[year]-[month]-[day]"),
            )
            .is_ok()
            {
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
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            time::OffsetDateTime::parse(item, &time::format_description::well_known::Rfc3339)
                .is_ok()
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
    fn is_valid(&self, instance: &Value) -> bool {
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
    fn is_valid(&self, instance: &Value) -> bool {
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
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            !(item.ends_with('-')
                || item.starts_with('-')
                || item.is_empty()
                || bytecount::num_chars(item.as_bytes()) > 255
                || item
                    .chars()
                    .any(|c| !(c.is_alphanumeric() || c == '-' || c == '.'))
                || item
                    .split('.')
                    .any(|part| bytecount::num_chars(part.as_bytes()) > 63))
        } else {
            true
        }
    }
}
format_validator!(IDNHostnameValidator, "idn-hostname");
impl Validate for IDNHostnameValidator {
    validate!("idn-hostname");
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            !(item.ends_with('-')
                || item.starts_with('-')
                || item.is_empty()
                || bytecount::num_chars(item.as_bytes()) > 255
                || item
                    .chars()
                    .any(|c| !(c.is_alphanumeric() || c == '-' || c == '.'))
                || item
                    .split('.')
                    .any(|part| bytecount::num_chars(part.as_bytes()) > 63))
        } else {
            true
        }
    }
}
format_validator!(IpV4Validator, "ipv4");
impl Validate for IpV4Validator {
    validate!("ipv4");
    fn is_valid(&self, instance: &Value) -> bool {
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
    fn is_valid(&self, instance: &Value) -> bool {
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
    fn is_valid(&self, instance: &Value) -> bool {
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
    fn is_valid(&self, instance: &Value) -> bool {
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
    fn is_valid(&self, instance: &Value) -> bool {
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
    fn is_valid(&self, instance: &Value) -> bool {
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
    fn is_valid(&self, instance: &Value) -> bool {
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
    fn is_valid(&self, instance: &Value) -> bool {
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
    fn is_valid(&self, instance: &Value) -> bool {
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
    fn is_valid(&self, instance: &Value) -> bool {
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
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            URI_TEMPLATE_RE
                .is_match(item)
                .expect("Simple URI_TEMPLATE_RE pattern")
        } else {
            true
        }
    }
}

format_validator!(UUIDValidator, "uuid");
impl Validate for UUIDValidator {
    validate!("uuid");
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            Uuid::from_str(item.as_str()).is_ok()
        } else {
            true
        }
    }
}

format_validator!(DurationValidator, "duration");
impl Validate for DurationValidator {
    validate!("duration");
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            iso8601::duration(item).is_ok()
        } else {
            true
        }
    }
}

struct CustomFormatValidator {
    schema_path: JSONPointer,
    format_name: String,
    check: Arc<dyn Format>,
}
impl CustomFormatValidator {
    pub(crate) fn compile<'a>(
        context: &CompilationContext,
        format_name: String,
        check: Arc<dyn Format>,
    ) -> CompilationResult<'a> {
        let schema_path = context.as_pointer_with("format");
        Ok(Box::new(CustomFormatValidator {
            schema_path,
            format_name,
            check,
        }))
    }
}
impl core::fmt::Display for CustomFormatValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "format: {}", self.format_name)
    }
}

impl Validate for CustomFormatValidator {
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if !self.is_valid(instance) {
            return error(ValidationError::format(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
                self.format_name.clone(),
            ));
        }
        no_error()
    }

    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            self.check.is_valid(item)
        } else {
            true
        }
    }
}

pub(crate) trait Format: Send + Sync + 'static {
    fn is_valid(&self, value: &str) -> bool;
}

impl<F> Format for F
where
    F: Fn(&str) -> bool + Send + Sync + 'static,
{
    #[inline]
    fn is_valid(&self, value: &str) -> bool {
        self(value)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    if !context.config.validate_formats() {
        return None;
    }

    if let Value::String(format) = schema {
        if let Some((name, func)) = context.config.get_format(format) {
            return Some(CustomFormatValidator::compile(
                context,
                name.clone(),
                func.clone(),
            ));
        }
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
            #[cfg(feature = "draft201909")]
            "idn-hostname" if draft_version == Draft::Draft201909 => {
                Some(IDNHostnameValidator::compile(context))
            }
            "ipv4" => Some(IpV4Validator::compile(context)),
            "ipv6" => Some(IpV6Validator::compile(context)),
            "iri-reference" if draft_version == Draft::Draft7 => {
                Some(IRIReferenceValidator::compile(context))
            }
            #[cfg(feature = "draft201909")]
            "iri-reference" if draft_version == Draft::Draft201909 => {
                Some(IRIReferenceValidator::compile(context))
            }
            "iri" if draft_version == Draft::Draft7 => Some(IRIValidator::compile(context)),
            #[cfg(feature = "draft201909")]
            "iri" if draft_version == Draft::Draft201909 => Some(IRIValidator::compile(context)),
            "json-pointer" if draft_version == Draft::Draft6 || draft_version == Draft::Draft7 => {
                Some(JSONPointerValidator::compile(context))
            }
            #[cfg(feature = "draft201909")]
            "json-pointer" if draft_version == Draft::Draft201909 => {
                Some(JSONPointerValidator::compile(context))
            }
            "regex" => Some(RegexValidator::compile(context)),
            "relative-json-pointer" if draft_version == Draft::Draft7 => {
                Some(RelativeJSONPointerValidator::compile(context))
            }
            #[cfg(feature = "draft201909")]
            "relative-json-pointer" if draft_version == Draft::Draft201909 => {
                Some(RelativeJSONPointerValidator::compile(context))
            }
            "time" => Some(TimeValidator::compile(context)),
            "uri-reference" if draft_version == Draft::Draft6 || draft_version == Draft::Draft7 => {
                Some(URIReferenceValidator::compile(context))
            }
            #[cfg(feature = "draft201909")]
            "uri-reference" if draft_version == Draft::Draft201909 => {
                Some(URIReferenceValidator::compile(context))
            }
            "uri-template" if draft_version == Draft::Draft6 || draft_version == Draft::Draft7 => {
                Some(URITemplateValidator::compile(context))
            }
            #[cfg(feature = "draft201909")]
            "uri-template" if draft_version == Draft::Draft201909 => {
                Some(URITemplateValidator::compile(context))
            }
            #[cfg(feature = "draft201909")]
            "uuid" if draft_version == Draft::Draft201909 => Some(UUIDValidator::compile(context)),
            "uri" => Some(URIValidator::compile(context)),
            #[cfg(feature = "draft201909")]
            "duration" if draft_version == Draft::Draft201909 => {
                Some(DurationValidator::compile(context))
            }
            _ => {
                if context.config.are_unknown_formats_ignored() {
                    None
                } else {
                    return Some(Err(ValidationError::format(
                        JSONPointer::default(),
                        context.clone().schema_path.into(),
                        schema,
                        "unknown format",
                    )));
                }
            }
        }
    } else {
        Some(Err(ValidationError::single_type_error(
            JSONPointer::default(),
            context.clone().into_pointer(),
            schema,
            PrimitiveType::String,
        )))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[cfg(feature = "draft201909")]
    use crate::schemas::Draft::Draft201909;
    use crate::{compilation::JSONSchema, error::ValidationErrorKind, tests_util};

    #[test]
    fn ignored_format() {
        let schema = json!({"format": "custom", "type": "string"});
        let instance = json!("foo");
        let compiled = JSONSchema::compile(&schema).unwrap();
        assert!(compiled.is_valid(&instance))
    }

    #[test]
    fn format_validation() {
        let schema = json!({"format": "email", "type": "string"});
        let email_instance = json!("email@example.com");
        let not_email_instance = json!("foo");

        let with_validation = JSONSchema::options()
            .should_validate_formats(true)
            .compile(&schema)
            .unwrap();
        let without_validation = JSONSchema::options()
            .should_validate_formats(false)
            .compile(&schema)
            .unwrap();

        assert!(with_validation.is_valid(&email_instance));
        assert!(!with_validation.is_valid(&not_email_instance));
        assert!(without_validation.is_valid(&email_instance));
        assert!(without_validation.is_valid(&not_email_instance));
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

    #[cfg(feature = "draft201909")]
    #[test]
    fn uuid() {
        let schema = json!({"format": "uuid", "type": "string"});

        let passing_instance = json!("f308a72c-fa84-11eb-9a03-0242ac130003");
        let failing_instance = json!("1");

        let compiled = JSONSchema::options()
            .with_draft(Draft201909)
            .should_validate_formats(true)
            .compile(&schema)
            .unwrap();

        assert!(compiled.is_valid(&passing_instance));
        assert!(!compiled.is_valid(&failing_instance))
    }

    #[test]
    fn uri() {
        let schema = json!({"format": "uri", "type": "string"});

        let passing_instance = json!("https://phillip.com");
        let failing_instance = json!("redis");

        tests_util::is_valid(&schema, &passing_instance);
        tests_util::is_not_valid(&schema, &failing_instance);
    }

    #[cfg(feature = "draft201909")]
    #[test]
    fn duration() {
        let schema = json!({"format": "duration", "type": "string"});

        let passing_instances = vec![json!("P15DT1H22M1.5S"), json!("P30D"), json!("PT5M")];
        let failing_instances = vec![json!("15DT1H22M1.5S"), json!("unknown")];

        let compiled = JSONSchema::options()
            .with_draft(Draft201909)
            .should_validate_formats(true)
            .compile(&schema)
            .unwrap();

        for passing_instance in passing_instances {
            assert!(compiled.is_valid(&passing_instance));
        }
        for failing_instance in failing_instances {
            assert!(!compiled.is_valid(&failing_instance));
        }
    }

    #[test]
    fn unknown_formats_should_not_be_ignored() {
        let schema = json!({ "format": "custom", "type": "string"});
        let validation_error = JSONSchema::options()
            .should_ignore_unknown_formats(false)
            .compile(&schema)
            .expect_err("the validation error should be returned");

        assert!(
            matches!(validation_error.kind, ValidationErrorKind::Format { format } if format == "unknown format")
        );
        assert_eq!("\"custom\"", validation_error.instance.to_string())
    }
}
