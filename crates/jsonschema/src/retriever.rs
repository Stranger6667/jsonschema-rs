//! Logic for retrieving external resources.
use referencing::{Retrieve, UriRef};
use serde_json::{json, Value};
use std::{error::Error as StdError, sync::Arc};
use url::Url;

/// An opaque error type that is returned by resolvers on resolution failures.
#[deprecated(
    since = "0.21.0",
    note = "The new `jsonschema::Retrieve` trait uses a different error type which obsolete this type alias. This type alias will be removed in a future release."
)]
pub type SchemaResolverError = anyhow::Error;

/// A resolver that resolves external schema references.
/// Internal references such as `#/definitions` and JSON pointers are handled internally.
///
/// All operations are blocking and it is not possible to return futures.
/// As a workaround, errors can be returned that will contain the schema URLs to resolve
/// and can be resolved outside the validation process if needed.
///
/// # Deprecated
///
/// Use [`jsonschema::Retrieve`] instead.
///
/// # Example
///
/// ```no_run
/// # use serde_json::{json, Value};
/// # use anyhow::anyhow;
/// # use jsonschema::{SchemaResolver, SchemaResolverError};
/// # use std::sync::Arc;
/// # use url::Url;
///
/// struct MyCustomResolver;
///
/// impl SchemaResolver for MyCustomResolver {
///     fn resolve(&self, root_schema: &Value, url: &Url, _original_reference: &str) -> Result<Arc<Value>, SchemaResolverError> {
///         match url.scheme() {
///             "json-schema" => {
///                 Err(anyhow!("cannot resolve schema without root schema ID"))
///             },
///             "http" | "https" => {
///                 Ok(Arc::new(json!({ "description": "an external schema" })))
///             }
///             _ => Err(anyhow!("scheme is not supported"))
///         }
///     }
/// }
/// ```
#[deprecated(
    since = "0.21.0",
    note = "Use `jsonschema::Retrieve` instead. This trait will be removed in a future release."
)]
pub trait SchemaResolver: Send + Sync {
    /// Resolve an external schema via an URL.
    ///
    /// Relative URLs are resolved based on the root schema's ID,
    /// if there is no root schema ID available, the scheme `json-schema` is used
    /// and any relative paths are turned into absolutes.
    ///
    /// Additionally the original reference string is also passed,
    /// in most cases it should not be needed, but it preserves some information,
    /// such as relative paths that are lost when the URL is built.
    #[allow(deprecated)]
    fn resolve(
        &self,
        root_schema: &Value,
        url: &Url,
        original_reference: &str,
    ) -> Result<Arc<Value>, SchemaResolverError>;
}

pub(crate) struct DefaultRetriever;

#[allow(deprecated)]
impl SchemaResolver for DefaultRetriever {
    fn resolve(
        &self,
        _root_schema: &Value,
        url: &Url,
        _reference: &str,
    ) -> Result<Arc<Value>, SchemaResolverError> {
        match url.scheme() {
            "http" | "https" => {
                #[cfg(all(feature = "reqwest", not(feature = "resolve-http")))]
                {
                    compile_error!("the `reqwest` feature does not enable HTTP schema resolving anymore, use the `resolve-http` feature instead");
                }
                #[cfg(any(feature = "resolve-http", test))]
                {
                    let response = reqwest::blocking::get(url.as_str())?;
                    let document: Value = response.json()?;
                    Ok(Arc::new(document))
                }
                #[cfg(not(any(feature = "resolve-http", test)))]
                Err(anyhow::anyhow!("`resolve-http` feature or a custom resolver is required to resolve external schemas via HTTP"))
            }
            "file" => {
                #[cfg(any(feature = "resolve-file", test))]
                {
                    if let Ok(path) = url.to_file_path() {
                        let f = std::fs::File::open(path)?;
                        let document: Value = serde_json::from_reader(f)?;
                        Ok(Arc::new(document))
                    } else {
                        Err(anyhow::anyhow!("invalid file path"))
                    }
                }
                #[cfg(not(any(feature = "resolve-file", test)))]
                {
                    Err(anyhow::anyhow!("`resolve-file` feature or a custom resolver is required to resolve external schemas via files"))
                }
            }
            "json-schema" => Err(anyhow::anyhow!(
                "cannot resolve relative external schema without root schema ID"
            )),
            _ => Err(anyhow::anyhow!("unknown scheme {}", url.scheme())),
        }
    }
}

impl Retrieve for DefaultRetriever {
    fn retrieve(
        &self,
        uri: &UriRef<&str>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        match uri.scheme().map(|scheme| scheme.as_str()) {
            Some("http" | "https") => {
                #[cfg(any(feature = "resolve-http", test))]
                {
                    Ok(reqwest::blocking::get(uri.as_str())?.json()?)
                }
                #[cfg(not(any(feature = "resolve-http", test)))]
                Err("`resolve-http` feature or a custom resolver is required to resolve external schemas via HTTP".into())
            }
            Some("file") => {
                #[cfg(any(feature = "resolve-file", test))]
                {
                    let file = std::fs::File::open(uri.path().as_str())?;
                    Ok(serde_json::from_reader(file)?)
                }
                #[cfg(not(any(feature = "resolve-file", test)))]
                {
                    Err("`resolve-file` feature or a custom resolver is required to resolve external schemas via files".into())
                }
            }
            Some(scheme) => Err(format!("Unknown scheme {scheme}").into()),
            None => Err("Can not resolve resource without a scheme".into()),
        }
    }
}

/// An adapter for the current implementation of [`SchemaResolver`] to work with [`referencing::Retriever`].
pub(crate) struct RetrieverAdapter {
    #[allow(deprecated)]
    resolver: Arc<dyn SchemaResolver>,
}

#[allow(deprecated)]
impl RetrieverAdapter {
    pub(crate) fn new(resolver: Arc<dyn SchemaResolver>) -> RetrieverAdapter {
        RetrieverAdapter { resolver }
    }
}

impl Retrieve for RetrieverAdapter {
    #[allow(deprecated)]
    fn retrieve(&self, uri: &UriRef<&str>) -> Result<Value, Box<dyn StdError + Send + Sync>> {
        let url = Url::parse(uri.as_str())?;
        // NOTE: There is no easy way to pass the original reference here without significant
        // changes to `referencing`. This argument does not seem to be used much in practice,
        // therefore using an empty string to fit the deprecated interface.
        match self.resolver.resolve(&json!({}), &url, "") {
            Ok(value) => Ok((*value).clone()),
            Err(err) => Err(err.into()),
        }
    }
}
