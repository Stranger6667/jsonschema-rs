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
//!
//! Here is a high-level algorithm used for building a compact schema representation:
//!   1. Recursively fetch all external schemas reachable from the input schema via references.
//!   2. Build reference resolvers for all schemas. At this point any reference in this set of
//!      schemas is resolvable locally without re-fetching.
//!   3. Traverse the input schema, and collect all reachable nodes into two vectors (nodes & edges).
//!      Traversal includes nodes referenced via `$ref`, and retain the original node types.
//!   4. The intermediate graph is minimized by removing nodes & edges that are not needed for
//!      validation, or represent default behavior (e.g. `uniqueItems: false`).
//!   5. Re-build the graph by transforming the input nodes into ones that contain pre-processed
//!      data needed for validation and do not depend on the original input type.
mod collection;
pub(crate) mod edges;
mod error;
mod references;
pub mod resolving;
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
//     Example: `anyOf` where some items are very cheap and common to pass validation.
//     collect average distance between two subsequent array accesses to measure it
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
        // let values = materialize(values, &mut edges);
        let (head_end, keywords) = build(values, &mut edges);
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

/// Build a keyword graph.
fn build(values: Vec<&Value>, edges: &mut Vec<RawEdge>) -> (usize, Vec<Keyword>) {
    // TODO: Some edges are useless - they don't represent any useful keyword
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
    (head_end, keywords)
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

    fn edge(source: usize, target: usize, label: impl Into<EdgeLabel>) -> RawEdge {
        RawEdge::new(source, target, label.into())
    }

    const SELF_REF: &str = "#";
    const REMOTE_REF: &str = "http://localhost:1234/subSchemas.json#/integer";
    const REMOTE_BASE: &str = "http://localhost:1234/subSchemas.json";

    #[test_case(
        j!(true),
        &[j!(true)],
        &[],
        &[];
        "Boolean schema"
    )]
    #[test_case(
        j!({"maximum": 5}),
        &[
            j!({"maximum": 5}),
            j!(5),
        ],
        &[edge(0, 1, KeywordName::Maximum)],
        &[];
        "No reference"
    )]
    #[test_case(
        j!({"properties": {"maximum": true}}),
        &[
            j!({"properties": {"maximum": true}}),
            j!({"maximum": true}),
            j!(true),
        ],
        &[
            edge(0, 1, KeywordName::Properties),
            edge(1, 2, "maximum"),
        ],
        &[];
        "Not a keyword"
    )]
    #[test_case(
        j!({
            "properties": {
                "$ref": {
                    "maximum": 5
                }
            }
        }),
        &[
            j!({"properties":{"$ref":{"maximum": 5}}}),
            j!({"$ref":{"maximum": 5}}),
            j!({"maximum": 5}),
            j!(5),
        ],
        &[
            edge(0, 1, KeywordName::Properties),
            edge(1, 2, "$ref"),
            edge(2, 3, KeywordName::Maximum),
        ],
        &[];
        "Not a reference"
    )]
    #[test_case(
        j!({"$ref": SELF_REF}),
        &[
            j!({"$ref": SELF_REF}),
        ],
        &[
            edge(0, 0, KeywordName::Ref),
        ],
        &[];
        "Self reference"
    )]
    #[test_case(
        j!({"$ref": REMOTE_REF}),
        &[
            j!({"$ref": REMOTE_REF}),
            j!({"type": "integer"}),
            j!("integer"),
        ],
        &[
            edge(1, 2, KeywordName::Type),
            edge(0, 1, KeywordName::Ref),
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
        ],
        &[
            edge(0, 1, KeywordName::Items),
            edge(1, 2, KeywordName::Items),
            edge(3, 4, KeywordName::Type),
            edge(2, 3, KeywordName::Ref),
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
        ],
        &[
            edge(0, 1, KeywordName::Properties),
            edge(1, 2, "list"),
            edge(2, 3, KeywordName::Ref),
        ],
        &[];
        "Base URI change - change folder"
    )]
    #[test_case(
        j!({
            "$ref": "http://localhost:1234/subSchemas.json#/refToInteger"
        }),
        &[
            j!({"$ref": "http://localhost:1234/subSchemas.json#/refToInteger"}),
            j!({"$ref":"#/integer"}),
            j!({"type":"integer"}),
            j!("integer"),
        ],
        &[
            edge(2, 3, KeywordName::Type),
            edge(1, 2, KeywordName::Ref),
            edge(0, 1, KeywordName::Ref),
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
        ],
        &[
            edge(0, 1, KeywordName::Properties),
            edge(1, 2, "A"),
            edge(2, 0, KeywordName::Ref),
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
            j!({"allOf":[{"$ref":"#/allOf/1"},{"$ref":"#/allOf/0"}]}),
            j!([{"$ref":"#/allOf/1"},{"$ref":"#/allOf/0"}]),
            j!({"$ref":"#/allOf/1"}),
            j!({"$ref":"#/allOf/0"}),
        ],
        &[
            edge(0, 1, KeywordName::AllOf),
            edge(1, 2, 0),
            edge(3, 2, KeywordName::Ref),
            edge(2, 3, KeywordName::Ref),
            edge(1, 3, 1),
        ],
        &[];
        "Multiple references to the same target"
    )]
    #[test_case(
        j!({
            "$id": "http://localhost:1234/tree",
            "properties": {
                "nodes": {
                    "items": {"$ref": "node"}
                }
            },
            "definitions": {
                "node": {
                    "$id": "http://localhost:1234/node",
                    "properties": {
                        "subtree": {"$ref": "tree"}
                    }
                }
            }
        }),
        &[
            j!({"$id":"http://localhost:1234/tree","definitions":{"node":{"$id":"http://localhost:1234/node","properties":{"subtree":{"$ref":"tree"}}}},"properties":{"nodes":{"items":{"$ref":"node"}}}}),
            j!({"nodes":{"items":{"$ref":"node"}}}),
            j!({"items":{"$ref":"node"}}),
            j!({"$ref":"node"}),
            j!({"$id":"http://localhost:1234/node","properties":{"subtree":{"$ref":"tree"}}}),
            j!({"subtree":{"$ref":"tree"}}),
            j!({"$ref":"tree"}),
        ],
        &[
            edge(0, 1, KeywordName::Properties),
            edge(1, 2, "nodes"),
            edge(2, 3, KeywordName::Items),
            edge(4, 5, KeywordName::Properties),
            edge(5, 6, "subtree"),
            edge(6, 0, KeywordName::Ref),
            edge(3, 4, KeywordName::Ref),
        ],
        &[];
        "Recursive references between schemas"
    )]
    fn values_and_edges(schema: Value, values: &[Value], edges: &[RawEdge], keys: &[&str]) {
        let root = resolving::Resolver::new(&schema).unwrap();
        let external = resolving::fetch_external(&schema, &root).unwrap();
        let resolvers = resolving::build_resolvers(&external);
        let (values_, edges_) = collection::collect(&schema, &root, &resolvers).unwrap();
        testing::print_values(&values_);
        let concrete_values = values_.into_iter().cloned().collect::<Vec<Value>>();
        assert_eq!(concrete_values, values);
        assert_eq!(edges_, edges);
        testing::assert_unique_edges(&edges_);
        assert_eq!(resolvers.keys().cloned().collect::<Vec<&str>>(), keys);
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
                        let (_, edges) = collection::collect(schema, &root, &resolvers).unwrap();
                        testing::assert_unique_edges(&edges);
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
