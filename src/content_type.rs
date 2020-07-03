use crate::error::{error, no_error, ErrorIterator, ValidationError};
use serde_json::Value;

pub(crate) type ContentTypeCheckType = for<'a> fn(&'a Value, &str) -> ErrorIterator<'a>;
pub(crate) type ContentTypeConvertType =
    for<'a> fn(&'a Value, &str) -> Result<String, ValidationError<'a>>;

pub fn is_base64<'a>(instance: &'a Value, instance_string: &str) -> ErrorIterator<'a> {
    if base64::decode(instance_string).is_err() {
        return error(ValidationError::format(instance, "base64"));
    }
    no_error()
}

pub fn from_base64<'a>(
    instance: &'a Value,
    instance_string: &str,
) -> Result<String, ValidationError<'a>> {
    match base64::decode(instance_string) {
        Ok(value) => Ok(String::from_utf8(value)?),
        Err(_) => Err(ValidationError::format(instance, "base64")),
    }
}

pub(crate) static CONTENT_TYPE_CHECK_BUILDER: &[(&str, ContentTypeCheckType)] =
    &[("base64", is_base64)];

pub(crate) static CONTENT_TYPE_CONVERT_BUILDER: &[(&str, ContentTypeConvertType)] =
    &[("base64", from_base64)];
