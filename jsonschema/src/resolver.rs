//! Reference resolver. Implements logic, required by `$ref` keyword.
//! Is able to load documents from remote locations via HTTP(S).
use crate::{
    compilation::DEFAULT_ROOT_URL,
    error::ValidationError,
    schemas::{id_of, Draft},
};
use ahash::AHashMap;
use parking_lot::RwLock;
use serde_json::Value;
use std::{borrow::Cow, sync::Arc};
use url::Url;

/// An opaque error type that is returned
/// by resolvers on resolution failures.
pub type SchemaResolverError = anyhow::Error;

/// A resolver that resolves external schema references.
/// Internal references such as `#/definitions` and JSON pointers are handled internally.
///
/// All operations are blocking and it is not possible to return futures.
/// As a workaround, errors can be returned that will contain the schema URLs to resolve
/// and can be resolved outside the validation process if needed.
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
    fn resolve(
        &self,
        root_schema: &Value,
        url: &Url,
        original_reference: &str,
    ) -> Result<Arc<Value>, SchemaResolverError>;
}

pub(crate) struct DefaultResolver;

impl SchemaResolver for DefaultResolver {
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

pub(crate) struct Resolver {
    external_resolver: Arc<dyn SchemaResolver>,
    root_schema: Arc<Value>,
    // canonical_id: sub-schema mapping to resolve documents by their ID
    // canonical_id is composed with the root document id
    // (if not specified, then `DEFAULT_ROOT_URL` is used for this purpose)
    schemas: AHashMap<String, Arc<Value>>,
    store: RwLock<AHashMap<String, Arc<Value>>>,
}

impl std::fmt::Debug for Resolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resolver")
            .field("root_schema", &self.root_schema)
            .field("schemas", &self.schemas)
            .field("store", &self.store)
            .finish()
    }
}

impl Resolver {
    pub(crate) fn new<'a>(
        external_resolver: Arc<dyn SchemaResolver>,
        draft: Draft,
        scope: &Url,
        schema: Arc<Value>,
        store: AHashMap<String, Arc<Value>>,
    ) -> Result<Resolver, ValidationError<'a>> {
        let mut schemas: AHashMap<String, Arc<Value>> = AHashMap::new();
        // traverse the schema and store all named ones under their canonical ids
        find_schemas(draft, &schema, scope, &mut |id, schema| {
            schemas.insert(id, Arc::new(schema.clone()));
            None
        })?;
        Ok(Resolver {
            external_resolver,
            root_schema: schema,
            schemas,
            store: RwLock::new(store),
        })
    }

    /// Load a document for the given `url`.
    /// It may be:
    ///   - the root document (`DEFAULT_ROOT_URL`) case;
    ///   - named subschema that is stored in `self.schemas`;
    ///   - document from a remote location;
    fn resolve_url(&self, url: &Url, orig_ref: &str) -> Result<Arc<Value>, ValidationError<'_>> {
        match url.as_str() {
            DEFAULT_ROOT_URL => Ok(self.root_schema.clone()),
            url_str => match self.schemas.get(url_str) {
                Some(value) => Ok(value.clone()),
                None => {
                    if let Some(cached) = self.store.read().get(url_str) {
                        return Ok(cached.clone());
                    }
                    let resolved = self
                        .external_resolver
                        .resolve(&self.root_schema, url, orig_ref)
                        .map_err(|error| ValidationError::resolver(url.clone(), error))?;
                    self.store
                        .write()
                        .insert(url.clone().into(), resolved.clone());
                    Ok(resolved)
                }
            },
        }
    }

    /// Resolve a URL possibly containing a fragment to a `serde_json::Value`.
    ///
    /// Note that this copies the fragment from the underlying schema, so if
    /// you are memory constrained you may want to cache the result of this
    /// call.
    pub(crate) fn resolve_fragment(
        &self,
        draft: Draft,
        url: &Url,
        orig_ref: &str,
    ) -> Result<(Url, Arc<Value>), ValidationError> {
        let mut resource = url.clone();
        resource.set_fragment(None);
        let fragment =
            percent_encoding::percent_decode_str(url.fragment().unwrap_or("")).decode_utf8()?;

        // Location-independent identifiers are searched before trying to resolve by
        // fragment-less url
        if let Some(document) = self.schemas.get(url.as_str()) {
            return Ok((resource, Arc::clone(document)));
        }

        // Each resolved document may be in a changed subfolder
        // They are tracked when JSON pointer is resolved and added to the resource
        let document = self.resolve_url(&resource, orig_ref)?;
        if fragment.is_empty() {
            return Ok((resource, Arc::clone(&document)));
        }
        match pointer(draft, &document, fragment.as_ref()) {
            Some((folders, resolved)) => {
                let joined_folders = join_folders(resource, &folders)?;
                Ok((joined_folders, Arc::new(resolved.clone())))
            }
            None => Err(ValidationError::invalid_reference(url.as_str().to_string())),
        }
    }
}

fn join_folders(mut resource: Url, folders: &[&str]) -> Result<Url, url::ParseError> {
    if folders.len() > 1 {
        for i in folders.iter().skip(1) {
            resource = resource.join(i)?;
        }
    }
    Ok(resource)
}

/// Find all sub-schemas in the document and execute callback on each of them.
#[inline]
pub(crate) fn find_schemas<'a, F>(
    draft: Draft,
    schema: &'a Value,
    base_url: &Url,
    callback: &mut F,
) -> Result<Option<&'a Value>, url::ParseError>
where
    F: FnMut(String, &'a Value) -> Option<&'a Value>,
{
    match schema {
        Value::Object(item) => {
            if let Some(url) = id_of(draft, schema) {
                let mut new_url = base_url.join(url)?;
                // Empty fragments are discouraged and are not distinguishable absent fragments
                if let Some("") = new_url.fragment() {
                    new_url.set_fragment(None);
                }
                if let Some(x) = callback(new_url.to_string(), schema) {
                    return Ok(Some(x));
                }
                for (key, subschema) in item {
                    if key == "enum" || key == "const" {
                        continue;
                    }
                    let result = find_schemas(draft, subschema, &new_url, callback)?;
                    if result.is_some() {
                        return Ok(result);
                    }
                }
            } else {
                for (key, subschema) in item {
                    if key == "enum" || key == "const" {
                        continue;
                    }
                    let result = find_schemas(draft, subschema, base_url, callback)?;
                    if result.is_some() {
                        return Ok(result);
                    }
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                let result = find_schemas(draft, item, base_url, callback)?;
                if result.is_some() {
                    return Ok(result);
                }
            }
        }
        _ => {}
    }
    Ok(None)
}

/// Searching twice is better than unconditionally allocating a String twice
trait MaybeReplaceExt<'a> {
    fn maybe_replace(self, needle: &str, replacement: &str) -> Cow<'a, str>;
}

impl<'a> MaybeReplaceExt<'a> for &'a str {
    fn maybe_replace(self, needle: &str, replacement: &str) -> Cow<'a, str> {
        if memchr::memmem::find(self.as_bytes(), needle.as_bytes()).is_some() {
            self.replace(needle, replacement).into()
        } else {
            self.into()
        }
    }
}

impl<'a> MaybeReplaceExt<'a> for Cow<'a, str> {
    fn maybe_replace(self, needle: &str, replacement: &str) -> Cow<'a, str> {
        if memchr::memmem::find(self.as_bytes(), needle.as_bytes()).is_some() {
            self.replace(needle, replacement).into()
        } else {
            self
        }
    }
}

/// Based on `serde_json`, but tracks folders in the traversed documents.
pub(crate) fn pointer<'a>(
    draft: Draft,
    document: &'a Value,
    pointer: &str,
) -> Option<(Vec<&'a str>, &'a Value)> {
    if !pointer.starts_with('/') {
        return None;
    }
    let tokens = pointer
        .split('/')
        .skip(1)
        .map(|x| x.maybe_replace("~1", "/").maybe_replace("~0", "~"));
    let mut target = document;
    let mut folders = vec![];

    for token in tokens {
        let target_opt = match *target {
            Value::Object(ref map) => {
                if let Some(id) = id_of(draft, target) {
                    folders.push(id);
                }
                map.get(&*token)
            }
            Value::Array(ref list) => parse_index(&token).and_then(|x| list.get(x)),
            _ => return None,
        };
        if let Some(t) = target_opt {
            target = t;
        } else {
            return None;
        }
    }
    Some((folders, target))
}

fn parse_index(s: &str) -> Option<usize> {
    if s.starts_with('+') || (s.starts_with('0') && s.len() != 1) {
        None
    } else {
        s.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;
    use url::Url;

    fn make_resolver(schema: &Value) -> Resolver {
        Resolver::new(
            Arc::new(DefaultResolver),
            Draft::Draft7,
            &Url::parse("json-schema:///").unwrap(),
            Arc::new(schema.clone()),
            AHashMap::new(),
        )
        .unwrap()
    }

    #[test]
    fn only_keyword() {
        // When only one keyword is specified
        let schema = json!({"type": "string"});
        let resolver = make_resolver(&schema);
        // Then in the resolver schema there should be no schemas
        assert_eq!(resolver.schemas.len(), 0);
    }

    #[test]
    fn sub_schema_in_object() {
        // When only one sub-schema is specified inside an object
        let schema = json!({
            "allOf": [{"$ref": "#foo"}],
            "definitions": {
                "A": {"$id": "#foo", "type": "integer"}
            }
        });
        let resolver = make_resolver(&schema);
        // Then in the resolver schema there should be only this schema
        assert_eq!(resolver.schemas.len(), 1);
        assert_eq!(
            resolver
                .schemas
                .get("json-schema:///#foo")
                .map(AsRef::as_ref),
            schema.pointer("/definitions/A")
        );
    }

    #[test]
    fn sub_schemas_in_array() {
        // When sub-schemas are specified inside an array
        let schema = json!({
            "definitions": {
                "A": [
                    {"$id": "#foo", "type": "integer"},
                    {"$id": "#bar", "type": "string"},
                ]
            }
        });
        let resolver = make_resolver(&schema);
        // Then in the resolver schema there should be only these schemas
        assert_eq!(resolver.schemas.len(), 2);
        assert_eq!(
            resolver
                .schemas
                .get("json-schema:///#foo")
                .map(AsRef::as_ref),
            schema.pointer("/definitions/A/0")
        );
        assert_eq!(
            resolver
                .schemas
                .get("json-schema:///#bar")
                .map(AsRef::as_ref),
            schema.pointer("/definitions/A/1")
        );
    }

    #[test]
    fn root_schema_id() {
        // When the root schema has an ID
        let schema = json!({
            "$id": "http://localhost:1234/tree",
            "definitions": {
                "node": {
                    "$id": "http://localhost:1234/node",
                    "description": "node",
                    "properties": {
                        "subtree": {"$ref": "tree"},
                        "value": {"type": "number"}
                    },
                    "required": ["value"],
                    "type": "object"
                }
            },
            "description": "tree of nodes",
            "properties": {
                "meta": {"type": "string"},
                "nodes": {
                    "items": {"$ref": "node"},
                    "type": "array"
                }
            },
            "required": ["meta", "nodes"],
            "type": "object"
        });
        let resolver = make_resolver(&schema);
        // Then in the resolver schema there should be root & sub-schema
        assert_eq!(resolver.schemas.len(), 2);
        assert_eq!(
            resolver
                .schemas
                .get("http://localhost:1234/tree")
                .map(AsRef::as_ref),
            schema.pointer("")
        );
        assert_eq!(
            resolver
                .schemas
                .get("http://localhost:1234/node")
                .map(AsRef::as_ref),
            schema.pointer("/definitions/node")
        );
    }

    #[test]
    fn location_independent_with_absolute_uri() {
        let schema = json!({
            "allOf": [{"$ref": "http://localhost:1234/bar#foo"}],
            "definitions": {
                "A": {"$id": "http://localhost:1234/bar#foo", "type": "integer"}
            }
        });
        let resolver = make_resolver(&schema);
        assert_eq!(resolver.schemas.len(), 1);
        assert_eq!(
            resolver
                .schemas
                .get("http://localhost:1234/bar#foo")
                .map(AsRef::as_ref),
            schema.pointer("/definitions/A")
        );
    }

    #[test]
    fn location_independent_with_absolute_uri_base_change() {
        let schema = json!({
            "$id": "http://localhost:1234/root",
            "allOf":[{"$ref": "http://localhost:1234/nested.json#foo"}],
            "definitions": {
                "A": {
                    "$id": "nested.json",
                    "definitions": {
                        "B": {
                            "$id": "#foo",
                            "type": "integer"
                        }
                    }
                }
            }
        });
        let resolver = make_resolver(&schema);
        assert_eq!(resolver.schemas.len(), 3);
        assert_eq!(
            resolver
                .schemas
                .get("http://localhost:1234/root")
                .map(AsRef::as_ref),
            schema.pointer("")
        );
        assert_eq!(
            resolver
                .schemas
                .get("http://localhost:1234/nested.json")
                .map(AsRef::as_ref),
            schema.pointer("/definitions/A")
        );
        assert_eq!(
            resolver
                .schemas
                .get("http://localhost:1234/nested.json#foo")
                .map(AsRef::as_ref),
            schema.pointer("/definitions/A/definitions/B")
        );
    }

    #[test]
    fn base_uri_change() {
        let schema = json!({
            "$id": "http://localhost:1234/",
            "items": {
                "$id":"folder/",
                "items": {"$ref": "folderInteger.json"}
            }
        });
        let resolver = make_resolver(&schema);
        assert_eq!(resolver.schemas.len(), 2);
        assert_eq!(
            resolver
                .schemas
                .get("http://localhost:1234/")
                .map(AsRef::as_ref),
            schema.pointer("")
        );
        assert_eq!(
            resolver
                .schemas
                .get("http://localhost:1234/folder/")
                .map(AsRef::as_ref),
            schema.pointer("/items")
        );
    }

    #[test]
    fn base_uri_change_folder() {
        let schema = json!({
            "$id": "http://localhost:1234/scope_change_defs1.json",
            "definitions": {
                "baz": {
                    "$id": "folder/",
                    "items": {"$ref": "folderInteger.json"},
                    "type":"array"
                }
            },
            "properties": {
                "list": {"$ref": "#/definitions/baz"}
            },
            "type": "object"
        });
        let resolver = make_resolver(&schema);
        assert_eq!(resolver.schemas.len(), 2);
        assert_eq!(
            resolver
                .schemas
                .get("http://localhost:1234/scope_change_defs1.json")
                .map(AsRef::as_ref),
            schema.pointer("")
        );
        assert_eq!(
            resolver
                .schemas
                .get("http://localhost:1234/folder/")
                .map(AsRef::as_ref),
            schema.pointer("/definitions/baz")
        );
    }

    #[test]
    fn resolve_ref() {
        let schema = json!({
            "$ref": "#/definitions/c",
            "definitions": {
                "a": {"type": "integer"},
                "b": {"$ref": "#/definitions/a"},
                "c": {"$ref": "#/definitions/b"}
            }
        });
        let resolver = make_resolver(&schema);
        let url = Url::parse("json-schema:///#/definitions/a").unwrap();
        let (resource, resolved) = resolver
            .resolve_fragment(Draft::Draft7, &url, "#/definitions/a")
            .unwrap();
        assert_eq!(resource, Url::parse("json-schema:///").unwrap());
        assert_eq!(resolved.as_ref(), schema.pointer("/definitions/a").unwrap());
    }
}
