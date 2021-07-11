use percent_encoding::{percent_decode, percent_decode_str};
use serde_json::Value;
use std::borrow::Cow;
use thiserror::Error;
use url::Url;

pub(crate) mod internal;

pub trait Resolver<'schema>: Send + Sync {
    type Error: std::error::Error;

    fn resolve(
        &'schema self,
        parent: &Value,
        reference: &str,
    ) -> Result<Cow<'schema, Value>, Self::Error>;
}

#[derive(Debug, Default)]
pub struct DefaultResolver {
    root: Option<Url>,
}

impl DefaultResolver {
    fn create_url(&self, schema: &Value, reference: &str) -> Result<Url, DefaultResolverError> {
        if let Some(url) = Url::parse(reference).ok() {
            return Ok(url);
        }

        // NOTE(tamasfe): I intentionally didn't use the `id_of(draft...)`
        //      schema utilities in the library, as it doesn't report wrong types,
        //      and I didn't want to mix error types.
        //
        //      So we just try to guess the draft here, this approach isn't the best either.
        if let Some(id_value) = schema.get("$id") {
            match id_value {
                Value::Null => {}
                Value::String(s) => {
                    let base_url = Url::parse(s).map_err(|err| {
                        DefaultResolverError::InvalidSchemaReference(err.to_string())
                    })?;

                    return Url::options()
                        .base_url(Some(&base_url))
                        .parse(reference)
                        .map_err(|err| {
                            DefaultResolverError::InvalidSchemaReference(err.to_string())
                        });
                }
                _ => return Err(DefaultResolverError::InvalidSchemaIdType),
            };
        };

        if let Some(id_value) = schema.get("id") {
            match id_value {
                Value::Null => {}
                Value::String(s) => {
                    let base_url = Url::parse(s).map_err(|err| {
                        DefaultResolverError::InvalidSchemaReference(err.to_string())
                    })?;

                    return Url::options()
                        .base_url(Some(&base_url))
                        .parse(reference)
                        .map_err(|err| {
                            DefaultResolverError::InvalidSchemaReference(err.to_string())
                        });
                }
                _ => return Err(DefaultResolverError::InvalidSchemaIdType),
            };
        };

        match &self.root {
            Some(root) => Url::options()
                .base_url(Some(root))
                .parse(reference)
                .map_err(|err| DefaultResolverError::InvalidSchemaReference(err.to_string())),
            None => Err(DefaultResolverError::NoRoot),
        }
    }
}

impl<'schema> Resolver<'schema> for DefaultResolver {
    type Error = DefaultResolverError;

    fn resolve(
        &'schema self,
        parent: &Value,
        reference: &str,
    ) -> Result<Cow<'schema, Value>, Self::Error> {
        let url = self.create_url(parent, reference)?;

        match url.scheme() {
            "file" => {
                let file = std::fs::File::open(
                    percent_decode_str(url.path())
                        .decode_utf8()
                        .unwrap()
                        .as_ref(),
                )?;
                let schema: Value = serde_json::from_reader(&file)?;

                Ok(Cow::Owned(schema))
            }
            "http" | "https" => {
                #[cfg(feature = "reqwest")]
                {
                    let schema: Value = reqwest::blocking::get(url)?.json()?;

                    Ok(Cow::Owned(schema))
                }
                #[cfg(not(feature = "reqwest"))]
                {
                    Err(DefaultResolverError::UnsupportedUrlScheme(s.to_string()))
                }
            }
            other => Err(DefaultResolverError::UnsupportedUrlScheme(
                other.to_string(),
            )),
        }
    }
}

#[derive(Debug, Error)]
pub enum DefaultResolverError {
    #[error("an i/o error happened while resolving the schema: {0}")]
    Io(std::io::Error),

    #[error("no root path was found or provided for relative resolution")]
    NoRoot,

    #[cfg(feature = "reqwest")]
    #[error("HTTP request failed while resolving the schema: {0}")]
    Http(reqwest::Error),

    #[error("the URL scheme `{0}` is not supported")]
    UnsupportedUrlScheme(String),

    #[error("invalid Schema identifier `{0}`")]
    InvalidSchemaId(String),

    #[error("expected schema identifier to be a string")]
    InvalidSchemaIdType,

    #[error("invalid reference '{0}'")]
    InvalidSchemaReference(String),

    #[error("invalid JSON: {0}")]
    InvalidJson(serde_json::Error),
}

impl From<std::io::Error> for DefaultResolverError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for DefaultResolverError {
    fn from(err: serde_json::Error) -> Self {
        Self::InvalidJson(err)
    }
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for DefaultResolverError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err)
    }
}
