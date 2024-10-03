use fluent_uri::{encoding::EString, Uri, UriRef};
use once_cell::sync::Lazy;

use crate::Error;
pub use fluent_uri::encoding::encoder::Path;

/// Resolves the URI reference against the given base URI and returns the target URI.
///
/// # Errors
///
/// Returns an error if base has not schema or there is a fragment.
pub fn resolve_against(base: &Uri<&str>, uri: &str) -> Result<Uri<String>, Error> {
    Ok(UriRef::parse(uri)?.resolve_against(base)?.normalize())
}

/// Parses a URI reference from a string into a [`crate::Uri`].
///
/// # Errors
///
/// Returns an error if the input string does not conform to URI-reference from RFC 3986.
pub fn from_str(uri: &str) -> Result<Uri<String>, Error> {
    let uriref = UriRef::parse(uri)?.normalize();
    if uriref.has_scheme() {
        Ok(Uri::try_from(uriref.as_str())?.into())
    } else {
        Ok(uriref.resolve_against(&DEFAULT_ROOT_URI.borrow())?)
    }
}

static DEFAULT_ROOT_URI: Lazy<Uri<String>> =
    Lazy::new(|| Uri::parse("json-schema:///".to_string()).expect("Invalid URI"));

pub type EncodedString = EString<Path>;
