use crate::error::{error, no_error, ErrorIterator, ValidationError};
use serde_json::{from_str, Value};
use std::collections::HashMap;

pub(crate) type ContentMediaTypeCheckType = for<'a> fn(&'a Value, &str) -> ErrorIterator<'a>;

pub(crate) fn is_json<'a>(instance: &'a Value, instance_string: &str) -> ErrorIterator<'a> {
    if from_str::<Value>(instance_string).is_err() {
        return error(ValidationError::format(instance, "application/json"));
    }
    no_error()
}

lazy_static::lazy_static! {
    pub(crate) static ref DEFAULT_CONTENT_MEDIA_TYPE_CHECKS: HashMap<&'static str, ContentMediaTypeCheckType> = {
        let mut map: HashMap<&'static str, ContentMediaTypeCheckType> = HashMap::with_capacity(1);
        map.insert("application/json", is_json);
        map
    };
}
