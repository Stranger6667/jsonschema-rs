use super::error::{Error, Result};
use once_cell::sync::Lazy;
use serde_json::{Map, Value};
use std::{borrow::Cow, collections::HashMap};
use url::Url;

const DEFAULT_ROOT_URL: &str = "json-schema:///";
static DEFAULT_SCOPE: Lazy<Url> =
    Lazy::new(|| Url::parse(DEFAULT_ROOT_URL).expect("Is a valid URL"));

fn is_default_scope(scope: &Url) -> bool {
    scope == &*DEFAULT_SCOPE
}

fn id_of(schema: &Value) -> Option<&str> {
    schema.as_object().and_then(id_of_object)
}

pub(crate) fn resolve(schema: &Value) -> Result<(Resolver<'_>, HashMap<Url, Value>)> {
    let resolver = Resolver::new(schema)?;
    let external = fetch_external(schema, &resolver)?;
    Ok((resolver, external))
}

#[inline]
pub(crate) fn id_of_object(object: &Map<String, Value>) -> Option<&str> {
    object.get("$id").and_then(Value::as_str)
}

/// Get a scope of the given document.
/// If there is no `$id` key, get the default scope.
pub fn scope_of(schema: &Value) -> Result<Url> {
    if let Some(id) = id_of(schema).map(Url::parse) {
        Ok(id?)
    } else {
        Ok(DEFAULT_SCOPE.clone())
    }
}

pub(crate) fn with_folders(scope: &Url, reference: &str, folders: &[&str]) -> Result<Url> {
    let mut location = scope.clone();
    if folders.len() > 1 {
        for folder in folders.iter().skip(1) {
            location = location.join(folder)?;
        }
    }
    Ok(location.join(reference)?)
}

#[derive(Debug)]
pub struct Resolver<'schema> {
    document: &'schema Value,
    schemas: HashMap<String, &'schema Value>,
    scope: Url,
}

impl<'schema> Resolver<'schema> {
    /// Create a new resolver with automatic scope detection.
    pub fn new(document: &'schema Value) -> Result<Self> {
        Ok(Self::with_scope(document, scope_of(document)?))
    }
    /// Create a resolver with an explicit scope.
    pub fn with_scope(document: &'schema Value, scope: Url) -> Self {
        let schemas = collect_schemas(document, scope.clone());
        Self {
            document,
            schemas,
            scope,
        }
    }

    pub const fn scope(&self) -> &Url {
        &self.scope
    }

    pub fn contains(&self, key: &str) -> bool {
        self.schemas.contains_key(key)
    }

    /// Resolve a reference that is known to be local for this resolver.
    pub fn resolve(&self, reference: &str) -> Result<(Vec<&str>, &'schema Value)> {
        // First, build the full URL that is aware of the resolution context
        let url = self.build_url(reference)?;
        // Then, look for location-independent identifiers in the current schema
        if let Some(document) = self.schemas.get(url.as_str()) {
            Ok((vec![], document))
        } else {
            // And resolve the reference in the stored document
            let raw_pointer = to_pointer(&url);
            if raw_pointer == "#" {
                Ok((vec![], self.document))
            } else if let Some((folders, resolved)) = pointer(self.document, &raw_pointer) {
                Ok((folders, resolved))
            } else {
                panic!("Failed to resolve: {reference}")
            }
        }
    }
    pub(crate) fn build_url(&self, reference: &str) -> Result<Url> {
        Ok(Url::options()
            .base_url(Some(&self.scope))
            .parse(reference)?)
    }
}

/// Based on `serde_json`, but tracks folders in the traversed documents.
pub(crate) fn pointer<'a>(document: &'a Value, pointer: &str) -> Option<(Vec<&'a str>, &'a Value)> {
    if pointer.is_empty() {
        return Some((vec![], document));
    }
    let tokens = pointer
        .split('/')
        .skip(1)
        // TODO. use `maybe_replace`
        .map(|x| x.replace("~1", "/").replace("~0", "~"));
    let mut target = document;
    let mut folders = vec![];

    for token in tokens {
        let target_opt = match target {
            Value::Object(map) => {
                if let Some(id) = id_of(target) {
                    folders.push(id);
                }
                map.get(&*token)
            }
            Value::Array(list) => parse_index(&token).and_then(|x| list.get(x)),
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

fn to_pointer(url: &Url) -> Cow<str> {
    percent_encoding::percent_decode_str(url.fragment().unwrap_or(""))
        .decode_utf8()
        .expect("Input URL is always UTF-8")
}

macro_rules! push_map {
    ($stack:expr, $object:expr, $scope_idx:expr) => {
        for (key, value) in $object {
            if key == "enum" || key == "const" {
                continue;
            }
            $stack.push(($scope_idx, value));
        }
    };
}

macro_rules! push_array {
    ($stack:expr, $array:expr, $scope_idx:expr) => {
        for item in $array {
            $stack.push(($scope_idx, item));
        }
    };
}

macro_rules! push_map_unrolled {
    ($stack:expr, $store:expr, $scopes:expr, $object:expr, $scope_idx:expr) => {
        for (key, value) in $object {
            if key == "enum" || key == "const" {
                continue;
            }
            match value {
                Value::Object(object) => {
                    if let Some(id) = id_of_object(object) {
                        new_schema!($store, $scopes, $scope_idx, id, value);
                        push_map!($stack, object, $scopes.len() - 1);
                    } else {
                        push_map!($stack, object, $scope_idx);
                    }
                }
                Value::Array(array) => {
                    push_array_unrolled!($stack, $store, $scopes, array, $scope_idx);
                }
                _ => {}
            }
        }
    };
}

macro_rules! push_array_unrolled {
    ($stack:expr, $store:expr, $scopes:expr, $array:expr, $scope_idx:expr) => {
        for item in $array {
            match item {
                Value::Object(object) => {
                    if let Some(id) = id_of_object(object) {
                        new_schema!($store, $scopes, $scope_idx, id, item);
                        push_map!($stack, object, $scopes.len() - 1);
                    } else {
                        push_map!($stack, object, $scope_idx);
                    }
                }
                Value::Array(array) => {
                    push_array!($stack, array, $scope_idx);
                }
                _ => {}
            }
        }
    };
}

macro_rules! new_schema {
    ($store:expr, $scopes:expr, $scope_idx:expr, $id:expr, $value:expr) => {
        let mut scope = $scopes[$scope_idx].join($id).expect("Invalid scope id");
        // Empty fragments are discouraged and are not distinguishable absent fragments
        if let Some("") = scope.fragment() {
            scope.set_fragment(None);
        }
        $store.insert(scope.to_string(), $value);
        $scopes.push(scope);
    };
}

fn collect_schemas(schema: &Value, scope: Url) -> HashMap<String, &Value> {
    let mut store = HashMap::new();
    let mut scopes = vec![scope];
    let mut stack = Vec::with_capacity(64);
    stack.push((0_usize, schema));
    while let Some((scope_idx, value)) = stack.pop() {
        match value {
            Value::Object(object) => {
                if let Some(id) = id_of_object(object) {
                    new_schema!(store, scopes, scope_idx, id, value);
                    push_map_unrolled!(stack, store, scopes, object, scopes.len() - 1);
                } else {
                    push_map_unrolled!(stack, store, scopes, object, scope_idx);
                }
            }
            Value::Array(array) => {
                push_array_unrolled!(stack, store, scopes, array, scope_idx);
            }
            _ => {}
        }
    }
    store
}

/// Fetch all external schemas reachable from the root.
pub(crate) fn fetch_external(schema: &Value, resolver: &Resolver) -> Result<HashMap<Url, Value>> {
    let mut store = HashMap::new();
    fetch_routine(schema, &mut store, resolver)?;
    Ok(store)
}

/// Recursive routine for traversing a schema and fetching external references.
fn fetch_routine(
    schema: &Value,
    store: &mut HashMap<Url, Value>,
    resolver: &Resolver,
) -> Result<()> {
    // Current schema id - if occurs in a reference, then there is no need to resolve it
    let scope = scope_of(schema)?;
    let mut stack = vec![(vec![], schema)];
    while let Some((mut folders, value)) = stack.pop() {
        match value {
            // Only objects may have references to external schemas
            Value::Object(object) => {
                if let Some(id) = id_of_object(object) {
                    folders.push(id);
                }
                for (key, value) in object {
                    if key == "$ref" && value.as_str().map_or(false, |value| !is_local(value)) {
                        if let Some(reference) = value.as_str() {
                            if resolver.contains(reference) {
                                continue;
                            }
                        }
                        fetch_external_reference(value, &folders, store, &scope, resolver)?;
                    } else {
                        // Explore any other key
                        stack.push((folders.clone(), value))
                    }
                }
            }
            // Explore arrays further
            Value::Array(items) => {
                for item in items {
                    stack.push((folders.clone(), item))
                }
            }
            // Primitive types do not contain any references, skip
            _ => continue,
        }
    }
    Ok(())
}

/// If reference is pointing to an external resource, then fetch & store it.
fn fetch_external_reference(
    value: &Value,
    folders: &[&str],
    store: &mut HashMap<Url, Value>,
    scope: &Url,
    resolver: &Resolver,
) -> Result<()> {
    if let Some(location) = without_fragment(value) {
        // Resolve only references that are:
        //   - pointing to another resource
        //   - are not already resolved
        fetch_and_store(store, scope, location, resolver)?;
    } else if !is_default_scope(scope) {
        if let Some(reference) = value.as_str() {
            let location = with_folders(scope, reference, folders)?;
            fetch_and_store(store, scope, location, resolver)?;
        }
    }
    Ok(())
}

fn fetch_and_store(
    store: &mut HashMap<Url, Value>,
    scope: &Url,
    location: Url,
    resolver: &Resolver,
) -> Result<()> {
    if location != *scope && !store.contains_key(&location) && !resolver.contains(location.as_str())
    {
        let response = reqwest::blocking::get(location.as_str())?;
        let document = response.json::<Value>()?;
        // Make a recursive call to the callee routine
        fetch_routine(&document, store, resolver)?;
        store.insert(location, document);
    }
    Ok(())
}

/// Extract a fragment-less URL from the reference value.
fn without_fragment(value: &Value) -> Option<Url> {
    if let Some(Ok(mut location)) = value.as_str().map(Url::parse) {
        location.set_fragment(None);
        Some(location)
    } else {
        None
    }
}

pub(crate) fn build_resolvers(external: &HashMap<Url, Value>) -> HashMap<&str, Resolver> {
    let mut resolvers = HashMap::with_capacity(external.len());
    for (scope, document) in external {
        resolvers.insert(
            scope.as_str(),
            Resolver::with_scope(document, scope.clone()),
        );
    }
    resolvers
}

pub(crate) fn is_local(reference: &str) -> bool {
    reference.starts_with('#')
}

/// A JSON Schema reference.
pub(crate) enum Reference<'a> {
    /// Absolute reference.
    /// Example: `http://localhost:1234/subSchemas.json#/integer`
    Absolute(String),
    /// Relative reference.
    /// Example: `#foo`
    Relative(&'a str),
}

impl<'a> TryFrom<&'a str> for Reference<'a> {
    type Error = Error;

    fn try_from(value: &'a str) -> Result<Self> {
        match Url::parse(value) {
            Ok(mut location) => {
                location.set_fragment(None);
                Ok(Self::Absolute(location.to_string()))
            }
            Err(url::ParseError::RelativeUrlWithoutBase) => Ok(Self::Relative(value)),
            Err(error) => Err(Error::InvalidUrl(error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::load_case;
    use serde_json::{json, Value};
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

    #[test_case("ref-absolute-uri", &["http://localhost:1234/bar#foo"], &["/definitions/A"])]
    #[test_case(
        "ref-schemas-in-object",
        &["json-schema:///#foo"],
        &["/definitions/A"]
    )]
    #[test_case(
        "ref-schemas-in-array",
        &[
            "json-schema:///#foo",
            "json-schema:///#bar",
        ],
        &[
            "/definitions/A/0",
            "/definitions/A/1",
        ]
    )]
    #[test_case(
        "ref-recursive-between-schemas",
        &[
            "http://localhost:1234/tree",
            "http://localhost:1234/node",
        ],
        &[
            "",
            "/definitions/node",
        ]
    )]
    #[test_case(
        "ref-remote-base-uri-change-in-subschema",
        &[
            "http://localhost:1234/root",
            "http://localhost:1234/nested.json",
            "http://localhost:1234/nested.json#foo",
        ],
        &[
            "",
            "/$defs/A",
            "/$defs/A/$defs/B",
        ]
    )]
    #[test_case(
        "ref-remote-base-uri-change",
        &[
            "http://localhost:1234/",
            "http://localhost:1234/baseUriChange/"
        ],
        &[
            "",
            "/items"
        ]
    )]
    #[test_case(
        "ref-remote-base-uri-change-folder",
        &[
            "http://localhost:1234/scope_change_defs1.json",
            "http://localhost:1234/baseUriChangeFolder/",
        ],
        &[
            "",
            "/definitions/baz",
        ]
    )]
    fn location_identifiers(name: &str, ids: &[&str], pointers: &[&str]) {
        let schema = &load_case(name)["schema"];
        let store = collect_schemas(schema, scope_of(schema).unwrap());
        assert_eq!(store.len(), ids.len());
        for (id, pointer) in ids.iter().zip(pointers.iter()) {
            assert_eq!(store[*id], schema.pointer(pointer).unwrap());
        }
    }

    // No ID
    #[test_case(json!({}), None)]
    // String ID
    #[test_case(json!({"$id": "ID"}), Some("ID"))]
    // Non-string ID
    #[test_case(json!({"$id": 42}), None)]
    fn ids_of(schema: Value, id: Option<&str>) {
        assert_eq!(id_of(&schema), id);
    }

    const ID: &str = "http://127.0.0.1/";

    // No ID
    #[test_case(json!({}), DEFAULT_ROOT_URL)]
    #[test_case(json!({"$id": ID}), ID)]
    fn scopes_of(schema: Value, url: &str) {
        assert_eq!(scope_of(&schema).unwrap(), Url::parse(url).unwrap());
    }

    #[test_case(
        "ref-remote-base-uri-change-folder",
        "#/definitions/baz/type",
        "/definitions/baz/type"
    )]
    #[test_case("ref-schemas-in-object", "#foo", "/definitions/A")]
    #[test_case("ref-schemas-in-object", "#", "")]
    fn resolving(name: &str, reference: &str, pointer: &str) {
        // Compare resolving results by JSON Schema reference and an equivalent JSON pointer
        let schema = &load_case(name)["schema"];
        let resolver = Resolver::new(schema).unwrap();
        let (_, by_reference) = resolver.resolve(reference).unwrap();
        let by_pointer = schema.pointer(pointer).unwrap();
        assert_eq!(by_reference, by_pointer);
    }
}
