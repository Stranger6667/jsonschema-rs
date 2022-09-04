//! Compressed sparse row format for JSON Schema.
//!
//! Fast and cache efficient validation requires fast iteration over the schema, therefore a
//! representation like `serde_json::Value` should be converted to a more compact one.
//!
//! JSON Schema is a directed graph, where cycles could be represented via the `$ref` keyword.
//! It contains two types of edges:
//!   - Concrete edges are regular Rust references that connect two JSON values;
//!   - Virtual edges are using the `$ref` keyword to do this.
//!
//! TODO. add more theory about how `serde_json::Value` is represented and how CSR is represented
use crate::{
    compilation::resolver::{id_of_object, is_default_scope, scope_of, Resolver},
    vocabularies::Keyword,
};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use url::Url;

mod edges;
pub mod resolver;

use crate::{compilation::edges::EdgeLabel, vocabularies::Maximum};
use edges::{label, CompressedEdge, RawEdge};

#[derive(Debug)]
pub struct JsonSchema {
    keywords: Box<[Keyword]>,
    offsets: Box<[usize]>,
    edges: Box<[CompressedEdge]>,
}

impl JsonSchema {
    pub fn new(schema: &Value) -> Self {
        // Fetch all external schemas reachable from the root
        let external = fetch_external(schema);
        // Build resolvers for external schemas
        let resolvers = build_resolvers(&external);
        // Collect all values and resolve references
        let (values, mut edges) = collect(schema, &resolvers);
        // Build a `Keyword` graph together with dropping not needed nodes and edges
        let keywords = materialize(values, &mut edges);
        // And finally, compress the edges into the CSR format
        let (offsets, edges) = compress(edges);
        JsonSchema {
            keywords: keywords.into_boxed_slice(),
            offsets: offsets.into_boxed_slice(),
            edges: edges.into_boxed_slice(),
        }
    }
}

/// Fetch all external schemas reachable from the root.
fn fetch_external(schema: &Value) -> HashMap<Url, Value> {
    let mut store = HashMap::new();
    fetch_routine(schema, &mut store);
    store
}

/// Recursive routine for traversing a schema and fetching external references.
fn fetch_routine(schema: &Value, store: &mut HashMap<Url, Value>) {
    // Current schema id - if occurs in a reference, then there is no need to resolve it
    let scope = scope_of(schema).unwrap();
    let mut stack = vec![(vec![], schema)];
    while let Some((mut folders, value)) = stack.pop() {
        match value {
            // Only objects may have references to external schemas
            Value::Object(object) => {
                if let Some(id) = id_of_object(object) {
                    folders.push(id);
                }
                for (key, value) in object {
                    if key == "$ref" {
                        fetch_external_reference(value, &folders, store, &scope);
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
}

/// If reference is pointing to an external resource, then fetch & store it.
fn fetch_external_reference(
    value: &Value,
    folders: &[&str],
    store: &mut HashMap<Url, Value>,
    scope: &Url,
) {
    if let Some(location) = without_fragment(value) {
        // Resolve only references that are:
        //   - pointing to another resource
        //   - are not already resolved
        fetch_and_store(store, scope, location);
    } else if !is_default_scope(scope) {
        if let Some(reference) = value.as_str() {
            let location = with_folders(scope, reference, folders).unwrap();
            fetch_and_store(store, scope, location);
        }
    }
}

fn with_folders(scope: &Url, reference: &str, folders: &[&str]) -> Result<Url, url::ParseError> {
    let mut location = scope.clone();
    if folders.len() > 1 {
        for folder in folders.iter().skip(1) {
            location = location.join(folder)?;
        }
    }
    location.join(reference)
}

fn fetch_and_store(store: &mut HashMap<Url, Value>, scope: &Url, location: Url) {
    if location != *scope && !store.contains_key(&location) {
        let response = reqwest::blocking::get(location.as_str()).unwrap();
        let document = response.json::<Value>().unwrap();
        // Make a recursive call to the callee routine
        fetch_routine(&document, store);
        store.insert(location, document);
    }
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

fn build_resolvers(external: &HashMap<Url, Value>) -> HashMap<&str, Resolver> {
    let mut resolvers = HashMap::with_capacity(external.len());
    for (scope, document) in external {
        resolvers.insert(scope.as_str(), Resolver::new(document, scope.clone()));
    }
    resolvers
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ValueReference<'schema> {
    /// Reference to a concrete JSON value.
    Concrete(&'schema Value),
    /// Resolved `$ref` to a JSON value.
    Virtual(&'schema Value),
}

/// The main goal of this phase is to collect all nodes from the input schema and its remote
/// dependencies into a single graph where each vertex is a reference to a JSON value from these
/// schemas. Each edge is represented as a pair indexes into the vertex vector.
///
/// This representation format is efficient to construct the schema graph, but not for input
/// validation. Input requires arbitrary traversal order from the root node, because it depends on
/// the input value - certain schema branches are needed only for certain types or values.
/// For example, `properties` sub-schemas are needed only if the input contains matching keys.
fn collect<'a>(
    schema: &'a Value,
    resolvers: &'a HashMap<&str, Resolver>,
) -> (Vec<ValueReference<'a>>, Vec<RawEdge>) {
    // TODO. idea - store values interleaved with edges to improve cache locality
    // TODO. maybe remove nodes that were not referenced by anything?
    let mut values = vec![];
    let mut edges = vec![];
    let resolver = Resolver::new(schema, scope_of(schema).unwrap());
    let mut stack = vec![(None, vec![], &resolver, schema)];
    let mut seen = HashSet::new();
    while let Some((parent, mut folders, mut resolver, value)) = stack.pop() {
        seen.insert(value as *const _);
        // TODO.
        //   - validate - there should be no invalid schemas
        let node_idx = values.len();
        match value {
            Value::Object(object) => {
                values.push(ValueReference::Concrete(value));
                if let Some(id) = id_of_object(object) {
                    folders.push(id);
                }
                // TODO. it could be just an object with key `$ref` - handle it
                if let Some(reference_value) = object.get("$ref") {
                    if let Value::String(reference) = reference_value {
                        match Url::parse(reference) {
                            // Remote reference
                            // Example: `http://localhost:1234/subSchemas.json#/integer`
                            // TODO. what if it has the same scope as the local one?
                            Ok(mut url) => {
                                url.set_fragment(None);
                                if let Some(external_resolver) = resolvers.get(url.as_str()) {
                                    resolver = external_resolver;
                                }
                                let resolved = resolver.resolve(reference).unwrap().unwrap();
                                values.push(ValueReference::Virtual(resolved));
                                if !seen.contains(&(resolved as *const _)) {
                                    stack.push((
                                        Some((node_idx, label("$ref"))),
                                        folders.clone(),
                                        resolver,
                                        resolved,
                                    ));
                                }
                            }
                            // Local reference
                            // Example: `#/foo/bar`
                            Err(url::ParseError::RelativeUrlWithoutBase) => {
                                let resolved = if folders.len() > 1 {
                                    let location =
                                        with_folders(resolver.scope(), reference, &folders)
                                            .unwrap();
                                    resolver =
                                        resolvers.get(location.as_str()).unwrap_or_else(|| {
                                            panic!("Failed to find resolver for `{}`", location)
                                        });
                                    let resolved = resolver.resolve(reference).unwrap().unwrap();
                                    if !seen.contains(&(resolved as *const _)) {
                                        values.push(ValueReference::Concrete(resolved));
                                    };
                                    resolved
                                } else {
                                    resolver.resolve(reference).unwrap().unwrap()
                                };
                                values.push(ValueReference::Virtual(resolved));
                                // TODO. should external ones be explored? Likely so - they could
                                // have references too
                                if !seen.contains(&(reference_value as *const _)) {
                                    stack.push((
                                        Some((node_idx, label("$ref"))),
                                        folders.clone(),
                                        resolver,
                                        reference_value,
                                    ));
                                }
                            }
                            _ => todo!(),
                        }
                    } else {
                        todo!()
                    }
                } else {
                    // TODO. Order - some keywords might work better if they are first
                    // E.g - two keywords, both fail, but for `is_valid` we don't need both
                    // so, put the cheapest first
                    for (key, value) in object {
                        stack.push((
                            Some((node_idx, label(key))),
                            folders.clone(),
                            resolver,
                            value,
                        ))
                    }
                }
            }
            Value::Array(items) => {
                values.push(ValueReference::Concrete(value));
                for (idx, item) in items.iter().enumerate() {
                    stack.push((
                        Some((node_idx, label(idx))),
                        folders.clone(),
                        resolver,
                        item,
                    ))
                }
            }
            _ => {
                values.push(ValueReference::Concrete(value));
            }
        };
        if let Some((parent_idx, label)) = parent {
            edges.push(RawEdge::new(parent_idx, node_idx, label));
        }
    }
    (values, edges)
}

/// Build a graph of `Keyword` from an intermediate graph that contain all possible values.
///
/// There are a few optimizations happen here:
///  - Inlining virtual references
///  - Skipping not needed nodes
fn materialize(values: Vec<ValueReference>, edges: &mut Vec<RawEdge>) -> Vec<Keyword> {
    let mut nodes = Vec::with_capacity(values.len());
    for edge in edges {
        match &values[edge.target] {
            ValueReference::Concrete(value) => match &edge.label {
                EdgeLabel::Key(key) => match key.as_ref() {
                    // TODO. it should be at the right index - otherwise the graph is broken
                    "maximum" => nodes.push(Maximum::build(value.as_u64().unwrap())),
                    _ => {}
                },
                EdgeLabel::Index(_) => {}
            },
            // Each node that has an edge to `$ref` could be replaced by a new edge that leads
            // directly to the $ref target. Example:
            //
            //  {
            //     "integer": {
            //         "type": "integer"
            //     },
            //     "refToInteger": {
            //         "$ref": "#/integer"
            //     }
            // }
            //
            // This schema could be represented as:
            // [
            //     Concrete(root),                -- 0
            //     Virtual({"type": "integer"})   -- 1. refToInteger
            //     Concrete({"type": "integer"})  -- 2. integer
            //     Concrete("integer")            -- 3. type
            // ]
            //
            // Edges: 0-1 (refToInteger), 0-2 (integer), 2-3 (type)
            //
            // Here we change the first edge to 0-2. After all edges leading to this node are
            // inlined, node #1 is not needed anymore
            ValueReference::Virtual(reference) => {
                // TODO. detect cycles - get the target unless it is concrete + collect seen idx
                //   if idx is seen, then it is a cycle or refs without concrete nodes in between
                // Find the concrete reference by comparing the pointers
                // All virtual references point to values that are always in the `values` vector
                // Therefore this loop should always find such a reference
                for (target_idx, target) in values.iter().enumerate() {
                    if let ValueReference::Concrete(target) = target {
                        if std::ptr::eq(*reference, *target) {
                            edge.target = target_idx;
                        }
                    }
                }
            }
        }
    }
    nodes
}

fn compress(mut edges: Vec<RawEdge>) -> (Vec<usize>, Vec<CompressedEdge>) {
    edges.sort_unstable_by_key(|edge| edge.source);
    let max_node_id = match edges
        .iter()
        .map(|edge| std::cmp::max(edge.source, edge.target))
        .max()
    {
        None => return (vec![0], vec![]),
        Some(id) => id,
    };
    let mut iter = edges.iter().peekable();
    let mut edges = Vec::new();
    let mut offsets = vec![0_usize; max_node_id + 2];
    let mut rows = offsets.iter_mut();
    let mut start = 0;
    'outer: for (node, row) in (&mut rows).enumerate() {
        *row = start;
        'inner: loop {
            if let Some(edge) = iter.peek() {
                if edge.source != node {
                    break 'inner;
                }
                edges.push(edge.compress());
                start += 1;
            } else {
                break 'outer;
            }
            iter.next();
        }
    }
    for row in rows {
        *row = start;
    }
    (offsets, edges)
}

#[cfg(test)]
mod tests {
    use super::{edges::EdgeLabel, *};
    use serde_json::{json as j, Value};
    use test_case::test_case;

    macro_rules! c {
        ($el:tt) => {
            ValueReference::Concrete(&j!($el))
        };
    }

    macro_rules! v {
        ($el:tt) => {
            ValueReference::Virtual(&j!($el))
        };
    }

    // TODO: write a helper that any virtual node references a concrete one

    pub(crate) fn edge(source: usize, target: usize, label: impl Into<EdgeLabel>) -> RawEdge {
        RawEdge {
            source,
            target,
            label: label.into(),
        }
    }

    fn print_values(values: &[ValueReference]) {
        for (i, v) in values.iter().enumerate() {
            match v {
                ValueReference::Concrete(r) => {
                    println!("C({}): {}", i, r)
                }
                ValueReference::Virtual(r) => {
                    println!("V({}): {}", i, r)
                }
            }
        }
    }

    const SELF_REF: &str = "#";
    const REMOTE_REF: &str = "http://localhost:1234/subSchemas.json#/integer";
    const REMOTE_BASE: &str = "http://localhost:1234/subSchemas.json";

    #[test_case(
        j!(true),
        &[c!(true)],
        &[],
        &[];
        "Boolean schema"
    )]
    #[test_case(
        j!({"maximum": 5}),
        &[
            c!({"maximum": 5}),
            c!(5),
        ],
        &[edge(0, 1, "maximum")],
        &[];
        "No reference"
    )]
    #[test_case(
        j!({"$ref": SELF_REF}),
        &[
            c!({"$ref": SELF_REF}),
            v!({"$ref": SELF_REF}),
            c!(SELF_REF),
        ],
        &[edge(0, 2, "$ref")],
        &[];
        "Self reference"
    )]
    #[test_case(
        j!({"$ref": REMOTE_REF}),
        &[
            c!({"$ref": REMOTE_REF}),
            v!({"type": "integer"}),
            c!({"type": "integer"}),
            c!("integer"),
        ],
        &[edge(0, 2, "$ref"), edge(2, 3, "type")],
        &[REMOTE_BASE];
        "Remote reference"
    )]
    #[test_case(
        j!({
            "$id": "http://localhost:1234/",
            "items": {
                "$id": "baseUriChange/",
                "items": {"$ref": "folderInteger.json"}
            }
        }),
        &[
            c!({
                "$id": "http://localhost:1234/",
                "items": {
                    "$id": "baseUriChange/",
                    "items": {"$ref": "folderInteger.json"}
                }
            }),
            c!({
                "$id": "baseUriChange/",
                "items": {"$ref": "folderInteger.json"}
            }),
            c!({"$ref": "folderInteger.json"}),
            c!({"type": "integer"}),
            v!({"type": "integer"}),
            c!("folderInteger.json"),
            c!("baseUriChange/"),
            c!("http://localhost:1234/"),
        ],
        &[
            edge(0, 1, "items"),
            edge(1, 2, "items"),
            edge(2, 5, "$ref"),
            edge(1, 6, "$id"),
            edge(0, 7, "$id"),
            // TODO: there should be edges to `{"type": "integer"}`
        ],
        &["http://localhost:1234/baseUriChange/folderInteger.json"];
        "Base URI change"
    )]
    #[test_case(
        j!({
            "$ref": "http://localhost:1234/subSchemas.json#/refToInteger"
        }),
        &[
            c!({
                "$ref": "http://localhost:1234/subSchemas.json#/refToInteger"
            }),
            v!({"$ref":"#/integer"}),
            c!({"$ref":"#/integer"}),
            v!({"type":"integer"}),
            c!("#/integer"),
        ],
        &[
            edge(0, 2, "$ref"),
            edge(2, 4, "$ref"),
        ],
        &["http://localhost:1234/subSchemas.json"];
        "Reference within remote reference"
    )]
    #[test_case(
        j!({
            "$id": "http://localhost:1234/root",
            "properties": {
                "A": {
                    "$ref": "http://localhost:1234/root"
                }
            }
        }),
        &[
            c!({
                "$id": "http://localhost:1234/root",
                "properties": {
                    "A": {
                        "$ref": "http://localhost:1234/root"
                    }
                }
            }),
            c!({
                "A": {
                    "$ref": "http://localhost:1234/root"
                }
            }),
            c!({"$ref": "http://localhost:1234/root"}),
            v!({
                "$id": "http://localhost:1234/root",
                "properties": {
                    "A": {
                        "$ref": "http://localhost:1234/root"
                    }
                }
            }),
            c!("http://localhost:1234/root"),
        ],
        &[
            edge(0, 1, "properties"),
            edge(1, 2, "A"),
            edge(0, 4, "$id"),
        ],
        &[];
        "Absolute reference to the same schema"
    )]
    // TODO. refs without # - see `root_schema_id` test for context
    fn values_and_edges(
        schema: Value,
        values: &[ValueReference],
        edges: &[RawEdge],
        keys: &[&str],
    ) {
        let external = fetch_external(&schema);
        let resolvers = build_resolvers(&external);
        let (values_, edges_) = collect(&schema, &resolvers);
        print_values(&values_);
        assert_eq!(values_, values);
        assert_eq!(edges_, edges);
        assert_eq!(resolvers.keys().cloned().collect::<Vec<&str>>(), keys);
    }

    #[test_case(
        j!({"$ref": "http://localhost:1234/subSchemas.json"}),
        &["http://localhost:1234/subSchemas.json"];
        "One remote"
    )]
    #[test_case(
        j!({"$ref": "http://localhost:1234/draft2019-09/metaschema-no-validation.json"}),
        &[
            "http://localhost:1234/draft2019-09/metaschema-no-validation.json",
            "https://json-schema.org/draft/2019-09/meta/applicator",
            "https://json-schema.org/draft/2019-09/meta/core"
        ];
        "One + two remote"
    )]
    #[test_case(
        j!({"$id": REMOTE_BASE, "defs": {"type": "string"}, "$ref": REMOTE_BASE}),
        &[];
        "Absolute reference to the root schema which is not a remote"
    )]
    // TODO. Remote without ID
    fn external_schemas(schema: Value, expected: &[&str]) {
        let external = fetch_external(&schema);
        assert!(
            expected
                .iter()
                .all(|key| external.contains_key(&Url::parse(key).unwrap())),
            "{:?}",
            external.keys()
        )
    }

    #[test]
    fn new_schema() {
        let schema = j!({
            "integer": {
                "type": "integer"
            },
            "refToInteger": {
                "$ref": "#/integer"
            }
        });
        let compiled = JsonSchema::new(&schema);
    }

    #[test_case(
        j!({"maximum": 1}),
        vec![edge(0, 1, "maximum")];
        "Single keyword"
    )]
    #[test_case(
        j!({"foo": {"maximum": 1}, "bar": {"$ref": "#/foo"}}),
        vec![
            edge(0, 1, "foo"),
            edge(1, 2, "maximum"),
            edge(0, 3, "bar"),
            edge(3, 5, "$ref"),
        ];
        "Reference to another keyword"
    )]
    fn materialization(schema: Value, expected: Vec<RawEdge>) {
        let resolvers = HashMap::new();
        let (values, mut edges) = collect(&schema, &resolvers);
        print_values(&values);
        let _ = materialize(values, &mut edges);
        assert_eq!(edges, expected)
    }
}
