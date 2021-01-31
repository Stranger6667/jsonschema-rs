use ahash::AHashMap;
use serde_json::{from_str, Value};

pub(crate) type ContentMediaTypeCheckType = fn(&str) -> bool;

pub(crate) fn is_json(instance_string: &str) -> bool {
    from_str::<Value>(instance_string).is_ok()
}

lazy_static::lazy_static! {
    pub(crate) static ref DEFAULT_CONTENT_MEDIA_TYPE_CHECKS: AHashMap<&'static str, ContentMediaTypeCheckType> = {
        let mut map: AHashMap<&'static str, ContentMediaTypeCheckType> = AHashMap::with_capacity(1);
        map.insert("application/json", is_json);
        map
    };
}
