use fluent_uri::{encoding::EString, Uri, UriRef};

use crate::Error;
pub use fluent_uri::encoding::encoder::Path;

const fn table() -> [bool; 259] {
    let mut bytes = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+-.".as_slice();
    let mut table = [false; 259];
    while let [cur, rem @ ..] = bytes {
        table[*cur as usize] = true;
        bytes = rem;
    }
    table
}

const SCHEME: &[bool; 259] = &table();
/// Resolves the URI reference against the given base URI and returns the target URI.
///
/// # Errors
///
/// Returns an error if base has not schema or there is a fragment.
pub fn resolve_against(base: &UriRef<&str>, uri: &str) -> Result<UriRef<String>, Error> {
    // Emulate `ParseError` to avoid panic on invalid input.
    //
    // See https://github.com/yescallop/fluent-uri-rs/issues/28
    if base.as_str().is_empty() || !base.as_str().bytes().any(|x| !SCHEME[x as usize]) {
        Uri::try_from("/".to_string()).map_err(|err| err.strip_input())?;
    }
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
