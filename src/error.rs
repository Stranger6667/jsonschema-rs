use serde_json::Value;
use std::fmt::{Error, Formatter};
use std::iter::{empty, once};
use std::string::FromUtf8Error;
use std::{error, fmt, io};

#[derive(Debug, PartialEq)]
pub enum CompilationError {
    SchemaError,
}

impl From<regex::Error> for CompilationError {
    fn from(_: regex::Error) -> Self {
        CompilationError::SchemaError
    }
}
impl From<url::ParseError> for CompilationError {
    fn from(_: url::ParseError) -> Self {
        CompilationError::SchemaError
    }
}

/// An error that can occur during validation.
#[derive(Debug)]
pub struct ValidationError {
    kind: ValidationErrorKind,
}

pub type ErrorIterator<'a> = Box<dyn Iterator<Item = ValidationError> + 'a>;

// Empty iterator means no error happened
pub(crate) fn no_error<'a>() -> ErrorIterator<'a> {
    Box::new(empty())
}
// A wrapper for one error
pub(crate) fn error<'a>(instance: ValidationError) -> ErrorIterator<'a> {
    Box::new(once(instance))
}

/// Kinds of errors that may happen during validation
#[derive(Debug)]
pub enum ValidationErrorKind {
    /// The input array contain more items than expected.
    AdditionalItems { items: Vec<Value>, limit: usize },
    /// The input value is not valid under any of the given schemas.
    AnyOf(Value),
    /// The input value doesn't match expected constant.
    Constant(String),
    /// The input array doesn't contain items conforming to the specified schema.
    Contains(Value),
    /// The input value doesn't match any of specified options.
    Enum { instance: Value, options: Value },
    /// Value is too large.
    ExclusiveMaximum { instance: f64, limit: f64 },
    /// Value is too small.
    ExclusiveMinimum { instance: f64, limit: f64 },
    /// Everything is invalid for `false` schema.
    FalseSchema(Value),
    /// If the referenced file is not found during ref resolution.
    FileNotFound(io::Error),
    /// When the input doesn't match to the specified format.
    Format { instance: String, format: String },
    /// May happen in `contentEncoding` validation if `base64` encoded data is invalid.
    FromUtf8(FromUtf8Error),
    /// May happen during ref resolution when remote document is not a valid JSON.
    JSONParse(serde_json::Error),
    /// `ref` value is not valid.
    InvalidReference(String),
    /// Too many items in an array.
    MaxItems(Value),
    /// Value is too large.
    Maximum { instance: f64, limit: f64 },
    /// String is too long.
    MaxLength(String),
    /// Too many properties in an object.
    MaxProperties(Value),
    /// Too few items in an array.
    MinItems(Value),
    /// Value is too small.
    Minimum { instance: f64, limit: f64 },
    /// String is too short.
    MinLength(String),
    /// Not enough properties in an object.
    MinProperties(Value),
    /// When some number is not a multiple of another number.
    MultipleOf { instance: f64, multiple_of: f64 },
    /// Negated schema failed validation.
    Not { instance: Value, schema: Value },
    /// The given schema is valid under more than one of the given schemas.
    OneOfMultipleValid(Value),
    /// The given schema is not valid under any on the given schemas.
    OneOfNotValid(Value),
    /// When the input doesn't match to a pattern.
    Pattern { instance: String, pattern: String },
    /// When a required property is missing.
    Required(String),
    /// Resolved schema failed to compile.
    Schema,
    /// When the input value doesn't match one or multiple required types.
    Type { instance: Value, kind: TypeKind },
    /// When the input array has non-unique elements.
    UniqueItems(Value),
    /// Reference contains unknown scheme.
    UnknownReferenceScheme(String),
}

/// For faster error handling in "type" keyword validator we have this enum, to match
/// with it instead of a string.
#[derive(Debug, Clone)]
pub enum PrimitiveType {
    Integer,
    Null,
    Boolean,
    String,
    Array,
    Object,
    Number,
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            PrimitiveType::Integer => write!(f, "integer"),
            PrimitiveType::Null => write!(f, "null"),
            PrimitiveType::Boolean => write!(f, "boolean"),
            PrimitiveType::String => write!(f, "string"),
            PrimitiveType::Array => write!(f, "array"),
            PrimitiveType::Object => write!(f, "object"),
            PrimitiveType::Number => write!(f, "number"),
        }
    }
}

#[derive(Debug)]
pub enum TypeKind {
    Single(PrimitiveType),
    Multiple(Vec<PrimitiveType>),
}

/// Shortcuts for creation of specific error kinds.
impl<'a> ValidationError {
    pub(crate) fn additional_items(items: Vec<Value>, limit: usize) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::AdditionalItems { items, limit },
        })
    }
    pub(crate) fn any_of(instance: Value) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::AnyOf(instance),
        })
    }
    pub(crate) fn constant(message: String) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::Constant(message),
        })
    }
    pub(crate) fn contains(instance: Value) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::Contains(instance),
        })
    }
    pub(crate) fn enumeration(instance: Value, options: Value) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::Enum { instance, options },
        })
    }
    pub(crate) fn exclusive_maximum(instance: f64, limit: f64) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::ExclusiveMaximum { instance, limit },
        })
    }
    pub(crate) fn exclusive_minimum(instance: f64, limit: f64) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::ExclusiveMinimum { instance, limit },
        })
    }
    pub(crate) fn false_schema(instance: Value) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::FalseSchema(instance),
        })
    }
    pub(crate) fn file_not_found(err: io::Error) -> ValidationError {
        ValidationError {
            kind: ValidationErrorKind::FileNotFound(err),
        }
    }
    pub(crate) fn format(instance: String, format: String) -> ValidationError {
        ValidationError {
            kind: ValidationErrorKind::Format { instance, format },
        }
    }
    pub(crate) fn from_utf8(err: FromUtf8Error) -> ValidationError {
        ValidationError {
            kind: ValidationErrorKind::FromUtf8(err),
        }
    }
    pub(crate) fn json_parse(err: serde_json::Error) -> ValidationError {
        ValidationError {
            kind: ValidationErrorKind::JSONParse(err),
        }
    }
    pub(crate) fn invalid_reference(reference: String) -> ValidationError {
        ValidationError {
            kind: ValidationErrorKind::InvalidReference(reference),
        }
    }
    pub(crate) fn max_items(instance: Value) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::MaxItems(instance),
        })
    }
    pub(crate) fn maximum(instance: f64, limit: f64) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::Maximum { instance, limit },
        })
    }
    pub(crate) fn max_length(instance: String) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::MaxLength(instance),
        })
    }
    pub(crate) fn max_properties(instance: Value) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::MaxProperties(instance),
        })
    }
    pub(crate) fn min_items(instance: Value) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::MinItems(instance),
        })
    }
    pub(crate) fn minimum(instance: f64, limit: f64) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::Minimum { instance, limit },
        })
    }
    pub(crate) fn min_length(instance: String) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::MinLength(instance),
        })
    }
    pub(crate) fn min_properties(instance: Value) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::MinProperties(instance),
        })
    }
    pub(crate) fn multiple_of(instance: f64, multiple_of: f64) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::MultipleOf {
                instance,
                multiple_of,
            },
        })
    }
    pub(crate) fn not(instance: Value, schema: Value) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::Not { instance, schema },
        })
    }
    pub(crate) fn one_of_multiple_valid(instance: Value) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::OneOfMultipleValid(instance),
        })
    }
    pub(crate) fn one_of_not_valid(instance: Value) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::OneOfNotValid(instance),
        })
    }
    pub(crate) fn pattern(instance: String, pattern: String) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::Pattern { instance, pattern },
        })
    }
    pub(crate) fn required(property: String) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::Required(property),
        })
    }
    pub(crate) fn schema() -> ValidationError {
        ValidationError {
            kind: ValidationErrorKind::Schema,
        }
    }
    pub(crate) fn single_type_error(
        instance: Value,
        type_name: PrimitiveType,
    ) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::Type {
                instance,
                kind: TypeKind::Single(type_name),
            },
        })
    }
    pub(crate) fn multiple_type_error(
        instance: Value,
        types: Vec<PrimitiveType>,
    ) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::Type {
                instance,
                kind: TypeKind::Multiple(types),
            },
        })
    }
    pub(crate) fn unique_items(instance: Value) -> ErrorIterator<'a> {
        error(ValidationError {
            kind: ValidationErrorKind::UniqueItems(instance),
        })
    }
    pub(crate) fn unknown_reference_scheme(scheme: String) -> ValidationError {
        ValidationError {
            kind: ValidationErrorKind::UnknownReferenceScheme(scheme),
        }
    }
}

impl From<CompilationError> for ValidationError {
    fn from(_: CompilationError) -> Self {
        ValidationError::schema()
    }
}
impl error::Error for ValidationError {}
impl From<serde_json::Error> for ValidationError {
    fn from(err: serde_json::Error) -> Self {
        ValidationError::json_parse(err)
    }
}
impl From<io::Error> for ValidationError {
    fn from(err: io::Error) -> Self {
        ValidationError::file_not_found(err)
    }
}
impl From<FromUtf8Error> for ValidationError {
    fn from(err: FromUtf8Error) -> Self {
        ValidationError::from_utf8(err)
    }
}

/// Textual representation of various validation errors.
impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind {
            ValidationErrorKind::Schema => write!(f, "Schema error"),
            ValidationErrorKind::JSONParse(ref err) => write!(f, "{}", err),
            ValidationErrorKind::FileNotFound(ref err) => write!(f, "{}", err),
            ValidationErrorKind::UnknownReferenceScheme(ref schema) => {
                write!(f, "Unknown schema: {}", schema)
            }
            ValidationErrorKind::Format {
                ref instance,
                ref format,
            } => write!(f, "'{}' is not a '{}'", instance, format),
            ValidationErrorKind::AdditionalItems { ref items, limit } => {
                let extras: Vec<&Value> = items.iter().skip(limit).collect();
                let verb = {
                    if extras.len() == 1 {
                        "was"
                    } else {
                        "were"
                    }
                };
                write!(
                    f,
                    "Additional items are not allowed ({} {} unexpected)",
                    extras
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(", "),
                    verb
                )
            }
            ValidationErrorKind::AnyOf(ref instance) => write!(
                f,
                "'{}' is not valid under any of the given schemas",
                instance
            ),
            ValidationErrorKind::Contains(ref instance) => {
                write!(f, "None of '{}' are valid under the given schema", instance)
            }
            ValidationErrorKind::Constant(ref message) => write!(f, "{}", message),
            ValidationErrorKind::FromUtf8(ref err) => write!(f, "{}", err),
            ValidationErrorKind::Enum {
                ref instance,
                ref options,
            } => write!(f, "'{}' is not one of '{}'", instance, options),
            ValidationErrorKind::ExclusiveMaximum { instance, limit } => write!(
                f,
                "{} is greater than or equal to the maximum of {}",
                instance, limit
            ),
            ValidationErrorKind::ExclusiveMinimum { instance, limit } => write!(
                f,
                "{} is less than or equal to the minimum of {}",
                instance, limit
            ),
            ValidationErrorKind::FalseSchema(ref instance) => {
                write!(f, "False schema does not allow '{}'", instance)
            }
            ValidationErrorKind::InvalidReference(ref path) => {
                write!(f, "Invalid reference: {}", path)
            }
            ValidationErrorKind::Maximum { instance, limit } => {
                write!(f, "{} is greater than the maximum of {}", instance, limit)
            }
            ValidationErrorKind::Minimum { instance, limit } => {
                write!(f, "{} is less than the minimum of {}", instance, limit)
            }
            ValidationErrorKind::MaxLength(ref instance) => write!(f, "'{}' is too long", instance),
            ValidationErrorKind::MinLength(ref instance) => {
                write!(f, "'{}' is too short", instance)
            }
            ValidationErrorKind::MaxItems(ref instance) => write!(f, "{} is too long", instance),
            ValidationErrorKind::MinItems(ref instance) => write!(f, "{} is too short", instance),
            ValidationErrorKind::MaxProperties(ref instance) => {
                write!(f, "{} has too many properties", instance)
            }
            ValidationErrorKind::MinProperties(ref instance) => {
                write!(f, "{} does not have enough properties", instance)
            }
            ValidationErrorKind::Not {
                ref instance,
                ref schema,
            } => write!(f, "{} is not allowed for {}", schema, instance),
            ValidationErrorKind::OneOfNotValid(ref instance) => write!(
                f,
                "'{}' is not valid under any of the given schemas",
                instance
            ),
            ValidationErrorKind::OneOfMultipleValid(ref instance) => write!(
                f,
                "'{}' is valid under more than one of the given schemas",
                instance
            ),
            ValidationErrorKind::Pattern {
                ref instance,
                ref pattern,
            } => write!(f, "'{}' does not match '{}'", instance, pattern),
            ValidationErrorKind::Required(ref property) => {
                write!(f, "'{}' is a required property", property)
            }
            ValidationErrorKind::MultipleOf {
                instance,
                multiple_of,
            } => write!(f, "{} is not a multiple of {}", instance, multiple_of),
            ValidationErrorKind::UniqueItems(ref instance) => {
                write!(f, "'{}' has non-unique elements", instance)
            }
            ValidationErrorKind::Type {
                ref instance,
                ref kind,
            } => match kind {
                TypeKind::Single(ref type_) => {
                    write!(f, "'{}' is not of type '{}'", instance, type_)
                }
                TypeKind::Multiple(ref types) => write!(
                    f,
                    "'{}' is not of types '{}'",
                    instance,
                    types
                        .iter()
                        .map(|t| format!("{}", t))
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn type_error() {
        let instance = json!(42);
        let err = ValidationError::single_type_error(instance, PrimitiveType::String)
            .next()
            .unwrap();
        let repr = format!("{}", err);
        assert_eq!(repr, "'42' is not of type 'string'")
    }
}
