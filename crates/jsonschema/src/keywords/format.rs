//! Validator for `format` keyword.
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
    sync::Arc,
};

use email_address::EmailAddress;
use fancy_regex::Regex;
use once_cell::sync::Lazy;
use serde_json::{Map, Value};
use uuid_simd::{parse_hyphenated, Out};

use crate::{
    compiler,
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{pattern, CompilationResult},
    paths::{JsonPointer, JsonPointerNode},
    primitive_type::PrimitiveType,
    validator::Validate,
    Draft,
};

static JSON_POINTER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(/(([^/~])|(~[01]))*)*\z").expect("Is a valid regex"));
static RELATIVE_JSON_POINTER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:0|[1-9][0-9]*)(?:#|(?:/(?:[^~/]|~0|~1)*)*)\z").expect("Is a valid regex")
});
static URI_TEMPLATE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"^(?:(?:[^\x00-\x20"'<>%\\^`{|}]|%[0-9a-f]{2})|\{[+#./;?&=,!@|]?(?:[a-z0-9_]|%[0-9a-f]{2})+(?::[1-9][0-9]{0,3}|\*)?(?:,(?:[a-z0-9_]|%[0-9a-f]{2})+(?::[1-9][0-9]{0,3}|\*)?)*})*\z"#
    )
    .expect("Is a valid regex")
});

macro_rules! format_validator {
    ($validator:ident, $format_name:tt) => {
        struct $validator {
            schema_path: JsonPointer,
        }
        impl $validator {
            pub(crate) fn compile<'a>(ctx: &compiler::Context) -> CompilationResult<'a> {
                let schema_path = ctx.as_pointer_with("format");
                Ok(Box::new($validator { schema_path }))
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

fn is_valid_date(date: &str) -> bool {
    if date.len() != 10 {
        return false;
    }

    let bytes = date.as_bytes();

    // Check format: YYYY-MM-DD
    if bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes[0].is_ascii_digit()
        || !bytes[1].is_ascii_digit()
        || !bytes[2].is_ascii_digit()
        || !bytes[3].is_ascii_digit()
        || !bytes[5].is_ascii_digit()
        || !bytes[6].is_ascii_digit()
        || !bytes[8].is_ascii_digit()
        || !bytes[9].is_ascii_digit()
    {
        return false;
    }

    // Parse year
    let year = (bytes[0] as u16 - b'0' as u16) * 1000
        + (bytes[1] as u16 - b'0' as u16) * 100
        + (bytes[2] as u16 - b'0' as u16) * 10
        + (bytes[3] as u16 - b'0' as u16);

    // Parse month
    let month = (bytes[5] as u8 - b'0') * 10 + (bytes[6] as u8 - b'0');
    if month < 1 || month > 12 {
        return false;
    }

    // Parse day
    let day = (bytes[8] as u8 - b'0') * 10 + (bytes[9] as u8 - b'0');

    // Check day validity
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => day <= 31,
        4 | 6 | 9 | 11 => day <= 30,
        2 => {
            if is_leap_year(year) {
                day <= 29
            } else {
                day <= 28
            }
        }
        _ => false,
    }
}

#[inline]
fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

format_validator!(DateValidator, "date");
impl Validate for DateValidator {
    validate!("date");
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            is_valid_date(item)
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
    if let Ok(parsed) = EmailAddress::from_str(email) {
        let domain = parsed.domain();
        if let Some(domain) = domain.strip_prefix('[').and_then(|d| d.strip_suffix(']')) {
            if let Some(domain) = domain.strip_prefix("IPv6:") {
                domain.parse::<Ipv6Addr>().is_ok()
            } else {
                domain.parse::<Ipv4Addr>().is_ok()
            }
        } else {
            is_valid_hostname(domain)
        }
    } else {
        false
    }
}

fn is_valid_hostname(hostname: &str) -> bool {
    !(hostname.ends_with('-')
        || hostname.starts_with('-')
        || hostname.is_empty()
        || bytecount::num_chars(hostname.as_bytes()) > 255
        || hostname
            .chars()
            .any(|c| !(c.is_alphanumeric() || c == '-' || c == '.'))
        || hostname
            .split('.')
            .any(|part| bytecount::num_chars(part.as_bytes()) > 63))
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
            is_valid_hostname(item)
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
            is_valid_hostname(item)
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
            Ipv4Addr::from_str(item).is_ok()
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
            Ipv6Addr::from_str(item).is_ok()
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
            referencing::Iri::parse(item.as_str()).is_ok()
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
            referencing::Uri::parse(item.as_str()).is_ok()
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
            referencing::IriRef::parse(item.as_str()).is_ok()
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
            referencing::UriRef::parse(item.as_str()).is_ok()
        } else {
            true
        }
    }
}
format_validator!(JsonPointerValidator, "json-pointer");
impl Validate for JsonPointerValidator {
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
format_validator!(RelativeJsonPointerValidator, "relative-json-pointer");
impl Validate for RelativeJsonPointerValidator {
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

fn is_valid_time(item: &str) -> bool {
    let bytes = item.as_bytes();
    let len = bytes.len();

    if len < 9 {
        // Minimum valid time is "HH:MM:SSZ"
        return false;
    }

    // Check HH:MM:SS format
    if !bytes[0].is_ascii_digit()
        || !bytes[1].is_ascii_digit()
        || bytes[2] != b':'
        || !bytes[3].is_ascii_digit()
        || !bytes[4].is_ascii_digit()
        || bytes[5] != b':'
        || !bytes[6].is_ascii_digit()
        || !bytes[7].is_ascii_digit()
    {
        return false;
    }

    // Parse hours, minutes, seconds
    let hh = (bytes[0] - b'0') * 10 + (bytes[1] - b'0');
    let mm = (bytes[3] - b'0') * 10 + (bytes[4] - b'0');
    let ss = (bytes[6] - b'0') * 10 + (bytes[7] - b'0');

    if hh > 23 || mm > 59 || ss > 60 {
        return false;
    }

    let mut i = 8;

    // Check fractional seconds
    if i < len && bytes[i] == b'.' {
        i += 1;
        let mut has_digit = false;
        while i < len && bytes[i].is_ascii_digit() {
            has_digit = true;
            i += 1;
        }
        if !has_digit {
            return false;
        }
    }

    // Check offset
    if i == len {
        return false;
    }

    match bytes[i] {
        b'Z' | b'z' => i == len - 1 && (ss != 60 || (hh == 23 && mm == 59)),
        b'+' | b'-' => {
            if len - i != 6 {
                return false;
            }
            i += 1;
            let offset_hh = (bytes[i] - b'0') * 10 + (bytes[i + 1] - b'0');
            let offset_mm = (bytes[i + 3] - b'0') * 10 + (bytes[i + 4] - b'0');
            if !bytes[i].is_ascii_digit()
                || !bytes[i + 1].is_ascii_digit()
                || bytes[i + 2] != b':'
                || !bytes[i + 3].is_ascii_digit()
                || !bytes[i + 4].is_ascii_digit()
                || offset_hh > 23
                || offset_mm > 59
            {
                return false;
            }
            if ss == 60 {
                let mut utc_hh = hh as i32;
                let mut utc_mm = mm as i32;
                if bytes[i - 1] == b'+' {
                    utc_hh -= offset_hh as i32;
                    utc_mm -= offset_mm as i32;
                } else {
                    // '-'
                    utc_hh += offset_hh as i32;
                    utc_mm += offset_mm as i32;
                }
                // Adjust for minute overflow/underflow
                utc_hh += utc_mm / 60;
                utc_mm %= 60;
                if utc_mm < 0 {
                    utc_mm += 60;
                    utc_hh -= 1;
                }
                // Adjust for hour overflow/underflow
                utc_hh = (utc_hh + 24) % 24;
                utc_hh == 23 && utc_mm == 59
            } else {
                true
            }
        }
        _ => false,
    }
}

format_validator!(TimeValidator, "time");
impl Validate for TimeValidator {
    validate!("time");
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            is_valid_time(item)
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
            let mut out = [0; 16];
            parse_hyphenated(item.as_bytes(), Out::from_mut(&mut out)).is_ok()
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
    schema_path: JsonPointer,
    format_name: String,
    check: Arc<dyn Format>,
}
impl CustomFormatValidator {
    pub(crate) fn compile<'a>(
        ctx: &compiler::Context,
        format_name: String,
        check: Arc<dyn Format>,
    ) -> CompilationResult<'a> {
        let schema_path = ctx.as_pointer_with("format");
        Ok(Box::new(CustomFormatValidator {
            schema_path,
            format_name,
            check,
        }))
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
    ctx: &compiler::Context,
    _: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    if !ctx.validates_formats_by_default() {
        return None;
    }

    if let Value::String(format) = schema {
        if let Some((name, func)) = ctx.get_format(format) {
            return Some(CustomFormatValidator::compile(
                ctx,
                name.clone(),
                func.clone(),
            ));
        }
        let draft = ctx.draft();
        match format.as_str() {
            "date-time" => Some(DateTimeValidator::compile(ctx)),
            "date" => Some(DateValidator::compile(ctx)),
            "email" => Some(EmailValidator::compile(ctx)),
            "hostname" => Some(HostnameValidator::compile(ctx)),
            "idn-email" => Some(IDNEmailValidator::compile(ctx)),
            "idn-hostname"
                if matches!(
                    draft,
                    Draft::Draft7 | Draft::Draft201909 | Draft::Draft202012
                ) =>
            {
                Some(IDNHostnameValidator::compile(ctx))
            }
            "ipv4" => Some(IpV4Validator::compile(ctx)),
            "ipv6" => Some(IpV6Validator::compile(ctx)),
            "iri-reference"
                if matches!(
                    draft,
                    Draft::Draft7 | Draft::Draft201909 | Draft::Draft202012
                ) =>
            {
                Some(IRIReferenceValidator::compile(ctx))
            }
            "iri"
                if matches!(
                    draft,
                    Draft::Draft7 | Draft::Draft201909 | Draft::Draft202012
                ) =>
            {
                Some(IRIValidator::compile(ctx))
            }
            "json-pointer"
                if matches!(
                    draft,
                    Draft::Draft6 | Draft::Draft7 | Draft::Draft201909 | Draft::Draft202012
                ) =>
            {
                Some(JsonPointerValidator::compile(ctx))
            }
            "regex" => Some(RegexValidator::compile(ctx)),
            "relative-json-pointer"
                if matches!(
                    draft,
                    Draft::Draft7 | Draft::Draft201909 | Draft::Draft202012
                ) =>
            {
                Some(RelativeJsonPointerValidator::compile(ctx))
            }
            "time" => Some(TimeValidator::compile(ctx)),
            "uri-reference"
                if matches!(
                    draft,
                    Draft::Draft6 | Draft::Draft7 | Draft::Draft201909 | Draft::Draft202012
                ) =>
            {
                Some(URIReferenceValidator::compile(ctx))
            }
            "uri-template"
                if matches!(
                    draft,
                    Draft::Draft6 | Draft::Draft7 | Draft::Draft201909 | Draft::Draft202012
                ) =>
            {
                Some(URITemplateValidator::compile(ctx))
            }
            "uuid" if matches!(draft, Draft::Draft201909 | Draft::Draft202012) => {
                Some(UUIDValidator::compile(ctx))
            }
            "uri" => Some(URIValidator::compile(ctx)),
            "duration" if matches!(draft, Draft::Draft201909 | Draft::Draft202012) => {
                Some(DurationValidator::compile(ctx))
            }
            _ => {
                if ctx.are_unknown_formats_ignored() {
                    None
                } else {
                    return Some(Err(ValidationError::format(
                        JsonPointer::default(),
                        ctx.clone().path.into(),
                        schema,
                        "unknown format",
                    )));
                }
            }
        }
    } else {
        Some(Err(ValidationError::single_type_error(
            JsonPointer::default(),
            ctx.clone().into_pointer(),
            schema,
            PrimitiveType::String,
        )))
    }
}

#[cfg(test)]
mod tests {
    use referencing::Draft;
    use serde_json::json;
    use test_case::test_case;

    use crate::{error::ValidationErrorKind, tests_util};

    #[test]
    fn ignored_format() {
        let schema = json!({"format": "custom", "type": "string"});
        let instance = json!("foo");
        let validator = crate::validator_for(&schema).unwrap();
        assert!(validator.is_valid(&instance))
    }

    #[test]
    fn format_validation() {
        let schema = json!({"format": "email", "type": "string"});
        let email_instance = json!("email@example.com");
        let not_email_instance = json!("foo");

        let with_validation = crate::options()
            .should_validate_formats(true)
            .build(&schema)
            .unwrap();
        let without_validation = crate::options()
            .should_validate_formats(false)
            .build(&schema)
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
        let validator = crate::validator_for(&schema).unwrap();
        assert!(validator.is_valid(&instance))
    }

    #[test]
    fn schema_path() {
        tests_util::assert_schema_path(&json!({"format": "date"}), &json!("bla"), "/format")
    }

    #[test]
    fn uuid() {
        let schema = json!({"format": "uuid", "type": "string"});

        let passing_instance = json!("f308a72c-fa84-11eb-9a03-0242ac130003");
        let failing_instance = json!("1");

        let validator = crate::options()
            .with_draft(Draft::Draft201909)
            .should_validate_formats(true)
            .build(&schema)
            .unwrap();

        assert!(validator.is_valid(&passing_instance));
        assert!(!validator.is_valid(&failing_instance))
    }

    #[test]
    fn uri() {
        let schema = json!({"format": "uri", "type": "string"});

        let passing_instance = json!("https://phillip.com");
        let failing_instance = json!("redis");

        tests_util::is_valid(&schema, &passing_instance);
        tests_util::is_not_valid(&schema, &failing_instance);
    }

    #[test]
    fn duration() {
        let schema = json!({"format": "duration", "type": "string"});

        let passing_instances = vec![json!("P15DT1H22M1.5S"), json!("P30D"), json!("PT5M")];
        let failing_instances = vec![json!("15DT1H22M1.5S"), json!("unknown")];

        let validator = crate::options()
            .with_draft(Draft::Draft201909)
            .should_validate_formats(true)
            .build(&schema)
            .unwrap();

        for passing_instance in passing_instances {
            assert!(validator.is_valid(&passing_instance));
        }
        for failing_instance in failing_instances {
            assert!(!validator.is_valid(&failing_instance));
        }
    }

    #[test]
    fn unknown_formats_should_not_be_ignored() {
        let schema = json!({ "format": "custom", "type": "string"});
        let validation_error = crate::options()
            .should_ignore_unknown_formats(false)
            .build(&schema)
            .expect_err("the validation error should be returned");

        assert!(
            matches!(validation_error.kind, ValidationErrorKind::Format { format } if format == "unknown format")
        );
        assert_eq!("\"custom\"", validation_error.instance.to_string())
    }

    #[test_case("127.0.0.1", true)]
    #[test_case("192.168.1.1", true)]
    #[test_case("10.0.0.1", true)]
    #[test_case("0.0.0.0", true)]
    #[test_case("256.1.2.3", false; "first octet too large")]
    #[test_case("1.256.3.4", false; "second octet too large")]
    #[test_case("1.2.256.4", false; "third octet too large")]
    #[test_case("1.2.3.256", false; "fourth octet too large")]
    #[test_case("01.2.3.4", false; "leading zero in first octet")]
    #[test_case("1.02.3.4", false; "leading zero in second octet")]
    #[test_case("1.2.03.4", false; "leading zero in third octet")]
    #[test_case("1.2.3.04", false; "leading zero in fourth octet")]
    #[test_case("1.2.3", false; "too few octets")]
    #[test_case("1.2.3.4.5", false; "too many octets")]
    fn ip_v4(input: &str, expected: bool) {
        let validator = crate::options()
            .should_validate_formats(true)
            .build(&json!({"format": "ipv4", "type": "string"}))
            .expect("Invalid schema");
        assert_eq!(validator.is_valid(&json!(input)), expected);
    }
}
