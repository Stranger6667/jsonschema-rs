//! Packed JSON Schema representation.
//!
//! This approach represents JSON Schema as a set of nodes & edges stored compactly in vectors.
//!
//! Fast and cache efficient validation requires fast iteration over the schema, therefore a
//! representation like `serde_json::Value` should be converted to a more compact one.
//!
//! JSON Schema is a directed graph, where cycles could be represented via the `$ref` keyword.
//! It contains two types of edges:
//!   - Concrete edges are regular Rust references that connect two JSON values;
//!   - Virtual edges are using the `$ref` keyword to do this.
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

use crate::vocabularies::{applicator, validation, Keyword, KeywordName};
use collection::{EdgeMap, KeywordMap};
use edges::Edge;
use error::Result;
use serde_json::Value;

// TODO. Optimization ideas:
//   - Values ordering. "Cheaper" keywords might work better if they are executed first.
//     Example: `anyOf` where some items are very cheap and common to pass validation.
//     collect average distance between two subsequent array accesses to measure it
//   - Interleave values & edges in the same struct. Might improve cache locality.

#[derive(Debug)]
pub struct JsonSchema {
    pub(crate) keywords: Box<[Keyword]>,
    head_end: usize,
    pub(crate) edges: Box<[Edge]>,
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
        let (keyword_map, edge_map) = collection::collect(schema, &root_resolver, &resolvers)?;
        // Build a `Keyword` graph
        let (head_end, keywords, edges) = build(&keyword_map, &edge_map);
        Ok(JsonSchema {
            keywords: keywords.into_boxed_slice(),
            head_end,
            edges: edges.into_boxed_slice(),
        })
    }

    pub fn is_valid(&self, instance: &Value) -> bool {
        if let [keyword] = &self.keywords[..] {
            keyword.is_valid(self, instance)
        } else {
            self.keywords[..self.head_end]
                .iter()
                .all(|keyword| keyword.is_valid(self, instance))
        }
    }
}

fn build(keyword_map: &KeywordMap<'_>, edge_map: &EdgeMap) -> (usize, Vec<Keyword>, Vec<Edge>) {
    let mut keywords = vec![];
    let mut edges = vec![];
    let head = keyword_map.get(&0).map_or(0, |kwords| kwords.len());
    for node_keywords in keyword_map.values() {
        let next_keyword_layer_start_idx = keywords.len() + node_keywords.len();
        for (target, value, keyword) in node_keywords {
            match keyword {
                KeywordName::AllOf => {
                    // TODO: child keywords should go immediately after this one, otherwise there is a gap
                    keywords.push(applicator::AllOf::build(
                        next_keyword_layer_start_idx,
                        next_keyword_layer_start_idx + edge_map[target].len(),
                    ));
                }
                KeywordName::Items => {}
                KeywordName::Maximum => {
                    keywords.push(validation::Maximum::build(value.as_u64().unwrap()))
                }
                KeywordName::Properties => {
                    keywords.push(applicator::Properties::build(
                        edges.len(),
                        edges.len() + edge_map[target].len(),
                    ));
                    let mut offset = 0;
                    for edge in &edge_map[target] {
                        let start = keywords.len() + offset;
                        let children_length = keyword_map[&edge.target].len();
                        let end = start + children_length;
                        edges.push(Edge::new(edge.label.clone(), start..end));
                        offset += children_length;
                    }
                }
                KeywordName::Ref => {}
                KeywordName::Type => {}
            }
        }
    }
    (head, keywords, edges)
}

#[cfg(test)]
mod tests {
    use super::{applicator::Properties, edges::EdgeLabel, validation::Maximum, *};
    use crate::compilation::edges::RawEdge;
    use bench_helpers::read_json;
    use serde_json::{json as j, Value};
    use std::fs;
    use test_case::test_case;

    fn raw(source: usize, target: usize, label: impl Into<EdgeLabel>) -> RawEdge {
        RawEdge::new(source, target, label.into())
    }

    const SELF_REF: &str = "#";
    const REMOTE_REF: &str = "http://localhost:1234/subSchemas.json#/integer";
    const REMOTE_BASE: &str = "http://localhost:1234/subSchemas.json";

    // #[test_case(
    //     j!(true),
    //     &[j!(true)],
    //     &[],
    //     &[];
    //     "Boolean schema"
    // )]
    // #[test_case(
    //     j!({"maximum": 5}),
    //     &[
    //         j!({"maximum": 5}),
    //         j!(5),
    //     ],
    //     &[raw(0, 1, KeywordName::Maximum)],
    //     &[];
    //     "No reference"
    // )]
    // #[test_case(
    //     j!({"properties": {"maximum": true}}),
    //     &[
    //         j!({"properties": {"maximum": true}}),
    //         j!({"maximum": true}),
    //         j!(true),
    //     ],
    //     &[
    //         raw(0, 1, KeywordName::Properties),
    //         raw(1, 2, "maximum"),
    //     ],
    //     &[];
    //     "Not a keyword"
    // )]
    // #[test_case(
    //     j!({
    //         "properties": {
    //             "$ref": {
    //                 "maximum": 5
    //             }
    //         }
    //     }),
    //     &[
    //         j!({"properties":{"$ref":{"maximum": 5}}}),
    //         j!({"$ref":{"maximum": 5}}),
    //         j!({"maximum": 5}),
    //         j!(5),
    //     ],
    //     &[
    //         raw(0, 1, KeywordName::Properties),
    //         raw(1, 2, "$ref"),
    //         raw(2, 3, KeywordName::Maximum),
    //     ],
    //     &[];
    //     "Not a reference"
    // )]
    // #[test_case(
    //     j!({"$ref": SELF_REF}),
    //     &[
    //         j!({"$ref": SELF_REF}),
    //     ],
    //     &[
    //         raw(0, 0, KeywordName::Ref),
    //     ],
    //     &[];
    //     "Self reference"
    // )]
    // #[test_case(
    //     j!({"$ref": REMOTE_REF}),
    //     &[
    //         j!({"$ref": REMOTE_REF}),
    //         j!({"type": "integer"}),
    //         j!("integer"),
    //     ],
    //     &[
    //         raw(1, 2, KeywordName::Type),
    //         raw(0, 1, KeywordName::Ref),
    //     ],
    //     &[REMOTE_BASE];
    //     "Remote reference"
    // )]
    // #[test_case(
    //     j!({
    //         "$id": "http://localhost:1234/",
    //         "items": {
    //             "$id": "baseUriChange/",
    //             "items": {"$ref": "folderInteger.json"}
    //         }
    //     }),
    //     &[
    //         j!({
    //             "$id": "http://localhost:1234/",
    //             "items": {
    //                 "$id": "baseUriChange/",
    //                 "items": {"$ref": "folderInteger.json"}
    //             }
    //         }),
    //         j!({
    //             "$id": "baseUriChange/",
    //             "items": {"$ref": "folderInteger.json"}
    //         }),
    //         j!({"$ref": "folderInteger.json"}),
    //         j!({"type": "integer"}),
    //         j!("integer"),
    //     ],
    //     &[
    //         raw(0, 1, KeywordName::Items),
    //         raw(1, 2, KeywordName::Items),
    //         raw(3, 4, KeywordName::Type),
    //         raw(2, 3, KeywordName::Ref),
    //     ],
    //     &["http://localhost:1234/baseUriChange/folderInteger.json"];
    //     "Base URI change"
    // )]
    // #[test_case(
    //     j!({
    //         "$id": "http://localhost:1234/scope_change_defs1.json",
    //         "properties": {
    //             "list": {"$ref": "#/definitions"}
    //         },
    //         "definitions": {
    //             "$id": "baseUriChangeFolder/",
    //         }
    //     }),
    //     &[
    //         j!({
    //             "$id": "http://localhost:1234/scope_change_defs1.json",
    //             "properties": {
    //                 "list": {"$ref": "#/definitions"}
    //             },
    //             "definitions": {
    //                 "$id": "baseUriChangeFolder/",
    //             }
    //         }),
    //         j!({"list":{"$ref":"#/definitions"}}),
    //         j!({"$ref":"#/definitions"}),
    //         j!({"$id":"baseUriChangeFolder/"}),
    //     ],
    //     &[
    //         raw(0, 1, KeywordName::Properties),
    //         raw(1, 2, "list"),
    //         raw(2, 3, KeywordName::Ref),
    //     ],
    //     &[];
    //     "Base URI change - change folder"
    // )]
    // #[test_case(
    //     j!({
    //         "$ref": "http://localhost:1234/subSchemas.json#/refToInteger"
    //     }),
    //     &[
    //         j!({"$ref": "http://localhost:1234/subSchemas.json#/refToInteger"}),
    //         j!({"$ref":"#/integer"}),
    //         j!({"type":"integer"}),
    //         j!("integer"),
    //     ],
    //     &[
    //         raw(2, 3, KeywordName::Type),
    //         raw(1, 2, KeywordName::Ref),
    //         raw(0, 1, KeywordName::Ref),
    //     ],
    //     &["http://localhost:1234/subSchemas.json"];
    //     "Reference within remote reference"
    // )]
    // #[test_case(
    //     j!({
    //         "$id": "http://localhost:1234/root",
    //         "properties": {
    //             "A": {
    //                 "$ref": "http://localhost:1234/root"
    //             }
    //         }
    //     }),
    //     &[
    //         j!({
    //             "$id": "http://localhost:1234/root",
    //             "properties": {
    //                 "A": {
    //                     "$ref": "http://localhost:1234/root"
    //                 }
    //             }
    //         }),
    //         j!({
    //             "A": {
    //                 "$ref": "http://localhost:1234/root"
    //             }
    //         }),
    //         j!({"$ref": "http://localhost:1234/root"}),
    //     ],
    //     &[
    //         raw(0, 1, KeywordName::Properties),
    //         raw(1, 2, "A"),
    //         raw(2, 0, KeywordName::Ref),
    //     ],
    //     &[];
    //     "Absolute reference to the same schema"
    // )]
    // #[test_case(
    //     j!({
    //         "allOf": [
    //             {"$ref": "#/allOf/1"},
    //             {"$ref": "#/allOf/0"}
    //         ]
    //     }),
    //     &[
    //         j!({"allOf":[{"$ref":"#/allOf/1"},{"$ref":"#/allOf/0"}]}),
    //         j!([{"$ref":"#/allOf/1"},{"$ref":"#/allOf/0"}]),
    //         j!({"$ref":"#/allOf/1"}),
    //         j!({"$ref":"#/allOf/0"}),
    //     ],
    //     &[
    //         raw(0, 1, KeywordName::AllOf),
    //         raw(1, 2, 0),
    //         raw(3, 2, KeywordName::Ref),
    //         raw(2, 3, KeywordName::Ref),
    //         raw(1, 3, 1),
    //     ],
    //     &[];
    //     "Multiple references to the same target"
    // )]
    // #[test_case(
    //     j!({
    //         "$id": "http://localhost:1234/tree",
    //         "properties": {
    //             "nodes": {
    //                 "items": {"$ref": "node"}
    //             }
    //         },
    //         "definitions": {
    //             "node": {
    //                 "$id": "http://localhost:1234/node",
    //                 "properties": {
    //                     "subtree": {"$ref": "tree"}
    //                 }
    //             }
    //         }
    //     }),
    //     &[
    //         j!({"$id":"http://localhost:1234/tree","definitions":{"node":{"$id":"http://localhost:1234/node","properties":{"subtree":{"$ref":"tree"}}}},"properties":{"nodes":{"items":{"$ref":"node"}}}}),
    //         j!({"nodes":{"items":{"$ref":"node"}}}),
    //         j!({"items":{"$ref":"node"}}),
    //         j!({"$ref":"node"}),
    //         j!({"$id":"http://localhost:1234/node","properties":{"subtree":{"$ref":"tree"}}}),
    //         j!({"subtree":{"$ref":"tree"}}),
    //         j!({"$ref":"tree"}),
    //     ],
    //     &[
    //         raw(0, 1, KeywordName::Properties),
    //         raw(1, 2, "nodes"),
    //         raw(2, 3, KeywordName::Items),
    //         raw(4, 5, KeywordName::Properties),
    //         raw(5, 6, "subtree"),
    //         raw(6, 0, KeywordName::Ref),
    //         raw(3, 4, KeywordName::Ref),
    //     ],
    //     &[];
    //     "Recursive references between schemas"
    // )]
    // fn values_and_edges(schema: Value, values: &[Value], edges: &[RawEdge], keys: &[&str]) {
    //     let root = resolving::Resolver::new(&schema).unwrap();
    //     let external = resolving::fetch_external(&schema, &root).unwrap();
    //     let resolvers = resolving::build_resolvers(&external);
    //     let (values_, keywords_of, edges_if) =
    //         collection::collect(&schema, &root, &resolvers).unwrap();
    //     testing::print_values(&values_);
    //     let concrete_values = values_.into_iter().cloned().collect::<Vec<Value>>();
    //     assert_eq!(concrete_values, values);
    //     // assert_eq!(edges_, edges);
    //     // testing::assert_unique_edges(&edges_);
    //     assert_eq!(resolvers.keys().cloned().collect::<Vec<&str>>(), keys);
    // }

    // #[test_case(
    //     vec![
    //         &j!({"maximum": 5}),
    //         &j!(5),
    //     ],
    //     vec![raw(0, 1, KeywordName::Maximum)],
    //     1,
    //     vec![Maximum::build(5)],
    //     vec![];
    //     "Validator keyword"
    // )]
    // #[test_case(
    //     vec![
    //         &j!({"properties": {"A": {"maximum": 5}}}),
    //         &j!({"A":{"maximum": 5}}),
    //         &j!({"maximum": 5}),
    //         &j!(5),
    //     ],
    //     vec![
    //         raw(0, 1, KeywordName::Properties),
    //         raw(1, 2, "A"),
    //         raw(2, 3, KeywordName::Maximum),
    //     ],
    //     1,
    //     vec![
    //         Properties::build(0, 1),
    //         Maximum::build(5),
    //     ],
    //     vec![
    //         edge("A", 1..2),
    //     ];
    //     "Applicator keyword - one edge"
    // )]
    // #[test_case(
    //     vec![
    //         &j!({"properties": {"A": {"maximum": 5}, "B": {"maximum": 3}}}),
    //         &j!({"A":{"maximum": 5}, "B": {"maximum": 3}}),
    //         &j!({"maximum": 5}),
    //         &j!(5),
    //         &j!({"maximum": 3}),
    //         &j!(3),
    //     ],
    //     vec![
    //         raw(0, 1, KeywordName::Properties),
    //         raw(1, 2, "A"),
    //         raw(2, 3, KeywordName::Maximum),
    //         raw(1, 4, "B"),
    //         raw(4, 5, KeywordName::Maximum),
    //     ],
    //     1,
    //     vec![
    //         Properties::build(0, 2),
    //         Maximum::build(5),
    //         Maximum::build(3),
    //     ],
    //     vec![
    //         edge("A", 1..2),
    //         edge("B", 2..3),
    //     ];
    //     "Applicator keyword - two edges"
    // )]
    // #[test_case(
    //     vec![
    //         &j!({"properties": {"A": {"properties": {"B": {"maximum": 5}}}}}),
    //         &j!({"A": {"properties": {"B": {"maximum": 5}}}}),
    //         &j!({"properties": {"B": {"maximum": 5}}}),
    //         &j!({"B": {"maximum": 5}}),
    //         &j!({"maximum": 5}),
    //         &j!(5),
    //     ],
    //     vec![
    //         raw(0, 1, KeywordName::Properties),
    //         raw(1, 2, "A"),
    //         raw(2, 3, KeywordName::Properties),
    //         raw(3, 4, "B"),
    //         raw(4, 5, KeywordName::Maximum),
    //     ],
    //     1,
    //     vec![
    //         Properties::build(0, 1),
    //         Properties::build(1, 2),
    //         Maximum::build(5),
    //     ],
    //     vec![
    //         edge("A", 1..2),
    //         edge("B", 2..3),
    //     ];
    //     "Applicator keyword - nested"
    // )]
    // fn building(
    //     values: Vec<&Value>,
    //     edges: Vec<RawEdge>,
    //     expected_head: usize,
    //     expected_keywords: Vec<Keyword>,
    //     expected_edges: Vec<Edge>,
    // ) {
    //     let (head, keywords, edges_) = build(&values, edges);
    //     assert_eq!(head, expected_head);
    //     assert_eq!(keywords, expected_keywords);
    //     assert_eq!(edges_, expected_edges);
    // }

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
                        let (_, _) = collection::collect(schema, &root, &resolvers).unwrap();
                        // testing::assert_unique_edges(&edges);
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

    // #[test_case(j!({"maximum": 5}), j!(4), j!(6))]
    // #[test_case(j!({"properties": {"A": {"maximum": 5}}}), j!({"A": 4}), j!({"A": 6}))]
    // #[test_case(j!({"properties": {"A": {"properties": {"B": {"maximum": 5}}}}}), j!({"A": {"B": 4}}), j!({"A": {"B": 6}}))]
    #[test_case(
        j!({"allOf": [{"properties": {"A": {"maximum": 5}}}, {"properties": {"B": {"maximum": 7}}}]}),
        j!({"A": 4, "B": 6}),
        j!({"A": 4, "B": 9})
    )]
    fn is_valid(schema: Value, valid: Value, invalid: Value) {
        let compiled = JsonSchema::new(&schema).unwrap();
        println!("{:?}", compiled);
        assert!(compiled.is_valid(&valid));
        assert!(!compiled.is_valid(&invalid));
    }
}
