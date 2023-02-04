use crate::{value_type::ValueType, vocabularies::Keyword};
use serde_json::Value;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Type {
    value: ValueType,
}

impl Type {
    pub(crate) fn build(value: ValueType) -> Keyword {
        Self { value }.into()
    }
}

impl Type {
    pub(crate) fn is_valid(&self, instance: &Value) -> bool {
        match self.value {
            ValueType::Array => instance.is_array(),
            ValueType::Boolean => instance.is_boolean(),
            ValueType::Integer => instance.is_i64() | instance.is_u64(),
            ValueType::Null => instance.is_null(),
            ValueType::Number => instance.is_number(),
            ValueType::Object => instance.is_object(),
            ValueType::String => instance.is_string(),
        }
    }
}
