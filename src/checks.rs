use crate::error::{error, no_error, ErrorIterator, ValidationError};
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

pub(crate) fn date(instance: &str) -> ErrorIterator {
    if NaiveDate::parse_from_str(instance, "%Y-%m-%d").is_err() {
        return error(ValidationError::format(instance.to_owned(), "date"));
    }
    no_error()
}

pub(crate) fn datetime(instance: &str) -> ErrorIterator {
    if DateTime::parse_from_rfc3339(instance).is_err() {
        return error(ValidationError::format(instance.to_owned(), "date-time"));
    }
    no_error()
}

pub(crate) fn email(instance: &str) -> ErrorIterator {
    if !instance.contains('@') {
        return error(ValidationError::format(instance.to_owned(), "email"));
    }
    no_error()
}

pub(crate) fn hostname(instance: &str) -> ErrorIterator {
    if instance.ends_with('-')
        || instance.starts_with('-')
        || instance.is_empty()
        || instance.chars().count() > 255
        || instance
            .chars()
            .any(|c| !(c.is_alphanumeric() || c == '-' || c == '.'))
        || instance.split('.').any(|part| part.chars().count() > 63)
    {
        return error(ValidationError::format(instance.to_owned(), "hostname"));
    }
    no_error()
}

pub(crate) fn ipv4(instance: &str) -> ErrorIterator {
    if !match IpAddr::from_str(instance) {
        Ok(i) => match i {
            IpAddr::V4(_) => true,
            IpAddr::V6(_) => false,
        },
        Err(_) => false,
    } {
        return error(ValidationError::format(instance.to_owned(), "ipv4"));
    }
    no_error()
}

pub(crate) fn ipv6(instance: &str) -> ErrorIterator {
    if !match IpAddr::from_str(instance) {
        Ok(i) => match i {
            IpAddr::V4(_) => false,
            IpAddr::V6(_) => true,
        },
        Err(_) => false,
    } {
        return error(ValidationError::format(instance.to_owned(), "ipv6"));
    }
    no_error()
}

pub(crate) fn iri(instance: &str) -> ErrorIterator {
    if Url::from_str(instance).is_err() {
        return error(ValidationError::format(instance.to_owned(), "iri"));
    }
    no_error()
}

pub(crate) fn iri_reference(instance: &str) -> ErrorIterator {
    if !IRI_REFERENCE_RE.is_match(instance) {
        return error(ValidationError::format(
            instance.to_owned(),
            "iri-reference",
        ));
    }
    no_error()
}

pub(crate) fn json_pointer(instance: &str) -> ErrorIterator {
    if !JSON_POINTER_RE.is_match(instance) {
        return error(ValidationError::format(instance.to_owned(), "json-pointer"));
    }
    no_error()
}

pub(crate) fn regex(instance: &str) -> ErrorIterator {
    if Regex::new(instance).is_err() {
        return error(ValidationError::format(instance.to_owned(), "regex"));
    }
    no_error()
}

pub(crate) fn relative_json_pointer(instance: &str) -> ErrorIterator {
    if !RELATIVE_JSON_POINTER_RE.is_match(instance) {
        return error(ValidationError::format(
            instance.to_owned(),
            "relative-json-pointer",
        ));
    }
    no_error()
}

pub(crate) fn time(instance: &str) -> ErrorIterator {
    if !TIME_RE.is_match(instance) {
        return error(ValidationError::format(instance.to_owned(), "time"));
    }
    no_error()
}

pub(crate) fn uri_reference(instance: &str) -> ErrorIterator {
    if !URI_REFERENCE_RE.is_match(instance) {
        return error(ValidationError::format(
            instance.to_owned(),
            "uri-reference",
        ));
    }
    no_error()
}

pub(crate) fn uri_template(instance: &str) -> ErrorIterator {
    if !URI_TEMPLATE_RE.is_match(instance) {
        return error(ValidationError::format(instance.to_owned(), "uri-template"));
    }
    no_error()
}
