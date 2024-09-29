use fluent_uri::{encoding::EString, Uri, UriRef};

use crate::Error;
pub use fluent_uri::encoding::encoder::Path;

/// Resolves the URI reference against the given base URI and returns the target URI.
///
/// # Errors
///
/// Returns an error if base has not schema or there is a fragment.
pub fn resolve_against(base: &UriRef<&str>, uri: &str) -> Result<UriRef<String>, Error> {
    Ok(UriRef::parse(uri)?
        .resolve_against(&(Uri::try_from(*base)?))?
        .normalize()
        .into())
}

/// Parses a URI reference from a string into a [`crate::Uri`].
///
/// # Errors
///
/// Returns an error if the input string does not conform to URI-reference from RFC 3986.
pub fn from_str(uri: &str) -> Result<UriRef<String>, Error> {
    Ok(UriRef::parse(uri)?.normalize())
}

pub type EncodedString = EString<Path>;
