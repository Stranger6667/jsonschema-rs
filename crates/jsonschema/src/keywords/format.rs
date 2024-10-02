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
    if bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }

    // Parse year (YYYY)
    let Some(year) = parse_four_digits(&bytes[0..4]) else {
        return false;
    };

    // Parse month (MM)
    let Some(month) = parse_two_digits(&bytes[5..7]) else {
        return false;
    };
    if !(1..=12).contains(&month) {
        return false;
    }

    // Parse day (DD)
    let Some(day) = parse_two_digits(&bytes[8..10]) else {
        return false;
    };
    if day == 0 {
        return false;
    }

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
        _ => unreachable!("Month value is checked above"),
    }
}

#[inline]
fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[inline]
fn parse_four_digits(bytes: &[u8]) -> Option<u16> {
    let value = u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

    // Check if all bytes are ASCII digits
    if value.wrapping_sub(0x30303030) & 0xF0F0F0F0 == 0 {
        let val = (value & 0x0F0F_0F0F).wrapping_mul(2561) >> 8;
        Some(((val & 0x00FF_00FF).wrapping_mul(6_553_601) >> 16) as u16)
    } else {
        None
    }
}

#[inline]
fn parse_two_digits(bytes: &[u8]) -> Option<u8> {
    let value = u16::from_ne_bytes([bytes[0], bytes[1]]);

    // Check if both bytes are ASCII digits
    if value.wrapping_sub(0x3030) & 0xF0F0 == 0 {
        Some(((value & 0x0F0F).wrapping_mul(2561) >> 8) as u8)
    } else {
        None
    }
}

macro_rules! handle_offset {
    ($sign:tt, $i:ident, $bytes:expr, $hour:expr, $minute:expr, $second:expr) => {{
        if $bytes.len() - $i != 6 {
            return false;
        }
        $i += 1;
        if $bytes[$i + 2] != b':' {
            return false;
        }
        let Some(offset_hh) = parse_two_digits(&$bytes[$i..$i + 2]) else {
            return false;
        };
        let Some(offset_mm) = parse_two_digits(&$bytes[$i + 3..$i + 5]) else {
            return false;
        };
        if offset_hh > 23 || offset_mm > 59 {
            return false;
        }

        if $second == 60 {
            let mut utc_hh = $hour as i8;
            let mut utc_mm = $minute as i8;

            // Apply offset based on the sign (+ or -)
            utc_hh $sign offset_hh as i8;
            utc_mm $sign offset_mm as i8;

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
    }};
}
fn is_valid_time(time: &str) -> bool {
    let bytes = time.as_bytes();
    let len = bytes.len();

    if len < 9 {
        // Minimum valid time is "HH:MM:SSZ"
        return false;
    }

    // Check HH:MM:SS format
    if bytes[2] != b':' || bytes[5] != b':' {
        return false;
    }

    // Parse hour (HH)
    let Some(hour) = parse_two_digits(&bytes[..2]) else {
        return false;
    };
    // Parse minute (MM)
    let Some(minute) = parse_two_digits(&bytes[3..5]) else {
        return false;
    };
    // Parse second (SS)
    let Some(second) = parse_two_digits(&bytes[6..8]) else {
        return false;
    };

    if hour > 23 || minute > 59 || second > 60 {
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
        b'Z' | b'z' => i == len - 1 && (second != 60 || (hour == 23 && minute == 59)),
        b'+' => handle_offset!(-=, i, bytes, hour, minute, second),
        b'-' => handle_offset!(+=, i, bytes, hour, minute, second),
        _ => false,
    }
}

fn is_valid_datetime(datetime: &str) -> bool {
    // Find the position of 'T' or 't' separator
    let t_pos = match datetime.bytes().position(|b| b == b'T' || b == b't') {
        Some(pos) => pos,
        None => return false, // 'T' separator not found
    };

    // Split the string into date and time parts
    let (date_part, time_part) = datetime.split_at(t_pos);

    // Validate date part
    if !is_valid_date(date_part) {
        return false;
    }

    // Validate time part (skip the 'T' character)
    is_valid_time(&time_part[1..])
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

format_validator!(DateTimeValidator, "date-time");
impl Validate for DateTimeValidator {
    validate!("date-time");
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            is_valid_datetime(item)
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

fn is_valid_duration(duration: &str) -> bool {
    let bytes = duration.as_bytes();
    let len = bytes.len();

    if len < 2 || bytes[0] != b'P' {
        return false;
    }

    let mut i = 1;
    let mut has_component = false;
    let mut has_time = false;
    let mut last_date_unit = 0;
    let mut last_time_unit = 0;
    let mut has_weeks = false;
    let mut has_time_component = false;
    let mut seen_units = 0u8;

    let date_units = [b'Y', b'M', b'W', b'D'];
    let time_units = [b'H', b'M', b'S'];

    fn unit_index(units: &[u8], unit: u8) -> Option<usize> {
        units.iter().position(|&u| u == unit)
    }

    while i < len {
        if bytes[i] == b'T' {
            if has_time {
                return false;
            }
            has_time = true;
            i += 1;
            continue;
        }

        let start = i;
        while i < len && bytes[i].is_ascii_digit() {
            i += 1;
        }

        if i == start || i == len {
            return false;
        }

        let unit = bytes[i];

        if !has_time {
            if let Some(idx) = unit_index(&date_units, unit) {
                if unit == b'W' {
                    if has_component {
                        return false;
                    }
                    has_weeks = true;
                } else if has_weeks {
                    return false;
                }
                if idx < last_date_unit || (seen_units & (1 << idx) != 0) {
                    return false;
                }
                seen_units |= 1 << idx;
                last_date_unit = idx;
            } else {
                return false;
            }
        } else if let Some(idx) = unit_index(&time_units, unit) {
            if idx < last_time_unit || (seen_units & (1 << (idx + 4)) != 0) {
                return false;
            }
            seen_units |= 1 << (idx + 4);
            last_time_unit = idx;
            has_time_component = true;
        } else {
            return false;
        }

        has_component = true;
        i += 1;
    }

    has_component && (!has_time || has_time_component)
}

format_validator!(DurationValidator, "duration");
impl Validate for DurationValidator {
    validate!("duration");
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::String(item) = instance {
            is_valid_duration(item)
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

    use super::{is_valid_date, is_valid_datetime, is_valid_duration, is_valid_time};

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

    #[test_case("P1Y1Y")]
    #[test_case("PT1H1H")]
    fn test_invalid_duration(input: &str) {
        assert!(!is_valid_duration(input));
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

    #[test_case("2023-01-01", true; "valid regular date")]
    #[test_case("2020-02-29", true; "valid leap year date")]
    #[test_case("2021-02-28", true; "valid non-leap year date")]
    #[test_case("1900-02-28", true; "valid century non-leap year")]
    #[test_case("2000-02-29", true; "valid leap century year")]
    #[test_case("1999-12-31", true; "valid end of year date")]
    #[test_case("202-12-01", false; "invalid short year")]
    #[test_case("2023-1-01", false; "invalid short month")]
    #[test_case("2023-12-1", false; "invalid short day")]
    #[test_case("2023/12/01", false; "invalid separators")]
    #[test_case("2023-13-01", false; "invalid month too high")]
    #[test_case("2023-00-01", false; "invalid month too low")]
    #[test_case("2023-12-32", false; "invalid day too high")]
    #[test_case("2023-11-31", false; "invalid day for 30-day month")]
    #[test_case("2023-02-30", false; "invalid day for February in non-leap year")]
    #[test_case("2021-02-29", false; "invalid day for non-leap year")]
    #[test_case("2023-12-00", false; "invalid day too low")]
    #[test_case("99999-12-01", false; "year too long")]
    #[test_case("1900-02-29", false; "invalid leap century non-leap year")]
    #[test_case("2000-02-30", false; "invalid day for leap century year")]
    #[test_case("2400-02-29", true; "valid leap year in distant future")]
    #[test_case("0000-01-01", true; "valid boundary start date")]
    #[test_case("9999-12-31", true; "valid boundary end date")]
    #[test_case("aaaa-01-12", false; "Malformed (letters in year)")]
    #[test_case("2000-bb-12", false; "Malformed (letters in month)")]
    #[test_case("2000-01-cc", false; "Malformed (letters in day)")]
    fn test_is_valid_date(input: &str, expected: bool) {
        assert_eq!(is_valid_date(input), expected);
    }

    #[test_case("23:59:59Z", true; "valid time with Z")]
    #[test_case("00:00:00Z", true; "valid midnight time with Z")]
    #[test_case("12:30:45.123Z", true; "valid time with fractional seconds and Z")]
    #[test_case("23:59:60Z", true; "valid leap second UTC time")]
    #[test_case("12:30:45+01:00", true; "valid time with positive offset")]
    #[test_case("12:30:45-01:00", true; "valid time with negative offset")]
    #[test_case("23:59:60+00:00", true; "valid leap second with offset UTC 00:00")]
    #[test_case("23:59:59+01:00", true; "valid time with +01:00 offset")]
    #[test_case("23:59:59A", false; "invalid time with non-Z/non-offset letter")]
    #[test_case("12:3:45Z", false; "invalid time with missing digit in minute")]
    #[test_case("12:30:4Z", false; "invalid time with missing digit in second")]
    #[test_case("12-30-45Z", false; "invalid time with wrong separator")]
    #[test_case("12:30:45Z+01:00", false; "invalid time with Z and offset together")]
    #[test_case("12:30:45A01:00", false; "invalid time with wrong separator between time and offset")]
    #[test_case("12:30:45++01:00", false; "invalid double plus in offset")]
    #[test_case("12:30:45+01:60", false; "invalid minute in offset")]
    #[test_case("12:30:45+24:00", false; "invalid hour in offset")]
    #[test_case("12:30:45.", false; "invalid time with incomplete fractional second")]
    #[test_case("24:00:00Z", false; "invalid hour > 23")]
    #[test_case("12:60:00Z", false; "invalid minute > 59")]
    #[test_case("12:30:61Z", false; "invalid second > 60")]
    #[test_case("12:30:60+01:00", false; "invalid leap second with non-UTC offset")]
    #[test_case("23:59:60Z+01:00", false; "invalid leap second with non-zero offset")]
    #[test_case("23:59:60+00:30", false; "invalid leap second with non-zero minute offset")]
    #[test_case("23:59:60Z", true; "valid leap second at the end of day")]
    #[test_case("23:59:60+00:00", true; "valid leap second with zero offset")]
    #[test_case("ab:59:59Z", false; "invalid time with letters in hour")]
    #[test_case("23:ab:59Z", false; "invalid time with letters in minute")]
    #[test_case("23:59:abZ", false; "invalid time with letters in second")]
    #[test_case("23:59:59aZ", false; "invalid time with letter after seconds")]
    #[test_case("12:30:45+ab:00", false; "invalid offset hour with letters")]
    #[test_case("12:30:45+01:ab", false; "invalid offset minute with letters")]
    #[test_case("12:30:45.abcZ", false; "invalid fractional seconds with letters")]
    fn test_is_valid_time(input: &str, expected: bool) {
        assert_eq!(is_valid_time(input), expected);
    }

    #[test]
    fn test_is_valid_datetime() {
        assert!(!is_valid_datetime(""));
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

    #[test]
    fn test_is_valid_datetime_panic() {
        is_valid_datetime("2624-04-25t23:14:04-256\x112");
    }
}
