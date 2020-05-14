use serde_json::Value;
use std::{
    borrow::Cow,
    error, fmt,
    fmt::{Error, Formatter},
    io,
    iter::{empty, once},
    string::FromUtf8Error,
};

#[derive(Debug, PartialEq)]
pub enum CompilationError {
    SchemaError,
}

impl error::Error for CompilationError {}

impl fmt::Display for CompilationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Schema compilation error")
    }
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
pub struct ValidationError<'a> {
    instance: Cow<'a, Value>,
    kind: ValidationErrorKind,
}

pub type ErrorIterator<'a> = Box<dyn Iterator<Item = ValidationError<'a>> + Sync + Send + 'a>;

// Empty iterator means no error happened
pub(crate) fn no_error<'a>() -> ErrorIterator<'a> {
    Box::new(empty())
}
// A wrapper for one error
pub(crate) fn error(instance: ValidationError) -> ErrorIterator {
    Box::new(once(instance))
}

/// Kinds of errors that may happen during validation
#[derive(Debug)]
pub enum ValidationErrorKind {
    /// The input array contain more items than expected.
    AdditionalItems { limit: usize },
    /// The input value is not valid under any of the given schemas.
    AnyOf,
    /// The input value doesn't match expected constant.
    Constant { expected_value: Value },
    /// The input array doesn't contain items conforming to the specified schema.
    Contains,
    /// The input value doesn't match any of specified options.
    Enum { options: Value },
    /// Value is too large.
    ExclusiveMaximum { limit: f64 },
    /// Value is too small.
    ExclusiveMinimum { limit: f64 },
    /// Everything is invalid for `false` schema.
    FalseSchema,
    /// If the referenced file is not found during ref resolution.
    FileNotFound { error: io::Error },
    /// When the input doesn't match to the specified format.
    Format { format: String },
    /// May happen in `contentEncoding` validation if `base64` encoded data is invalid.
    FromUtf8 { error: FromUtf8Error },
    /// May happen during ref resolution when remote document is not a valid JSON.
    JSONParse { error: serde_json::Error },
    /// `ref` value is not valid.
    InvalidReference { reference: String },
    /// Too many items in an array.
    MaxItems { limit: usize },
    /// Value is too large.
    Maximum { limit: f64 },
    /// String is too long.
    MaxLength { limit: usize },
    /// Too many properties in an object.
    MaxProperties { limit: usize },
    /// Too few items in an array.
    MinItems { limit: usize },
    /// Value is too small.
    Minimum { limit: f64 },
    /// String is too short.
    MinLength { limit: usize },
    /// Not enough properties in an object.
    MinProperties { limit: usize },
    /// When some number is not a multiple of another number.
    MultipleOf { multiple_of: f64 },
    /// Negated schema failed validation.
    Not { schema: Value },
    /// The given schema is valid under more than one of the given schemas.
    OneOfMultipleValid,
    /// The given schema is not valid under any on the given schemas.
    OneOfNotValid,
    /// When the input doesn't match to a pattern.
    Pattern { pattern: String },
    /// When a required property is missing.
    Required { property: String },
    /// Resolved schema failed to compile.
    Schema,
    /// When the input value doesn't match one or multiple required types.
    Type { kind: TypeKind },
    /// When the input array has non-unique elements.
    UniqueItems,
    /// Reference contains unknown scheme.
    UnknownReferenceScheme { scheme: String },
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
impl<'a> ValidationError<'a> {
    pub(crate) fn into_owned(self) -> ValidationError<'static> {
        ValidationError {
            instance: Cow::Owned(self.instance.into_owned()),
            kind: self.kind,
        }
    }

    pub(crate) fn additional_items(instance: &'a Value, limit: usize) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::AdditionalItems { limit },
        }
    }
    pub(crate) fn any_of(instance: &'a Value) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::AnyOf,
        }
    }
    pub(crate) fn constant(instance: &'a Value, expected_value: &Value) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Constant {
                expected_value: expected_value.clone(),
            },
        }
    }
    pub(crate) fn contains(instance: &'a Value) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Contains,
        }
    }
    pub(crate) fn enumeration(instance: &'a Value, options: &Value) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Enum {
                options: options.clone(),
            },
        }
    }
    pub(crate) fn exclusive_maximum(instance: &'a Value, limit: f64) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::ExclusiveMaximum { limit },
        }
    }
    pub(crate) fn exclusive_minimum(instance: &'a Value, limit: f64) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::ExclusiveMinimum { limit },
        }
    }
    pub(crate) fn false_schema(instance: &'a Value) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::FalseSchema,
        }
    }
    pub(crate) fn file_not_found(error: io::Error) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Owned(Value::Null),
            kind: ValidationErrorKind::FileNotFound { error },
        }
    }
    pub(crate) fn format(instance: &'a Value, format: String) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Format { format },
        }
    }
    pub(crate) fn from_utf8(error: FromUtf8Error) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Owned(Value::Null),
            kind: ValidationErrorKind::FromUtf8 { error },
        }
    }
    pub(crate) fn json_parse(error: serde_json::Error) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Owned(Value::Null),
            kind: ValidationErrorKind::JSONParse { error },
        }
    }
    pub(crate) fn invalid_reference(reference: String) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Owned(Value::Null),
            kind: ValidationErrorKind::InvalidReference { reference },
        }
    }
    pub(crate) fn max_items(instance: &'a Value, limit: usize) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::MaxItems { limit },
        }
    }
    pub(crate) fn maximum(instance: &'a Value, limit: f64) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Maximum { limit },
        }
    }
    pub(crate) fn max_length(instance: &'a Value, limit: usize) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::MaxLength { limit },
        }
    }
    pub(crate) fn max_properties(instance: &'a Value, limit: usize) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::MaxProperties { limit },
        }
    }
    pub(crate) fn min_items(instance: &'a Value, limit: usize) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::MinItems { limit },
        }
    }
    pub(crate) fn minimum(instance: &'a Value, limit: f64) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Minimum { limit },
        }
    }
    pub(crate) fn min_length(instance: &'a Value, limit: usize) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::MinLength { limit },
        }
    }
    pub(crate) fn min_properties(instance: &'a Value, limit: usize) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::MinProperties { limit },
        }
    }
    pub(crate) fn multiple_of(instance: &'a Value, multiple_of: f64) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::MultipleOf { multiple_of },
        }
    }
    pub(crate) fn not(instance: &'a Value, schema: Value) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Not { schema },
        }
    }
    pub(crate) fn one_of_multiple_valid(instance: &'a Value) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::OneOfMultipleValid,
        }
    }
    pub(crate) fn one_of_not_valid(instance: &'a Value) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::OneOfNotValid,
        }
    }
    pub(crate) fn pattern(instance: &'a Value, pattern: String) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Pattern { pattern },
        }
    }
    pub(crate) fn required(instance: &'a Value, property: String) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Required { property },
        }
    }
    pub(crate) fn schema() -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Owned(Value::Null),
            kind: ValidationErrorKind::Schema,
        }
    }
    pub(crate) fn single_type_error(
        instance: &'a Value,
        type_name: PrimitiveType,
    ) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Type {
                kind: TypeKind::Single(type_name),
            },
        }
    }
    pub(crate) fn multiple_type_error(
        instance: &'a Value,
        types: Vec<PrimitiveType>,
    ) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Type {
                kind: TypeKind::Multiple(types),
            },
        }
    }
    pub(crate) fn unique_items(instance: &'a Value) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::UniqueItems,
        }
    }
    pub(crate) fn unknown_reference_scheme(scheme: String) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Owned(Value::Null),
            kind: ValidationErrorKind::UnknownReferenceScheme { scheme },
        }
    }
}

impl<'a> From<CompilationError> for ValidationError<'a> {
    fn from(_: CompilationError) -> Self {
        ValidationError::schema()
    }
}
impl<'a> error::Error for ValidationError<'a> {}
impl<'a> From<serde_json::Error> for ValidationError<'a> {
    fn from(err: serde_json::Error) -> Self {
        ValidationError::json_parse(err)
    }
}
impl<'a> From<io::Error> for ValidationError<'a> {
    fn from(err: io::Error) -> Self {
        ValidationError::file_not_found(err)
    }
}
impl<'a> From<FromUtf8Error> for ValidationError<'a> {
    fn from(err: FromUtf8Error) -> Self {
        ValidationError::from_utf8(err)
    }
}

/// Textual representation of various validation errors.
impl<'a> fmt::Display for ValidationError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ValidationErrorKind::Schema => write!(f, "Schema error"),
            ValidationErrorKind::JSONParse { error } => write!(f, "{}", error),
            ValidationErrorKind::FileNotFound { error } => write!(f, "{}", error),
            ValidationErrorKind::UnknownReferenceScheme { scheme } => {
                write!(f, "Unknown scheme: {}", scheme)
            }
            ValidationErrorKind::Format { format } => {
                write!(f, "'{}' is not a '{}'", self.instance, format)
            }
            ValidationErrorKind::AdditionalItems { limit } => {
                // It's safe to unwrap here as ValidationErrorKind::AdditionalItems is reported only in
                // case of arrays with more items than expected
                let extras: Vec<&Value> = self
                    .instance
                    .as_array()
                    .unwrap()
                    .iter()
                    .skip(*limit)
                    .collect();
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
            ValidationErrorKind::AnyOf => write!(
                f,
                "'{}' is not valid under any of the given schemas",
                self.instance
            ),
            ValidationErrorKind::Contains => write!(
                f,
                "None of '{}' are valid under the given schema",
                self.instance
            ),
            ValidationErrorKind::Constant { expected_value } => {
                write!(f, "'{}' was expected", expected_value)
            }
            ValidationErrorKind::FromUtf8 { error } => write!(f, "{}", error),
            ValidationErrorKind::Enum { options } => {
                write!(f, "'{}' is not one of '{}'", self.instance, options)
            }
            ValidationErrorKind::ExclusiveMaximum { limit } => write!(
                f,
                "{} is greater than or equal to the maximum of {}",
                self.instance, limit
            ),
            ValidationErrorKind::ExclusiveMinimum { limit } => write!(
                f,
                "{} is less than or equal to the minimum of {}",
                self.instance, limit
            ),
            ValidationErrorKind::FalseSchema => {
                write!(f, "False schema does not allow '{}'", self.instance)
            }
            ValidationErrorKind::InvalidReference { reference } => {
                write!(f, "Invalid reference: {}", reference)
            }
            ValidationErrorKind::Maximum { limit } => write!(
                f,
                "{} is greater than the maximum of {}",
                self.instance, limit
            ),
            ValidationErrorKind::Minimum { limit } => {
                write!(f, "{} is less than the minimum of {}", self.instance, limit)
            }
            ValidationErrorKind::MaxLength { limit } => write!(
                f,
                "'{}' is longer than {} character{}",
                self.instance,
                limit,
                if *limit == 1 { "" } else { "s" }
            ),
            ValidationErrorKind::MinLength { limit } => write!(
                f,
                "'{}' is shorter than {} character{}",
                self.instance,
                limit,
                if *limit == 1 { "" } else { "s" }
            ),
            ValidationErrorKind::MaxItems { limit } => write!(
                f,
                "{} has more than {} item{}",
                self.instance,
                limit,
                if *limit == 1 { "" } else { "s" }
            ),
            ValidationErrorKind::MinItems { limit } => write!(
                f,
                "{} has less than {} item{}",
                self.instance,
                limit,
                if *limit == 1 { "" } else { "s" }
            ),
            ValidationErrorKind::MaxProperties { limit } => write!(
                f,
                "{} has more than {} propert{}",
                self.instance,
                limit,
                if *limit == 1 { "y" } else { "ies" }
            ),
            ValidationErrorKind::MinProperties { limit } => write!(
                f,
                "{} has less than {} propert{}",
                self.instance,
                limit,
                if *limit == 1 { "y" } else { "ies" }
            ),
            ValidationErrorKind::Not { schema } => {
                write!(f, "{} is not allowed for {}", schema, self.instance)
            }
            ValidationErrorKind::OneOfNotValid => write!(
                f,
                "'{}' is not valid under any of the given schemas",
                self.instance
            ),
            ValidationErrorKind::OneOfMultipleValid => write!(
                f,
                "'{}' is valid under more than one of the given schemas",
                self.instance
            ),
            ValidationErrorKind::Pattern { pattern } => {
                write!(f, "'{}' does not match '{}'", self.instance, pattern)
            }
            ValidationErrorKind::Required { property } => {
                write!(f, "'{}' is a required property", property)
            }
            ValidationErrorKind::MultipleOf { multiple_of } => {
                write!(f, "{} is not a multiple of {}", self.instance, multiple_of)
            }
            ValidationErrorKind::UniqueItems => {
                write!(f, "'{}' has non-unique elements", self.instance)
            }
            ValidationErrorKind::Type {
                kind: TypeKind::Single(type_),
            } => write!(f, "'{}' is not of type '{}'", self.instance, type_),
            ValidationErrorKind::Type {
                kind: TypeKind::Multiple(types),
            } => write!(
                f,
                "'{}' is not of types {}",
                self.instance,
                types
                    .iter()
                    .map(|t| format!("'{}'", t))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn single_type_error() {
        let instance = json!(42);
        let err = ValidationError::single_type_error(&instance, PrimitiveType::String);
        let repr = format!("{}", err);
        assert_eq!(repr, "'42' is not of type 'string'")
    }

    #[test]
    fn multiple_types_error() {
        let instance = json!(42);
        let err = ValidationError::multiple_type_error(
            &instance,
            vec![PrimitiveType::String, PrimitiveType::Number],
        );
        let repr = format!("{}", err);
        assert_eq!(repr, "'42' is not of types 'string', 'number'")
    }
}
