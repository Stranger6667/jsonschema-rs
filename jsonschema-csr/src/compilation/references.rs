use super::error::Error;
use url::Url;

pub(crate) fn is_local(reference: &str) -> bool {
    reference.starts_with('#')
}

/// A JSON Schema reference.
pub(crate) enum Reference<'a> {
    /// Absolute reference.
    /// Example: `http://localhost:1234/subSchemas.json#/integer`
    Absolute(Url),
    /// Relative reference.
    /// Example: `#foo`
    Relative(&'a str),
}

impl<'a> TryFrom<&'a str> for Reference<'a> {
    type Error = Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match Url::parse(value) {
            Ok(mut location) => {
                location.set_fragment(None);
                Ok(Self::Absolute(location))
            }
            Err(url::ParseError::RelativeUrlWithoutBase) => Ok(Self::Relative(value)),
            Err(error) => Err(Error::InvalidUrl(error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("#foo"; "Location-independent identifier")]
    #[test_case("remote.json"; "Remote schema")]
    #[test_case("remote.json#/key"; "Remote schema with fragment")]
    fn relative(value: &str) {
        let reference = Reference::try_from(value).unwrap();
        assert!(matches!(reference, Reference::Relative(_)))
    }

    #[test_case("http://localhost/integer.json"; "Absolute reference")]
    #[test_case("http://localhost/integer.json#/integer"; "Absolute reference with fragment")]
    #[test_case("http://localhost/bar#foo"; "Location-independent identifier with an absolute URI")]
    fn absolute(value: &str) {
        let reference = Reference::try_from(value).unwrap();
        assert!(matches!(reference, Reference::Absolute(_)))
    }

    #[test]
    fn error() {
        assert!(Reference::try_from("https://127.999.999.999/").is_err());
    }
}
