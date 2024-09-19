use core::fmt;
use std::{num::ParseIntError, str::Utf8Error};

use fluent_uri::error::{BuildError, ParseError, ResolveError};

/// Errors that can occur during reference resolution and resource handling.
#[derive(Debug)]
pub enum Error {
    /// A resource is not present in a registry and retrieving it failed.
    Unretrievable {
        uri: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    /// A JSON Pointer leads to a part of a document that does not exist.
    PointerToNowhere { pointer: String },
    /// JSON Pointer contains invalid percent-encoded data.
    InvalidPercentEncoding { pointer: String, source: Utf8Error },
    /// Failed to parse array index in JSON Pointer.
    InvalidArrayIndex {
        pointer: String,
        index: String,
        source: ParseIntError,
    },
    /// An anchor does not exist within a particular resource.
    NoSuchAnchor { anchor: String },
    /// An anchor which could never exist in a resource was dereferenced.
    InvalidAnchor { anchor: String },
    /// An error occurred while parsing or manipulating a URI.
    InvalidUri(UriError),
    /// An unknown JSON Schema specification was encountered.
    UnknownSpecification { specification: String },
}

impl Error {
    pub(crate) fn pointer_to_nowhere(pointer: impl Into<String>) -> Error {
        Error::PointerToNowhere {
            pointer: pointer.into(),
        }
    }
    pub(crate) fn invalid_percent_encoding(pointer: impl Into<String>, source: Utf8Error) -> Error {
        Error::InvalidPercentEncoding {
            pointer: pointer.into(),
            source,
        }
    }
    pub(crate) fn invalid_array_index(
        pointer: impl Into<String>,
        index: impl Into<String>,
        source: ParseIntError,
    ) -> Error {
        Error::InvalidArrayIndex {
            pointer: pointer.into(),
            index: index.into(),
            source,
        }
    }
    pub(crate) fn invalid_anchor(anchor: impl Into<String>) -> Error {
        Error::InvalidAnchor {
            anchor: anchor.into(),
        }
    }
    pub(crate) fn no_such_anchor(anchor: impl Into<String>) -> Error {
        Error::NoSuchAnchor {
            anchor: anchor.into(),
        }
    }
    pub(crate) fn unknown_specification(specification: impl Into<String>) -> Error {
        Error::UnknownSpecification {
            specification: specification.into(),
        }
    }

    pub(crate) fn unretrievable(
        uri: impl Into<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Error {
        Error::Unretrievable {
            uri: uri.into(),
            source,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Unretrievable { uri, source } => {
                f.write_fmt(format_args!("Resource '{uri}' is not present in a registry and retrieving it failed"))?;
                if let Some(err) = source {
                    f.write_fmt(format_args!(": {err}"))?;
                }
                Ok(())
            },
            Error::PointerToNowhere { pointer } => {
                f.write_fmt(format_args!("Pointer '{pointer}' does not exist"))
            }
            Error::InvalidPercentEncoding { pointer, .. } => {
                f.write_fmt(format_args!("Invalid percent encoding in pointer '{pointer}': the decoded bytes do not represent valid UTF-8"))
            }
            Error::InvalidArrayIndex { pointer, index, .. } => {
                f.write_fmt(format_args!("Failed to parse array index '{index}' in pointer '{pointer}'"))
            }
            Error::NoSuchAnchor { anchor } => {
                f.write_fmt(format_args!("Anchor '{anchor}' does not exist"))
            }
            Error::InvalidAnchor { anchor } => {
                f.write_fmt(format_args!("Anchor '{anchor}' is invalid"))
            }
            Error::InvalidUri(error) => f.write_fmt(format_args!("Invalid URI: {error}")),
            Error::UnknownSpecification { specification } => {
                f.write_fmt(format_args!("Unknown specification: {specification}"))
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Unretrievable { source, .. } => {
                if let Some(source) = source {
                    Some(&**source)
                } else {
                    None
                }
            }
            Error::InvalidUri(error) => Some(error),
            Error::InvalidPercentEncoding { source, .. } => Some(source),
            Error::InvalidArrayIndex { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum UriError {
    Parse(ParseError),
    Resolve(ResolveError),
    Build(BuildError),
}

impl fmt::Display for UriError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UriError::Parse(err) => err.fmt(f),
            UriError::Resolve(err) => err.fmt(f),
            UriError::Build(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for UriError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            UriError::Parse(err) => Some(err),
            UriError::Resolve(err) => Some(err),
            UriError::Build(err) => Some(err),
        }
    }
}

impl From<ParseError<String>> for UriError {
    fn from(error: ParseError<String>) -> Self {
        UriError::Parse(error.strip_input())
    }
}

impl From<ParseError> for UriError {
    fn from(error: ParseError) -> Self {
        UriError::Parse(error)
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Self {
        Error::InvalidUri(error.into())
    }
}

impl From<ResolveError> for Error {
    fn from(error: ResolveError) -> Self {
        Error::InvalidUri(UriError::Resolve(error))
    }
}

impl From<BuildError> for Error {
    fn from(error: BuildError) -> Self {
        Error::InvalidUri(UriError::Build(error))
    }
}
