use std::{convert::TryFrom, fmt};

/// For faster error handling in "type" keyword validator we have this enum, to match
/// with it instead of a string.
#[derive(Debug, Clone)]
pub enum PrimitiveType {
    Array,
    Boolean,
    Integer,
    Null,
    Number,
    Object,
    String,
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimitiveType::Array => write!(f, "array"),
            PrimitiveType::Boolean => write!(f, "boolean"),
            PrimitiveType::Integer => write!(f, "integer"),
            PrimitiveType::Null => write!(f, "null"),
            PrimitiveType::Number => write!(f, "number"),
            PrimitiveType::Object => write!(f, "object"),
            PrimitiveType::String => write!(f, "string"),
        }
    }
}

impl TryFrom<&str> for PrimitiveType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "array" => Ok(PrimitiveType::Array),
            "boolean" => Ok(PrimitiveType::Boolean),
            "integer" => Ok(PrimitiveType::Integer),
            "null" => Ok(PrimitiveType::Null),
            "number" => Ok(PrimitiveType::Number),
            "object" => Ok(PrimitiveType::Object),
            "string" => Ok(PrimitiveType::String),
            _ => Err(()),
        }
    }
}
