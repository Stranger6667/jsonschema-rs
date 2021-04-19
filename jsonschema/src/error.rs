//! Error types

use crate::primitive_type::{PrimitiveType, PrimitiveTypesBitMap};
use serde_json::{Map, Number, Value};
use std::{
    borrow::Cow,
    error, fmt,
    fmt::{Error, Formatter},
    io,
    iter::{empty, once},
    str::Utf8Error,
    string::FromUtf8Error,
};

/// The error type that happens when the input schema is not valid.
///
/// It includes cases when during validation a reference is resolved into an invalid schema,
/// which we can't know upfront because schemas can be in remote locations.
#[derive(Debug, PartialEq)]
pub enum CompilationError {
    /// Invalid schema structure
    SchemaError,
}

impl error::Error for CompilationError {}

impl fmt::Display for CompilationError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Schema compilation error")
    }
}

impl From<regex::Error> for CompilationError {
    #[inline]
    fn from(_: regex::Error) -> Self {
        CompilationError::SchemaError
    }
}
impl From<url::ParseError> for CompilationError {
    #[inline]
    fn from(_: url::ParseError) -> Self {
        CompilationError::SchemaError
    }
}

/// An error that can occur during validation.
#[derive(Debug)]
pub struct ValidationError<'a> {
    /// Value of the property that failed validation
    pub instance: Cow<'a, Value>,
    /// Type of validation error
    pub kind: ValidationErrorKind,
}

/// An iterator over instances of `ValidationError` that represent validation error for the
/// input instance.
///
/// # Examples
///
/// ```rust
/// use jsonschema::JSONSchema;
/// use serde_json::json;
///
/// let schema = json!({"maxLength": 5});
/// let instance = json!("foo");
/// if let Ok(compiled) = JSONSchema::compile(&schema) {
///     let result = compiled.validate(&instance);
///     if let Err(errors) = result {
///         for error in errors {
///             println!("Validation error: {}", error)
///         }
///     }
/// }
/// ```
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
#[allow(missing_docs)]
pub enum ValidationErrorKind {
    /// The input array contain more items than expected.
    AdditionalItems { limit: usize },
    /// The input value is not valid under any of the given schemas.
    AnyOf,
    /// The input value doesn't match expected constant.
    Constant { expected_value: Value },
    /// The input array doesn't contain items conforming to the specified schema.
    Contains,
    /// Ths input value does not respect the defined contentEncoding
    ContentEncoding { content_encoding: String },
    /// Ths input value does not respect the defined contentMediaType
    ContentMediaType { content_media_type: String },
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
    Format { format: &'static str },
    /// May happen in `contentEncoding` validation if `base64` encoded data is invalid.
    FromUtf8 { error: FromUtf8Error },
    /// Invalid UTF-8 string during percent encoding when resolving happens
    Utf8 { error: Utf8Error },
    /// May happen during ref resolution when remote document is not a valid JSON.
    JSONParse { error: serde_json::Error },
    /// `ref` value is not valid.
    InvalidReference { reference: String },
    /// Invalid URL, e.g. invalid port number or IP address
    InvalidURL { error: url::ParseError },
    /// Too many items in an array.
    MaxItems { limit: u64 },
    /// Value is too large.
    Maximum { limit: f64 },
    /// String is too long.
    MaxLength { limit: u64 },
    /// Too many properties in an object.
    MaxProperties { limit: u64 },
    /// Too few items in an array.
    MinItems { limit: u64 },
    /// Value is too small.
    Minimum { limit: f64 },
    /// String is too short.
    MinLength { limit: u64 },
    /// Not enough properties in an object.
    MinProperties { limit: u64 },
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
    /// Any error that happens during network request via `reqwest` crate
    #[cfg(any(feature = "reqwest", test))]
    Reqwest { error: reqwest::Error },
    /// Resolved schema failed to compile.
    Schema,
    /// When the input value doesn't match one or multiple required types.
    Type { kind: TypeKind },
    /// When the input array has non-unique elements.
    UniqueItems,
    /// Reference contains unknown scheme.
    UnknownReferenceScheme { scheme: String },

    /// Unexpected error. This usually represent a bug into the validation
    Unexpected { validator_representation: String },
}

#[derive(Debug)]
#[allow(missing_docs)]
pub enum TypeKind {
    Single(PrimitiveType),
    Multiple(PrimitiveTypesBitMap),
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
    pub(crate) fn constant_array(
        instance: &'a Value,
        expected_value: &[Value],
    ) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Constant {
                expected_value: Value::Array(expected_value.to_vec()),
            },
        }
    }
    pub(crate) fn constant_boolean(
        instance: &'a Value,
        expected_value: bool,
    ) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Constant {
                expected_value: Value::Bool(expected_value),
            },
        }
    }
    pub(crate) fn constant_null(instance: &'a Value) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Constant {
                expected_value: Value::Null,
            },
        }
    }
    pub(crate) fn constant_number(
        instance: &'a Value,
        expected_value: &Number,
    ) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Constant {
                expected_value: Value::Number(expected_value.clone()),
            },
        }
    }
    pub(crate) fn constant_object(
        instance: &'a Value,
        expected_value: &Map<String, Value>,
    ) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Constant {
                expected_value: Value::Object(expected_value.clone()),
            },
        }
    }
    pub(crate) fn constant_string(
        instance: &'a Value,
        expected_value: &str,
    ) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Constant {
                expected_value: Value::String(expected_value.to_string()),
            },
        }
    }
    pub(crate) fn contains(instance: &'a Value) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Contains,
        }
    }
    pub(crate) fn content_encoding(instance: &'a Value, encoding: &str) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::ContentEncoding {
                content_encoding: encoding.to_string(),
            },
        }
    }
    pub(crate) fn content_media_type(instance: &'a Value, media_type: &str) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::ContentMediaType {
                content_media_type: media_type.to_string(),
            },
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
    pub(crate) fn format(instance: &'a Value, format: &'static str) -> ValidationError<'a> {
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
    pub(crate) fn invalid_url(error: url::ParseError) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Owned(Value::Null),
            kind: ValidationErrorKind::InvalidURL { error },
        }
    }
    pub(crate) fn max_items(instance: &'a Value, limit: u64) -> ValidationError<'a> {
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
    pub(crate) fn max_length(instance: &'a Value, limit: u64) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::MaxLength { limit },
        }
    }
    pub(crate) fn max_properties(instance: &'a Value, limit: u64) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::MaxProperties { limit },
        }
    }
    pub(crate) fn min_items(instance: &'a Value, limit: u64) -> ValidationError<'a> {
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
    pub(crate) fn min_length(instance: &'a Value, limit: u64) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::MinLength { limit },
        }
    }
    pub(crate) fn min_properties(instance: &'a Value, limit: u64) -> ValidationError<'a> {
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
    #[cfg(any(feature = "reqwest", test))]
    pub(crate) fn reqwest(error: reqwest::Error) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Owned(Value::Null),
            kind: ValidationErrorKind::Reqwest { error },
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
        types: PrimitiveTypesBitMap,
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
    /// Unexpected `ValidationError`
    ///
    /// This validation error is the only `ValidationError` that can be created by external crates.
    pub fn unexpected(instance: &'a Value, validator_representation: &str) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Borrowed(instance),
            kind: ValidationErrorKind::Unexpected {
                validator_representation: validator_representation.to_string(),
            },
        }
    }
    pub(crate) fn utf8(error: Utf8Error) -> ValidationError<'a> {
        ValidationError {
            instance: Cow::Owned(Value::Null),
            kind: ValidationErrorKind::Utf8 { error },
        }
    }
}

impl From<CompilationError> for ValidationError<'_> {
    #[inline]
    fn from(_: CompilationError) -> Self {
        ValidationError::schema()
    }
}
impl error::Error for ValidationError<'_> {}
impl From<serde_json::Error> for ValidationError<'_> {
    #[inline]
    fn from(err: serde_json::Error) -> Self {
        ValidationError::json_parse(err)
    }
}
impl From<io::Error> for ValidationError<'_> {
    #[inline]
    fn from(err: io::Error) -> Self {
        ValidationError::file_not_found(err)
    }
}
impl From<FromUtf8Error> for ValidationError<'_> {
    #[inline]
    fn from(err: FromUtf8Error) -> Self {
        ValidationError::from_utf8(err)
    }
}
impl From<Utf8Error> for ValidationError<'_> {
    #[inline]
    fn from(err: Utf8Error) -> Self {
        ValidationError::utf8(err)
    }
}
impl From<url::ParseError> for ValidationError<'_> {
    #[inline]
    fn from(err: url::ParseError) -> Self {
        ValidationError::invalid_url(err)
    }
}
#[cfg(any(feature = "reqwest", test))]
impl From<reqwest::Error> for ValidationError<'_> {
    #[inline]
    fn from(err: reqwest::Error) -> Self {
        ValidationError::reqwest(err)
    }
}

/// Textual representation of various validation errors.
impl fmt::Display for ValidationError<'_> {
    #[allow(clippy::too_many_lines)] // The function is long but it does formatting only
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ValidationErrorKind::Schema => write!(f, "Schema error"),
            ValidationErrorKind::JSONParse { error } => write!(f, "{}", error),
            #[cfg(any(feature = "reqwest", test))]
            ValidationErrorKind::Reqwest { error } => write!(f, "{}", error),
            ValidationErrorKind::FileNotFound { error } => write!(f, "{}", error),
            ValidationErrorKind::InvalidURL { error } => write!(f, "{}", error),
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
                    .expect("Always valid")
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
            ValidationErrorKind::AnyOf | ValidationErrorKind::OneOfNotValid => write!(
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
            ValidationErrorKind::ContentEncoding { content_encoding } => {
                write!(f, "'{}' is not compliant with encoding={}", self.instance, content_encoding)
            }
            ValidationErrorKind::ContentMediaType { content_media_type } => {
                write!(f, "'{}' is not compliant with media_type={}", self.instance, content_media_type)
            }
            ValidationErrorKind::FromUtf8 { error } => write!(f, "{}", error),
            ValidationErrorKind::Utf8 { error } => write!(f, "{}", error),
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
                    .into_iter()
                    .map(|t| format!("'{}'", t))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ValidationErrorKind::Unexpected { validator_representation } => write!(
                f,
                "Unexpected validation error. Usually this reflect a bug in the keywords implementation. Please make sure to report the problem to {}. Instance: {}, Validator: {}",
                env!("CARGO_PKG_REPOSITORY"),
                self.instance,
                validator_representation,
            )
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
        assert_eq!(err.to_string(), "'42' is not of type 'string'")
    }

    #[test]
    fn multiple_types_error() {
        let instance = json!(42);
        let err = ValidationError::multiple_type_error(
            &instance,
            vec![PrimitiveType::String, PrimitiveType::Number].into(),
        );
        assert_eq!(err.to_string(), "'42' is not of types 'number', 'string'")
    }
}
