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
mod collection;
pub(crate) mod edges;
mod error;
mod references;
mod resolving;
#[cfg(test)]
mod testing;

use crate::{
    compilation::edges::EdgeLabel,
    vocabularies::{applicator, validation, Keyword, KeywordName},
};
use edges::{CompressedEdge, RawEdge};
use error::Result;
use serde_json::Value;

// TODO. Optimization ideas:
//   - Values ordering. "Cheaper" keywords might work better if they are executed first.
//     Example: `anyOf` where some items are very cheap and common to pass validation
//   - Interleave values & edges in the same struct. Might improve cache locality.
//   - Remove nodes that are not references by anything & not needed for validation.
//     Might reduce the graph size.
//   - Store root's keywords explicitly upfront to avoid calculation for small schemas

#[derive(Debug)]
pub struct JsonSchema {
    pub(crate) keywords: Box<[Keyword]>,
    head_end: usize,
    offsets: Box<[usize]>,
    pub(crate) edges: Box<[CompressedEdge]>,
}

impl JsonSchema {
    pub fn new(schema: &Value) -> Result<Self> {
        // Resolver for the root schema - needed to resolve location-independent references during
        // the initial resolving step
        let root_resolver = resolving::Resolver::new(schema)?;
        // Fetch all external schemas reachable from the root
        let external = resolving::fetch_external(schema, &root_resolver)?;
        // Build resolvers for external schemas
        let resolvers = resolving::build_resolvers(&external);
        // Collect all values and resolve references
        let (values, mut edges) = collection::collect(schema, &root_resolver, &resolvers)?;
        // Build a `Keyword` graph together with dropping not needed nodes and edges
        let mut values = materialize(values, &mut edges);
        let (head_end, keywords, edges) = build(values, &mut edges);
        // And finally, compress the edges into the CSR format
        let (offsets, edges) = compress(edges);
        Ok(JsonSchema {
            keywords: keywords.into_boxed_slice(),
            head_end,
            offsets: offsets.into_boxed_slice(),
            edges: edges.into_boxed_slice(),
        })
    }

    #[inline]
    pub(crate) fn edges_of(&self, node_idx: usize) -> impl Iterator<Item = &CompressedEdge> {
        let start = self.offsets[node_idx];
        let end = self.offsets[node_idx + 1];
        self.edges[start..end].iter()
    }

    pub fn is_valid(&self, instance: &Value) -> bool {
        self.keywords[..self.head_end]
            // TODO. each keyword should know its position, so it can find its edges
            .iter()
            .all(|keyword| keyword.is_valid(self, instance))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ValueReference<'schema> {
    /// Reference to a concrete JSON value.
    Concrete(&'schema Value),
    /// Resolved `$ref` to a JSON value.
    Virtual(&'schema Value),
}

/// Make all virtual edges point to concrete values.
fn materialize<'a>(values: Vec<ValueReference<'a>>, edges: &mut Vec<RawEdge>) -> Vec<&'a Value> {
    let mut concrete = vec![];
    for edge in edges.iter_mut() {
        if let ValueReference::Virtual(reference) = &values[edge.target] {
            // Find the concrete reference by comparing the pointers
            // All virtual references point to values that are always in the `values` vector
            // Therefore this loop should always find such a reference
            // TODO: Avoid O^2 - store seen targets & their indices.
            //       If not seen - check only the right side.
            for (target_idx, target) in values.iter().enumerate() {
                if let ValueReference::Concrete(target) = target {
                    if std::ptr::eq(*reference, *target) {
                        edge.target = target_idx;
                    }
                }
            }
        }
    }
    // Build a new vector of value by pushing only concrete values
    // When a virtual one occurs, then all further concrete nodes will have a smaller index
    let mut should_shift = false;
    for value in &values {
        match value {
            ValueReference::Concrete(reference) => {
                concrete.push(*reference);
                if should_shift {
                    for edge in &mut *edges {
                        if edge.source >= concrete.len() {
                            edge.source -= 1;
                        }
                        if edge.target >= concrete.len() {
                            edge.target -= 1;
                        }
                    }
                    should_shift = false;
                }
            }
            ValueReference::Virtual(_) => {
                should_shift = true;
            }
        }
    }
    // TODO. Is it possible to avoid deduplication?
    edges.sort_unstable_by_key(|edge| (edge.source, edge.target));
    edges.dedup();
    concrete
}

/// Build a keyword graph.
fn build(values: Vec<&Value>, edges: &mut Vec<RawEdge>) -> (usize, Vec<Keyword>, Vec<RawEdge>) {
    // TODO: edges should also be new - input ones refer to the original graph
    // TODO: Some edges are useless - they don't represent any useful keyword
    let mut new_edges = vec![];
    let mut keywords = Vec::with_capacity(values.len());
    edges.sort_unstable_by_key(|edge| edge.source);
    let mut head_end = 0_usize;
    for edge in edges {
        match &edge.label {
            EdgeLabel::Keyword(KeywordName::Maximum) => {
                if edge.source == 0 {
                    head_end += 1;
                }
                let value = values[edge.target];
                keywords.push(validation::Maximum::build(value.as_u64().unwrap()))
            }
            EdgeLabel::Keyword(KeywordName::Properties) => {
                if edge.source == 0 {
                    head_end += 1;
                }
                keywords.push(applicator::Properties::build())
            }
            _ => {}
        }
    }
    (head_end, keywords, new_edges)
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
    use bench_helpers::read_json;
    use serde_json::{json as j, Value};
    use std::fs;
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

    fn edge(source: usize, target: usize, label: impl Into<EdgeLabel>) -> RawEdge {
        RawEdge::new(source, target, label.into())
    }

    const SELF_REF: &str = "#";
    const REMOTE_REF: &str = "http://localhost:1234/subSchemas.json#/integer";
    const REMOTE_BASE: &str = "http://localhost:1234/subSchemas.json";

    #[test_case(
        j!(true),
        &[c!(true)],
        &[],
        &[j!(true)],
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
        &[edge(0, 1, KeywordName::Maximum)],
        &[j!({"maximum": 5}), j!(5)],
        &[edge(0, 1, KeywordName::Maximum)],
        &[];
        "No reference"
    )]
    #[test_case(
        j!({"properties": {"maximum": true}}),
        &[
            c!({"properties": {"maximum": true}}),
            c!({"maximum": true}),
            c!(true),
        ],
        &[
            edge(0, 1, KeywordName::Properties),
            edge(1, 2, "maximum"),
        ],
        &[j!({"maximum": 5}), j!(5)],
        &[edge(0, 1, KeywordName::Maximum)],
        &[];
        "Not a keyword"
    )]
    #[test_case(
        j!({
            "properties": {
                "$ref": {
                    "type": "string"
                }
            }
        }),
        &[
            c!({"properties":{"$ref":{"type":"string"}}}),
            c!({"$ref":{"type":"string"}}),
            c!({"type":"string"}),
            c!("string"),
        ],
        &[
            edge(0, 1, "properties"),
            edge(1, 2, "$ref"),
            edge(2, 3, "type"),
        ],
        &[
            j!({"properties":{"$ref":{"type":"string"}}}),
            j!({"$ref":{"type":"string"}}),
            j!({"type":"string"}),
            j!("string"),
        ],
        &[
            edge(0, 1, "properties"),
            edge(1, 2, "$ref"),
            edge(2, 3, "type"),
        ],
        &[];
        "Not a reference"
    )]
    #[test_case(
        j!({"$ref": SELF_REF}),
        &[
            c!({"$ref": SELF_REF}),
            v!({"$ref": SELF_REF}),
        ],
        &[
            edge(0, 1, "$ref"),
        ],
        &[
            j!({"$ref": SELF_REF}),
        ],
        &[
            edge(0, 0, "$ref"),
        ],
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
        &[
            edge(0, 1, "$ref"),
            edge(0, 2, "$ref"),
            edge(2, 3, "type")
        ],
        &[
            j!({"$ref": REMOTE_REF}),
            j!({"type": "integer"}),
            j!("integer"),
        ],
        &[
            edge(0, 1, "$ref"),
            edge(1, 2, "type")
        ],
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
            v!({"type": "integer"}),
            c!({"type": "integer"}),
            c!("integer"),
            c!("baseUriChange/"),
            c!("http://localhost:1234/"),
        ],
        &[
            edge(0, 1, "items"),
            edge(2, 3, "$ref"),
            edge(1, 2, "items"),
            edge(2, 4, "$ref"),
            edge(4, 5, "type"),
            edge(1, 6, "$id"),
            edge(0, 7, "$id"),
        ],
        &[
            j!({
                "$id": "http://localhost:1234/",
                "items": {
                    "$id": "baseUriChange/",
                    "items": {"$ref": "folderInteger.json"}
                }
            }),
            j!({
                "$id": "baseUriChange/",
                "items": {"$ref": "folderInteger.json"}
            }),
            j!({"$ref": "folderInteger.json"}),
            j!({"type": "integer"}),
            j!("integer"),
            j!("baseUriChange/"),
            j!("http://localhost:1234/"),
        ],
        &[
            edge(0, 1, "items"),
            edge(0, 6, "$id"),
            edge(1, 2, "items"),
            edge(1, 5, "$id"),
            edge(2, 3, "$ref"),
            edge(3, 4, "type"),
        ],
        &["http://localhost:1234/baseUriChange/folderInteger.json"];
        "Base URI change"
    )]
    #[test_case(
        j!({
            "$id": "http://localhost:1234/scope_change_defs1.json",
            "properties": {
                "list": {"$ref": "#/definitions"}
            },
            "definitions": {
                "$id": "baseUriChangeFolder/",
            }
        }),
        &[
            c!({
                "$id": "http://localhost:1234/scope_change_defs1.json",
                "properties": {
                    "list": {"$ref": "#/definitions"}
                },
                "definitions": {
                    "$id": "baseUriChangeFolder/",
                }
            }),
            c!({"list":{"$ref":"#/definitions"}}),
            c!({"$ref":"#/definitions"}),
            v!({"$id":"baseUriChangeFolder/"}),
            c!({"$id":"baseUriChangeFolder/"}),
            c!("baseUriChangeFolder/"),
            c!("http://localhost:1234/scope_change_defs1.json"),
        ],
        &[
            edge(0, 1, "properties"),
            edge(2, 3, "$ref"),
            edge(1, 2, "list"),
            edge(0, 4, "definitions"),
            edge(4, 5, "$id"),
            edge(0, 6, "$id"),
        ],
        &[
            j!({
                "$id": "http://localhost:1234/scope_change_defs1.json",
                "properties": {
                    "list": {"$ref": "#/definitions"}
                },
                "definitions": {
                    "$id": "baseUriChangeFolder/",
                }
            }),
            j!({"list":{"$ref":"#/definitions"}}),
            j!({"$ref":"#/definitions"}),
            j!({"$id":"baseUriChangeFolder/"}),
            j!("baseUriChangeFolder/"),
            j!("http://localhost:1234/scope_change_defs1.json"),
        ],
        &[
            edge(0, 1, "properties"),
            edge(0, 3, "definitions"),
            edge(0, 5, "$id"),
            edge(1, 2, "list"),
            edge(2, 3, "$ref"),
            edge(3, 4, "$id"),
        ],
        &[];
        "Base URI change - change folder"
    )]
    #[test_case(
        j!({
            "$ref": "http://localhost:1234/subSchemas.json#/refToInteger"
        }),
        &[
            c!({"$ref": "http://localhost:1234/subSchemas.json#/refToInteger"}),
            v!({"$ref":"#/integer"}),
            c!({"$ref":"#/integer"}),
            v!({"type":"integer"}),
            c!({"type":"integer"}),
            c!("integer"),
        ],
        &[
            edge(0, 1, "$ref"),
            edge(2, 3, "$ref"),
            edge(0, 2, "$ref"),
            edge(2, 4, "$ref"),
            edge(4, 5, "type"),
        ],
        &[
            j!({"$ref": "http://localhost:1234/subSchemas.json#/refToInteger"}),
            j!({"$ref":"#/integer"}),
            j!({"type":"integer"}),
            j!("integer"),
        ],
        &[
            edge(0, 1, "$ref"),
            edge(1, 2, "$ref"),
            edge(2, 3, "type"),
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
            edge(2, 3, "$ref"),
            edge(1, 2, "A"),
            edge(0, 4, "$id"),
        ],
        &[
            j!({
                "$id": "http://localhost:1234/root",
                "properties": {
                    "A": {
                        "$ref": "http://localhost:1234/root"
                    }
                }
            }),
            j!({
                "A": {
                    "$ref": "http://localhost:1234/root"
                }
            }),
            j!({"$ref": "http://localhost:1234/root"}),
            j!("http://localhost:1234/root"),
        ],
        &[
            edge(0, 1, "properties"),
            edge(0, 3, "$id"),
            edge(1, 2, "A"),
            edge(2, 0, "$ref"),
        ],
        &[];
        "Absolute reference to the same schema"
    )]
    #[test_case(
        j!({
            "allOf": [
                {"$ref": "#/allOf/1"},
                {"$ref": "#/allOf/0"}
            ]
        }),
        &[
            c!({"allOf":[{"$ref":"#/allOf/1"},{"$ref":"#/allOf/0"}]}),
            c!([{"$ref":"#/allOf/1"},{"$ref":"#/allOf/0"}]),
            c!({"$ref":"#/allOf/0"}),
            v!({"$ref":"#/allOf/1"}),
            c!({"$ref":"#/allOf/1"}),
            v!({"$ref":"#/allOf/0"}),
        ],
        &[
            edge(0, 1, "allOf"),
            edge(2, 3, "$ref"),
            edge(1, 2, 1),
            edge(4, 5, "$ref"),
            edge(1, 4, 0),
        ],
        &[
            j!({"allOf":[{"$ref":"#/allOf/1"},{"$ref":"#/allOf/0"}]}),
            j!([{"$ref":"#/allOf/1"},{"$ref":"#/allOf/0"}]),
            j!({"$ref":"#/allOf/0"}),
            j!({"$ref":"#/allOf/1"}),
        ],
        &[
            edge(0, 1, "allOf"),
            edge(1, 2, 1),
            edge(1, 3, 0),
            edge(2, 3, "$ref"),
            edge(3, 2, "$ref"),
        ],
        &[];
        "Multiple references to the same target"
    )]
    #[test_case(
        j!({
            "$id": "http://localhost:1234/tree",
            "d": {
                "$id": "http://localhost:1234/node",
                "s": {
                    "$ref": "tree"
                },
            },
            "n": {
                "$ref": "node"
            }
        }),
        &[
            c!({"$id":"http://localhost:1234/tree","d":{"$id":"http://localhost:1234/node","s":{"$ref":"tree"}},"n":{"$ref":"node"}}),
            c!({"$ref":"node"}),
            v!({"$id":"http://localhost:1234/node","s":{"$ref":"tree"}}),
            c!({"$id":"http://localhost:1234/node","s":{"$ref":"tree"}}),
            c!({"$ref":"tree"}),
            v!({"$id":"http://localhost:1234/tree","d":{"$id":"http://localhost:1234/node","s":{"$ref":"tree"}},"n":{"$ref":"node"}}),
            c!("http://localhost:1234/node"),
            c!("http://localhost:1234/tree"),
        ],
        &[
            edge(1, 2, "$ref"),
            edge(0, 1, "n"),
            edge(0, 3, "d"),
            edge(4, 5, "$ref"),
            edge(3, 4, "s"),
            edge(3, 6, "$id"),
            edge(0, 7, "$id"),
        ],
        &[
            j!({"$id":"http://localhost:1234/tree","d":{"$id":"http://localhost:1234/node","s":{"$ref":"tree"}},"n":{"$ref":"node"}}),
            j!({"$ref":"node"}),
            j!({"$id":"http://localhost:1234/node","s":{"$ref":"tree"}}),
            j!({"$ref":"tree"}),
            j!("http://localhost:1234/node"),
            j!("http://localhost:1234/tree"),
        ],
        &[
            edge(0, 1, "n"),
            edge(0, 2, "d"),
            edge(0, 5, "$id"),
            edge(1, 2, "$ref"),
            edge(2, 3, "s"),
            edge(2, 4, "$id"),
            edge(3, 0, "$ref"),
        ],
        &[];
        "Recursive references between schemas"
    )]
    fn values_and_edges(
        schema: Value,
        values: &[ValueReference],
        edges: &[RawEdge],
        concrete_values: &[Value],
        concrete_edges: &[RawEdge],
        keys: &[&str],
    ) {
        let root = resolving::Resolver::new(&schema).unwrap();
        let external = resolving::fetch_external(&schema, &root).unwrap();
        let resolvers = resolving::build_resolvers(&external);
        let (values_, mut edges_) = collection::collect(&schema, &root, &resolvers).unwrap();
        testing::print_values(&values_);
        assert_eq!(values_, values);
        testing::assert_concrete_references(&values_);
        testing::assert_virtual_references(&values_);
        assert_eq!(edges_, edges);
        assert_eq!(resolvers.keys().cloned().collect::<Vec<&str>>(), keys);
        let mut values = materialize(values_, &mut edges_);
        minimize(&mut values, &mut edges_);
        assert_eq!(
            values.into_iter().cloned().collect::<Vec<Value>>(),
            concrete_values
        );
        assert_eq!(edges_, concrete_edges);
    }

    #[test]
    fn all_schemas() {
        for draft in &[7] {
            let paths =
                fs::read_dir(format!("../jsonschema/tests/suite/tests/draft{}", draft)).unwrap();
            for path in paths {
                let entry = path.unwrap();
                let path = entry.path();
                if path.is_file() && path.extension().unwrap().to_str().unwrap() == "json" {
                    println!("File: {}", path.display());
                    let data = read_json(path.as_path().to_str().unwrap());
                    for (idx, case) in data.as_array().unwrap().iter().enumerate() {
                        println!("Case: {}", idx);
                        let schema = &case["schema"];
                        let root = resolving::Resolver::new(schema).unwrap();
                        let external = resolving::fetch_external(schema, &root).unwrap();
                        let resolvers = resolving::build_resolvers(&external);
                        let (values_, _) = collection::collect(schema, &root, &resolvers).unwrap();
                        testing::print_values(&values_);
                        testing::assert_concrete_references(&values_);
                        testing::assert_virtual_references(&values_);
                    }
                }
            }
        }
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
        let root = resolving::Resolver::new(&schema).unwrap();
        let external = resolving::fetch_external(&schema, &root).unwrap();
        assert!(
            expected
                .iter()
                .all(|key| external.contains_key(&url::Url::parse(key).unwrap())),
            "{:?}",
            external.keys()
        )
    }

    #[test_case(j!({"maximum": 5}), j!(4), j!(6))]
    #[test_case(j!({"properties": {"A": {"maximum": 5}}}), j!({"A": 4}), j!({"A": 6}))]
    fn is_valid(schema: Value, valid: Value, invalid: Value) {
        let compiled = JsonSchema::new(&schema).unwrap();
        assert!(compiled.is_valid(&valid));
        assert!(!compiled.is_valid(&invalid));
    }
}
