use once_cell::sync::Lazy;
use serde_json::{Map, Value};
use std::{borrow::Cow, collections::HashMap};
use url::{ParseError, Url};

pub(crate) const DEFAULT_ROOT_URL: &str = "json-schema:///";
pub(crate) static DEFAULT_SCOPE: Lazy<Url> =
    Lazy::new(|| url::Url::parse(DEFAULT_ROOT_URL).expect("Is a valid URL"));

pub(crate) fn id_of(schema: &Value) -> Option<&str> {
    schema.as_object().and_then(id_of_object)
}

#[inline]
pub(crate) fn id_of_object(object: &Map<String, Value>) -> Option<&str> {
    object.get("$id").and_then(Value::as_str)
}

pub(crate) fn scope_of(schema: &Value) -> Url {
    if let Some(id) = id_of(schema) {
        Url::parse(id).unwrap()
    } else {
        DEFAULT_SCOPE.clone()
    }
}

#[derive(Debug)]
pub(crate) struct Resolver<'schema> {
    document: &'schema Value,
    schemas: HashMap<String, &'schema Value>,
    scope: Url,
}

impl<'schema> Resolver<'schema> {
    pub(crate) fn new(document: &'schema Value, scope: Url) -> Self {
        let schemas = collect_schemas(document, scope.clone());
        Self {
            document,
            schemas,
            scope,
        }
    }

    pub(crate) fn resolve(&self, reference: &str) -> Option<&'schema Value> {
        // First, build the full URL that is aware of the resolution context
        // TODO. is it even needed? Context is always about this document
        let url = self.build_url(reference).unwrap();
        // Then, look for location-independent identifiers in the current schema
        if let Some(document) = self.schemas.get(url.as_str()) {
            Some(document)
        } else {
            // And resolve the reference in the stored document
            let pointer = to_pointer(&url);
            if pointer == "#" {
                Some(self.document)
            } else {
                // TODO. use a more efficient impl
                self.document.pointer(&pointer)
            }
        }
    }
    pub(crate) fn build_url(&self, reference: &str) -> Result<Url, ParseError> {
        Url::options().base_url(Some(&self.scope)).parse(reference)
    }
}

fn to_pointer(url: &Url) -> Cow<str> {
    percent_encoding::percent_decode_str(url.fragment().unwrap_or(""))
        .decode_utf8()
        .unwrap()
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
        let mut scope = $scopes[$scope_idx].join($id).unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use test_case::test_case;

    fn default() -> Value {
        json!({
            "allOf": [{
                "$ref": "#foo"
            }],
            "definitions": {
                "A": {
                    "$id": "#foo",
                    "type": "integer"
                }
            }
        })
    }

    fn absolute_uri() -> Value {
        json!({
            "allOf": [{
                "$ref": "http://localhost:1234/bar#foo"
            }],
            "definitions": {
                "A": {
                    "$id": "http://localhost:1234/bar#foo",
                    "type": "integer"
                }
            }
        })
    }

    fn sub_schema_in_object() -> Value {
        json!({
            "allOf": [{"$ref": "#foo"}],
            "definitions": {
                "A": {"$id": "#foo", "type": "integer"}
            }
        })
    }

    fn sub_schemas_in_array() -> Value {
        json!({
            "definitions": {
                "A": [
                    {"$id": "#foo", "type": "integer"},
                    {"$id": "#bar", "type": "string"},
                ]
            }
        })
    }

    fn root_schema_id() -> Value {
        json!({
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
        })
    }

    fn base_uri_change_in_subschema() -> Value {
        json!({
            "$id": "http://localhost:1234/root",
            "allOf": [{
                "$ref": "http://localhost:1234/nested.json#foo"
            }],
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
        })
    }

    fn base_uri_change() -> Value {
        json!({
            "$id": "http://localhost:1234/",
            "items": {
                "$id":"folder/",
                "items": {"$ref": "folderInteger.json"}
            }
        })
    }

    fn base_uri_change_folder() -> Value {
        json!({
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
        })
    }

    #[test_case(default(), &["json-schema:///#foo"], &["/definitions/A"])]
    #[test_case(absolute_uri(), &["http://localhost:1234/bar#foo"], &["/definitions/A"])]
    #[test_case(
        sub_schema_in_object(),
        &["json-schema:///#foo"],
        &["/definitions/A"]
    )]
    #[test_case(
        sub_schemas_in_array(),
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
        root_schema_id(),
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
        base_uri_change_in_subschema(),
        &[
            "http://localhost:1234/root",
            "http://localhost:1234/nested.json",
            "http://localhost:1234/nested.json#foo",
        ],
        &[
            "",
            "/definitions/A",
            "/definitions/A/definitions/B",
        ]
    )]
    #[test_case(
        base_uri_change(),
        &[
            "http://localhost:1234/",
            "http://localhost:1234/folder/"
        ],
        &[
            "",
            "/items"
        ]
    )]
    #[test_case(
        base_uri_change_folder(),
        &[
            "http://localhost:1234/scope_change_defs1.json",
            "http://localhost:1234/folder/",
        ],
        &[
            "",
            "/definitions/baz",
        ]
    )]
    fn location_identifiers(schema: Value, ids: &[&str], pointers: &[&str]) {
        let store = collect_schemas(&schema, scope_of(&schema));
        assert_eq!(store.len(), ids.len());
        for (id, pointer) in ids.into_iter().zip(pointers.into_iter()) {
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
        assert_eq!(scope_of(&schema), Url::parse(url).unwrap());
    }
}
