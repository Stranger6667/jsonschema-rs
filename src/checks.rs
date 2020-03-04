use crate::error::ValidationError;
use crate::keywords::ValidationResult;
use chrono::{DateTime, NaiveDate};
use regex::Regex;
use std::net::IpAddr;
use std::str::FromStr;
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

pub(crate) fn date(instance: &str) -> ValidationResult {
    if NaiveDate::parse_from_str(instance, "%Y-%m-%d").is_err() {
        return Err(ValidationError::format(instance.to_owned(), "date"));
    }
    Ok(())
}

pub(crate) fn datetime(instance: &str) -> ValidationResult {
    if DateTime::parse_from_rfc3339(instance).is_err() {
        return Err(ValidationError::format(instance.to_owned(), "date-time"));
    }
    Ok(())
}

pub(crate) fn email(instance: &str) -> ValidationResult {
    if !instance.contains('@') {
        return Err(ValidationError::format(instance.to_owned(), "email"));
    }
    Ok(())
}

pub(crate) fn hostname(instance: &str) -> ValidationResult {
    if instance.ends_with('-')
        || instance.starts_with('-')
        || instance.is_empty()
        || instance.chars().count() > 255
        || instance
            .chars()
            .any(|c| !(c.is_alphanumeric() || c == '-' || c == '.'))
        || instance.split('.').any(|part| part.chars().count() > 63)
    {
        return Err(ValidationError::format(instance.to_owned(), "hostname"));
    }
    Ok(())
}

pub(crate) fn ipv4(instance: &str) -> ValidationResult {
    if !match IpAddr::from_str(instance) {
        Ok(i) => match i {
            IpAddr::V4(_) => true,
            IpAddr::V6(_) => false,
        },
        Err(_) => false,
    } {
        return Err(ValidationError::format(instance.to_owned(), "ipv4"));
    }
    Ok(())
}

pub(crate) fn ipv6(instance: &str) -> ValidationResult {
    if !match IpAddr::from_str(instance) {
        Ok(i) => match i {
            IpAddr::V4(_) => false,
            IpAddr::V6(_) => true,
        },
        Err(_) => false,
    } {
        return Err(ValidationError::format(instance.to_owned(), "ipv6"));
    }
    Ok(())
}

pub(crate) fn iri(instance: &str) -> ValidationResult {
    if Url::from_str(instance).is_err() {
        return Err(ValidationError::format(instance.to_owned(), "iri"));
    }
    Ok(())
}

pub(crate) fn iri_reference(instance: &str) -> ValidationResult {
    if !IRI_REFERENCE_RE.is_match(instance) {
        return Err(ValidationError::format(
            instance.to_owned(),
            "iri-reference",
        ));
    }
    Ok(())
}

pub(crate) fn json_pointer(instance: &str) -> ValidationResult {
    if !JSON_POINTER_RE.is_match(instance) {
        return Err(ValidationError::format(instance.to_owned(), "json-pointer"));
    }
    Ok(())
}

pub(crate) fn regex(instance: &str) -> ValidationResult {
    if Regex::new(instance).is_err() {
        return Err(ValidationError::format(instance.to_owned(), "regex"));
    }
    Ok(())
}

pub(crate) fn relative_json_pointer(instance: &str) -> ValidationResult {
    if !RELATIVE_JSON_POINTER_RE.is_match(instance) {
        return Err(ValidationError::format(
            instance.to_owned(),
            "relative-json-pointer",
        ));
    }
    Ok(())
}

pub(crate) fn time(instance: &str) -> ValidationResult {
    if !TIME_RE.is_match(instance) {
        return Err(ValidationError::format(instance.to_owned(), "time"));
    }
    Ok(())
}

pub(crate) fn uri_reference(instance: &str) -> ValidationResult {
    if !URI_REFERENCE_RE.is_match(instance) {
        return Err(ValidationError::format(
            instance.to_owned(),
            "uri-reference",
        ));
    }
    Ok(())
}

pub(crate) fn uri_template(instance: &str) -> ValidationResult {
    if !URI_TEMPLATE_RE.is_match(instance) {
        return Err(ValidationError::format(instance.to_owned(), "uri-template"));
    }
    Ok(())
}
