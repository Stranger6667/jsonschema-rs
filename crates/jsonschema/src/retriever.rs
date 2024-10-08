//! Logic for retrieving external resources.
use referencing::{Retrieve, Uri};
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
    #[allow(unused)]
    fn resolve(
        &self,
        _root_schema: &Value,
        url: &Url,
        _reference: &str,
    ) -> Result<Arc<Value>, SchemaResolverError> {
        #[cfg(target_arch = "wasm32")]
        {
            Err(anyhow::anyhow!(
                "External references are not supported in WASM"
            ))
        }
        #[cfg(not(target_arch = "wasm32"))]
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
            _ => Err(anyhow::anyhow!("Unknown scheme {}", url.scheme())),
        }
    }
}

impl Retrieve for DefaultRetriever {
    #[allow(unused)]
    fn retrieve(&self, uri: &Uri<&str>) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(target_arch = "wasm32")]
        {
            Err("External references are not supported in WASM".into())
        }
        #[cfg(not(target_arch = "wasm32"))]
        match uri.scheme().as_str() {
            "http" | "https" => {
                #[cfg(any(feature = "resolve-http", test))]
                {
                    Ok(reqwest::blocking::get(uri.as_str())?.json()?)
                }
                #[cfg(not(any(feature = "resolve-http", test)))]
                Err("`resolve-http` feature or a custom resolver is required to resolve external schemas via HTTP".into())
            }
            "file" => {
                #[cfg(any(feature = "resolve-file", test))]
                {
                    let path = uri.path().as_str();
                    let path = {
                        #[cfg(windows)]
                        {
                            // Remove the leading slash and replace forward slashes with backslashes
                            let path = path.trim_start_matches('/').replace('/', "\\");
                            std::path::PathBuf::from(path)
                        }
                        #[cfg(not(windows))]
                        {
                            std::path::PathBuf::from(path)
                        }
                    };
                    let file = std::fs::File::open(path)?;
                    Ok(serde_json::from_reader(file)?)
                }
                #[cfg(not(any(feature = "resolve-file", test)))]
                {
                    Err("`resolve-file` feature or a custom resolver is required to resolve external schemas via files".into())
                }
            }
            scheme => Err(format!("Unknown scheme {scheme}").into()),
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
    fn retrieve(&self, uri: &Uri<&str>) -> Result<Value, Box<dyn StdError + Send + Sync>> {
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

#[cfg(test)]
mod tests {
    use serde_json::json;
    #[cfg(not(target_arch = "wasm32"))]
    use std::io::Write;

    use super::DefaultRetriever;

    #[cfg(not(target_arch = "wasm32"))]
    fn path_to_uri(path: &std::path::Path) -> String {
        use percent_encoding::{percent_encode, AsciiSet, CONTROLS};

        let mut result = "file://".to_owned();
        const SEGMENT: &AsciiSet = &CONTROLS
            .add(b' ')
            .add(b'"')
            .add(b'<')
            .add(b'>')
            .add(b'`')
            .add(b'#')
            .add(b'?')
            .add(b'{')
            .add(b'}')
            .add(b'/')
            .add(b'%');

        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::ffi::OsStrExt;

            const CUSTOM_SEGMENT: &AsciiSet = &SEGMENT.add(b'\\');
            for component in path.components().skip(1) {
                result.push('/');
                result.extend(percent_encode(
                    component.as_os_str().as_bytes(),
                    CUSTOM_SEGMENT,
                ));
            }
        }
        #[cfg(target_os = "windows")]
        {
            use std::path::{Component, Prefix};
            let mut components = path.components();

            match components.next() {
                Some(Component::Prefix(ref p)) => match p.kind() {
                    Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => {
                        result.push('/');
                        result.push(letter as char);
                        result.push(':');
                    }
                    _ => panic!("Unexpected path"),
                },
                _ => panic!("Unexpected path"),
            }

            for component in components {
                if component == Component::RootDir {
                    continue;
                }

                let component = component.as_os_str().to_str().expect("Unexpected path");

                result.push('/');
                result.extend(percent_encode(component.as_bytes(), SEGMENT));
            }
        }
        result
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_retrieve_from_file() {
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let external_schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        });
        write!(temp_file, "{}", external_schema).expect("Failed to write to temp file");

        let uri = path_to_uri(temp_file.path());

        let schema = json!({
            "type": "object",
            "properties": {
                "user": { "$ref": uri }
            }
        });

        let validator = crate::validator_for(&schema).expect("Schema compilation failed");

        let valid = json!({"user": {"name": "John Doe"}});
        assert!(validator.is_valid(&valid));

        let invalid = json!({"user": {}});
        assert!(!validator.is_valid(&invalid));
    }

    #[test]
    fn test_unknown_scheme() {
        let schema = json!({
            "type": "object",
            "properties": {
                "test": { "$ref": "unknown-schema://test" }
            }
        });

        let result = crate::validator_for(&schema);

        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        #[cfg(not(target_arch = "wasm32"))]
        assert!(error.contains("Unknown scheme"));
        #[cfg(target_arch = "wasm32")]
        assert!(error.contains("External references are not supported in WASM"));
    }

    #[test]
    #[allow(deprecated)]
    fn test_deprecated_adapter_unknown_scheme() {
        let schema = json!({
            "type": "object",
            "properties": {
                "test": { "$ref": "unknown-schema://test" }
            }
        });
        let result = crate::options()
            .with_resolver(DefaultRetriever)
            .build(&schema);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        #[cfg(not(target_arch = "wasm32"))]
        assert!(error.contains("Unknown scheme"));
        #[cfg(target_arch = "wasm32")]
        assert!(error.contains("External references are not supported in WASM"));
    }

    #[test]
    #[allow(deprecated)]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_deprecated_adapter_file_scheme() {
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let external_schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        });
        write!(temp_file, "{}", external_schema).expect("Failed to write to temp file");

        let uri = path_to_uri(temp_file.path());

        let schema = json!({
            "type": "object",
            "properties": {
                "user": { "$ref": uri }
            }
        });

        let validator = crate::options()
            .with_resolver(DefaultRetriever)
            .build(&schema)
            .expect("Invalid schema");

        let valid = json!({"user": {"name": "John Doe"}});
        assert!(validator.is_valid(&valid));

        let invalid = json!({"user": {}});
        assert!(!validator.is_valid(&invalid));
    }
}
