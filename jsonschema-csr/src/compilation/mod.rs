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
    compilation::resolver::{scope_of, Resolver},
    vocabularies::Keyword,
};
use serde_json::Value;
use std::collections::HashMap;
use url::Url;

pub mod resolver;

#[derive(Debug)]
pub struct JsonSchema {
    keywords: Box<[Keyword]>,
    offsets: Box<[usize]>,
    edges: Box<[usize]>,
}

type RawEdge = (usize, usize);

impl JsonSchema {
    pub fn new(schema: &Value) -> Self {
        // Fetch all external schemas reachable from the root
        let external = fetch_external(schema);
        // Build resolvers for external schemas
        let resolvers = build_resolvers(&external);
        // Collect all values and resolve references
        let (values, mut edges) = collect(schema, &resolvers);
        // Replace all virtual edges with concrete ones and convert JSON values into keywords
        let keywords = concretize(values, &mut edges);
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
    let scope = scope_of(schema);
    let mut stack = vec![schema];
    while let Some(value) = stack.pop() {
        match value {
            // Only objects may have references to external schemas
            Value::Object(object) => {
                for (key, value) in object {
                    if key == "$ref" {
                        fetch_external_reference(value, store, &scope);
                    } else {
                        // Explore any other key
                        stack.push(value)
                    }
                }
            }
            // Explore arrays further
            Value::Array(items) => {
                stack.extend(items.iter());
            }
            // Primitive types do not contain any references, skip
            _ => continue,
        }
    }
}

/// If reference is pointing to an external resource, then fetch & store it.
fn fetch_external_reference(value: &Value, store: &mut HashMap<Url, Value>, scope: &Url) {
    if let Some(location) = without_fragment(value) {
        // Resolve only references that are:
        //   - pointing to another resource
        //   - are not already resolved
        if location != *scope && !store.contains_key(&location) {
            let response = reqwest::blocking::get(location.as_str()).unwrap();
            let document = response.json::<Value>().unwrap();
            // Make a recursive call to the callee routine
            fetch_routine(&document, store);
            store.insert(location, document);
        };
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
    let resolver = Resolver::new(schema, scope_of(schema));
    let mut stack = vec![(None, &resolver, schema)];
    while let Some((parent_idx, resolver, value)) = stack.pop() {
        // TODO.
        //   - validate - there should be no invalid schemas
        //   - Edge labels??? str or usize
        let node_idx = values.len();
        match value {
            Value::Object(object) => {
                // TODO. it could be just an object with key `$ref` - handle it
                if let Some(reference_value) = object.get("$ref") {
                    if let Value::String(reference) = reference_value {
                        match Url::parse(reference) {
                            // Remote reference
                            // TODO. what if it has the same scope as the local one?
                            Ok(mut url) => {
                                url.set_fragment(None);
                                let resolver = resolvers.get(url.as_str()).unwrap();
                                let resolved = resolver.resolve(reference).unwrap();
                                values.push(ValueReference::Virtual(resolved));
                                // TODO. What if the value was already explored??
                                stack.push((Some(node_idx), resolver, resolved));
                            }
                            // Local reference
                            Err(url::ParseError::RelativeUrlWithoutBase) => {
                                let resolved = resolver.resolve(reference).unwrap();
                                values.push(ValueReference::Virtual(resolved));
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
                    values.push(ValueReference::Concrete(value));
                    for (key, value) in object {
                        stack.push((Some(node_idx), resolver, value))
                    }
                }
            }
            Value::Array(items) => {
                values.push(ValueReference::Concrete(value));
                for item in items {
                    stack.push((Some(node_idx), resolver, item))
                }
            }
            _ => {
                values.push(ValueReference::Concrete(value));
            }
        };
        if let Some(parent_idx) = parent_idx {
            edges.push((parent_idx, node_idx));
        }
    }
    (values, edges)
}

/// Build a graph of `Keyword` instances with the same structure as the input one.
fn concretize(values: Vec<ValueReference>, edges: &mut Vec<RawEdge>) -> Vec<Keyword> {
    let nodes = Vec::with_capacity(values.len());
    for value in &values {
        match value {
            ValueReference::Concrete(_) => {}
            ValueReference::Virtual(reference) => {
                // If the target is simple enough, it could be inlined instead. It allows us to
                // avoid one reference jump during the validation process.
                //
                // Find the concrete reference by comparing the pointers
                // All virtual references point to values that are always in the `values` vector
                // Therefore this loop should always find such a reference
                for (target_idx, target) in values.iter().enumerate() {
                    if let ValueReference::Concrete(target) = target {
                        if std::ptr::eq(*reference, *target) {
                            println!("Target: {}", target_idx)
                        }
                    }
                }
            }
        }
    }
    nodes
}

fn compress(mut edges: Vec<RawEdge>) -> (Vec<usize>, Vec<usize>) {
    edges.sort_unstable_by_key(|edge| edge.0);
    let max_node_id = match edges.iter().map(|(x, y)| std::cmp::max(x, y)).max() {
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
                let (n, m) = *edge;
                if *n != node {
                    break 'inner;
                }
                edges.push(*m);
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
    use super::*;
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

    const SELF_REF: &str = "#";
    const REMOTE_REF: &str = "http://localhost:1234/subSchemas.json#/integer";
    const REMOTE_BASE: &str = "http://localhost:1234/subSchemas.json";

    // Boolean schema
    #[test_case(
        j!(true),
        &[c!(true)],
        &[],
        &[]
    )]
    // No references
    #[test_case(
        j!({"maximum": 5}),
        &[
            c!({"maximum": 5}),
            c!(5),
        ],
        &[(0, 1)],
        &[]
    )]
    // Recursive ref
    // TODO. the root schema should be present in the values
    #[test_case(
        j!({"$ref": SELF_REF}),
        &[
            v!({"$ref": SELF_REF}),
            c!(SELF_REF),
        ],
        &[(0, 1)],
        &[]
    )]
    // Remote ref - not resolved
    #[test_case(
        j!({"$ref": REMOTE_REF}),
        &[
            v!({"type": "integer"}),
            c!({"type": "integer"}),
            c!("integer"),
        ],
        &[(0, 1), (1, 2)],
        &[REMOTE_BASE]
    )]
    // Absolute ref to the same schema
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
            v!({
                "$id": "http://localhost:1234/root",
                "properties": {
                    "A": {
                        "$ref": "http://localhost:1234/root"
                    }
                }
            })
        ],
        &[],
        &[REMOTE_BASE]
    )]
    // TODO. refs without # - see `root_schema_id` test for context
    fn values_and_edges(
        schema: Value,
        values: &[ValueReference],
        edges: &[RawEdge],
        keys: &[&str],
    ) {
        let mut remote = HashMap::new();
        let (values_, edges_) = collect(&schema, &mut remote);
        assert_eq!(values_, values);
        assert_eq!(edges_, edges);
        assert_eq!(remote.keys().cloned().collect::<Vec<&str>>(), keys);
    }

    // One remote
    #[test_case(
        j!({"$ref": "http://localhost:1234/subSchemas.json"}),
        &["http://localhost:1234/subSchemas.json"]
    )]
    // One + two remote
    #[test_case(
        j!({"$ref": "http://localhost:1234/draft2019-09/metaschema-no-validation.json"}),
        &[
            "http://localhost:1234/draft2019-09/metaschema-no-validation.json",
            "https://json-schema.org/draft/2019-09/meta/applicator",
            "https://json-schema.org/draft/2019-09/meta/core"
        ]
    )]
    // Absolute ref to the root schema - is not a remote
    #[test_case(
        j!({"$id": REMOTE_BASE, "defs": {"type": "string"}, "$ref": REMOTE_BASE}),
        &[]
    )]
    // TODO. Remote without ID
    fn external_schemas(schema: Value, expected: &[&str]) {
        let external = fetch_external(&schema);
        assert!(
            expected
                .into_iter()
                .all(|key| external.contains_key(&Url::parse(key).unwrap())),
            "{:?}",
            external.keys()
        )
    }

    #[test]
    fn new_schema() {
        let schema = j!({"$ref": "http://localhost:1234/subSchemas.json"});
        let compiled = JsonSchema::new(&schema);
    }
}
