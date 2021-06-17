//! Reference resolver. Implements logic, required by `$ref` keyword.
//! Is able to load documents from remote locations via HTTP(S).
use crate::{
    compilation::{DEFAULT_ROOT_URL, DEFAULT_SCOPE},
    error::ValidationError,
    schemas::{id_of, Draft},
};
use ahash::AHashMap;
use parking_lot::RwLock;
use serde_json::Value;
use std::borrow::Cow;
use url::Url;

#[derive(Debug)]
pub(crate) struct Resolver<'a> {
    // canonical_id: sub-schema mapping to resolve documents by their ID
    // canonical_id is composed with the root document id
    // (if not specified, then `DEFAULT_ROOT_URL` is used for this purpose)
    schemas: AHashMap<String, &'a Value>,
    store: RwLock<AHashMap<String, Value>>,
}

impl<'a> Resolver<'a> {
    pub(crate) fn new(
        draft: Draft,
        scope: &Url,
        schema: &'a Value,
        store: AHashMap<String, Value>,
    ) -> Result<Resolver<'a>, ValidationError<'a>> {
        let mut schemas = AHashMap::new();
        // traverse the schema and store all named ones under their canonical ids
        find_schemas(draft, schema, scope, &mut |id, schema| {
            schemas.insert(id, schema);
            None
        })?;
        Ok(Resolver {
            schemas,
            store: RwLock::new(store),
        })
    }

    /// Load a document for the given `url`.
    /// It may be:
    ///   - the root document (`DEFAULT_ROOT_URL`) case;
    ///   - named subschema that is stored in `self.schemas`;
    ///   - document from a remote location;
    fn resolve_url(&self, url: &Url, schema: &'a Value) -> Result<Cow<'a, Value>, ValidationError> {
        match url.as_str() {
            DEFAULT_ROOT_URL => Ok(Cow::Borrowed(schema)),
            url_str => {
                if let Some(cached) = self.store.read().get(url_str) {
                    return Ok(Cow::Owned(cached.clone()));
                }
                match self.schemas.get(url_str) {
                    Some(value) => Ok(Cow::Borrowed(value)),
                    None => match url.scheme() {
                        "http" | "https" => {
                            #[cfg(any(feature = "reqwest", test))]
                            {
                                let response = reqwest::blocking::get(url.as_str())?;
                                let document: Value = response.json()?;
                                self.store
                                    .write()
                                    .insert(url_str.to_string(), document.clone());
                                Ok(Cow::Owned(document))
                            }
                            #[cfg(not(any(feature = "reqwest", test)))]
                            panic!("trying to resolve an http(s), but reqwest support has not been included");
                        }
                        http_scheme => Err(ValidationError::unknown_reference_scheme(
                            http_scheme.to_owned(),
                        )),
                    },
                }
            }
        }
    }
    pub(crate) fn resolve_fragment(
        &self,
        draft: Draft,
        url: &Url,
        schema: &'a Value,
    ) -> Result<(Url, Cow<'a, Value>), ValidationError> {
        let mut resource = url.clone();
        resource.set_fragment(None);
        let fragment =
            percent_encoding::percent_decode_str(url.fragment().unwrap_or("")).decode_utf8()?;

        // Location-independent identifiers are searched before trying to resolve by
        // fragment-less url
        if let Some(x) = find_schemas(draft, schema, &DEFAULT_SCOPE, &mut |id, x| {
            if id == url.as_str() {
                Some(x)
            } else {
                None
            }
        })? {
            return Ok((resource, Cow::Borrowed(x)));
        }

        // Each resolved document may be in a changed subfolder
        // They are tracked when JSON pointer is resolved and added to the resource
        match self.resolve_url(&resource, schema)? {
            Cow::Borrowed(document) => match pointer(draft, document, fragment.as_ref()) {
                Some((folders, resolved)) => {
                    Ok((join_folders(resource, &folders)?, Cow::Borrowed(resolved)))
                }
                None => Err(ValidationError::invalid_reference(url.as_str().to_string())),
            },
            Cow::Owned(document) => match pointer(draft, &document, fragment.as_ref()) {
                Some((folders, x)) => {
                    Ok((join_folders(resource, &folders)?, Cow::Owned(x.clone())))
                }
                None => Err(ValidationError::invalid_reference(url.as_str().to_string())),
            },
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

/// Based on `serde_json`, but tracks folders in the traversed documents.
pub(crate) fn pointer<'a>(
    draft: Draft,
    document: &'a Value,
    pointer: &str,
) -> Option<(Vec<&'a str>, &'a Value)> {
    if pointer.is_empty() {
        return Some((vec![], document));
    }
    if !pointer.starts_with('/') {
        return None;
    }
    let tokens = pointer
        .split('/')
        .skip(1)
        .map(|x| x.replace("~1", "/").replace("~0", "~"));
    let mut target = document;
    let mut folders = vec![];

    for token in tokens {
        let target_opt = match *target {
            Value::Object(ref map) => {
                if let Some(id) = id_of(draft, target) {
                    folders.push(id);
                }
                map.get(&token)
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
    use crate::JSONSchema;
    use serde_json::json;
    use std::borrow::Cow;
    use url::Url;

    fn make_resolver(schema: &Value) -> Resolver {
        Resolver::new(
            Draft::Draft7,
            &Url::parse("json-schema:///").unwrap(),
            schema,
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
            resolver.schemas.get("json-schema:///#foo"),
            schema.pointer("/definitions/A").as_ref()
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
            resolver.schemas.get("json-schema:///#foo"),
            schema.pointer("/definitions/A/0").as_ref()
        );
        assert_eq!(
            resolver.schemas.get("json-schema:///#bar"),
            schema.pointer("/definitions/A/1").as_ref()
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
            resolver.schemas.get("http://localhost:1234/tree"),
            schema.pointer("").as_ref()
        );
        assert_eq!(
            resolver.schemas.get("http://localhost:1234/node"),
            schema.pointer("/definitions/node").as_ref()
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
            resolver.schemas.get("http://localhost:1234/bar#foo"),
            schema.pointer("/definitions/A").as_ref()
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
            resolver.schemas.get("http://localhost:1234/root"),
            schema.pointer("").as_ref()
        );
        assert_eq!(
            resolver.schemas.get("http://localhost:1234/nested.json"),
            schema.pointer("/definitions/A").as_ref()
        );
        assert_eq!(
            resolver
                .schemas
                .get("http://localhost:1234/nested.json#foo"),
            schema.pointer("/definitions/A/definitions/B").as_ref()
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
            resolver.schemas.get("http://localhost:1234/"),
            schema.pointer("").as_ref()
        );
        assert_eq!(
            resolver.schemas.get("http://localhost:1234/folder/"),
            schema.pointer("/items").as_ref()
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
                .get("http://localhost:1234/scope_change_defs1.json"),
            schema.pointer("").as_ref()
        );
        assert_eq!(
            resolver.schemas.get("http://localhost:1234/folder/"),
            schema.pointer("/definitions/baz").as_ref()
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
        if let (resource, Cow::Borrowed(resolved)) = resolver
            .resolve_fragment(Draft::Draft7, &url, &schema)
            .unwrap()
        {
            assert_eq!(resource, Url::parse("json-schema:///").unwrap());
            assert_eq!(resolved, schema.pointer("/definitions/a").unwrap());
        }
    }

    #[test]
    fn id_value_is_cleaned() {
        let schema = json!({
            "$id": "http://foo.com/schema.json#",
            "properties": {
                "foo": {
                    "$ref": "#/definitions/Bar"
                }
            },
            "definitions": {
                "Bar": {
                    "const": 42
                }
            }
        });
        let compiled = JSONSchema::compile(&schema).unwrap();
        // `#` should be removed
        assert!(compiled
            .resolver
            .schemas
            .contains_key("http://foo.com/schema.json"));
    }
}
