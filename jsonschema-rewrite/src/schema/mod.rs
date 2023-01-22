//! This module contains data structures for representing JSON Schemas.
//!
//! # Motivation
//!
//! The main motivation behind all design decisions is high data validation performance.
//!
//! The most effective way to achieve it is to minimize the amount of work needed during validation.
//! It is achieved by:
//!
//! - Loading remote schemas only once
//! - Inlining trivial schemas
//! - Skipping sub-schemas that always succeed (e.g. `uniqueItems: false`)
//! - Grouping multiple sub-schemas into a single one
//! - Specializing internal data structures and algorithms by the input schema characteristics
//! - Avoiding allocations during validation
//!
//! # Design
//!
//! JSON Schema is a directed graph, where cycles could be represented via the `$ref` keyword.
//! It contains two types of edges:
//!   - Regular parent-child relationships between two JSON values;
//!   - Values connected via the `$ref` keyword.
//!
//! This graph is implemented as an arena that stores all nodes in a single vector with a separate
//! vector for edges.
//!
//! ## Example
//!
//! Schema:
//!
//! ```json
//! {
//!     "type": "object",
//!     "properties": {
//!         "A": {
//!             "type": "string",
//!             "maxLength": 5
//!         },
//!         "B": {
//!             "allOf": [
//!                 {
//!                     "type": "integer"
//!                 },
//!                 {
//!                     "type": "array"
//!                 }
//!             ]
//!         },
//!     },
//!     "minProperties": 1
//! }
//! ```
//!
//!   Keywords                                                    Edges
//!
//! [                                                                 [
//! -- 0..3 `/`                                    |------------>     -- 0..2 (`properties' edges)
//!      <type: object>                            |  |<------------------ A
//!      <properties> -----> 0..2 ---------------->|  |  |<--------------- B
//!      <minProperties: 1>                           |  |  |--->     -- 2..4 (`allOf` edges)
//! -- 3..5 `/properties/A`               <--- 3..5 <-|  |  |  |<--------- 0
//!      <type: string>                                  |  |  |  |<------ 1
//!      <maxLength: 5>                                  |  |  |  |   ]
//! -- 5..6 `/properties/B`               <--- 5..6 <----|  |  |  |
//!      <allOf> ----------> 2..4 ------------------------->|  |  |
//! -- 6..7 `/properties/B/allOf/0`       <--- 6..7 <----------|  |
//!      <type: integer>                                          |
//! -- 7..8 `/properties/B/allOf/1`       <--- 7..8 <-------------|
//!      <type: array>
//! ]
//!
//! The key here is that nodes are stored the same way as how Breadth-First-Search traverses a tree.
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
    value_type::ValueType,
    vocabularies::{applicator, validation, Keyword},
};
use collection::{EdgeMap, KeywordMap};
use edges::MultiEdge;
use error::Result;
use serde_json::Value;
use std::ops::Range;

// TODO. Optimization ideas:
//   - Values ordering. "Cheaper" keywords might work better if they are executed first.
//     Example: `anyOf` where some items are very cheap and common to pass validation.
//     collect average distance between two subsequent array accesses to measure it
//   - Interleave values & edges in the same struct. Might improve cache locality.
//   - Order keywords, so ones with edges are stored in the end of the current level -
//     this way there will be fewer jump to other levels and back to the current one

#[derive(Debug)]
pub struct Schema {
    pub(crate) keywords: Box<[Keyword]>,
    head_end: usize,
    pub(crate) edges: Box<[MultiEdge]>,
}

impl Schema {
    pub fn new(schema: &Value) -> Result<Self> {
        // Resolver for the root schema
        // needed to resolve location-independent references during the initial resolving step
        let root_resolver = resolving::Resolver::new(schema)?;
        // Fetch all external schemas reachable from the root
        let external = resolving::fetch_external(schema, &root_resolver)?;
        // Build resolvers for external schemas
        let resolvers = resolving::build_resolvers(&external);
        // Collect all values and resolve references
        let (keyword_map, edge_map) = collection::collect(schema, &root_resolver, &resolvers)?;
        // Build a `Keyword` graph
        let (head_end, keywords, edges) = build(&keyword_map, &edge_map);
        Ok(Schema {
            keywords: keywords.into_boxed_slice(),
            head_end,
            edges: edges.into_boxed_slice(),
        })
    }

    pub fn is_valid(&self, instance: &Value) -> bool {
        self.keywords[..self.head_end]
            .iter()
            .all(|keyword| keyword.is_valid(self, instance))
    }

    pub fn validate<'s, 'i>(&'s self, instance: &'i Value) -> ValidationResult<'s, 'i> {
        ValidationResult {
            schema: self,
            instance,
        }
    }
}

#[derive(Clone)]
pub struct ValidationResult<'s, 'i> {
    schema: &'s Schema,
    instance: &'i Value,
}

impl<'s, 'i> ValidationResult<'s, 'i> {
    pub fn errors(&self) -> ErrorIterator {
        ErrorIterator::new(self.schema, self.instance)
    }
}

pub struct ErrorIterator<'s, 'i> {
    keywords: Vec<Range<usize>>,
    edges: Vec<Range<usize>>,
    schema: &'s Schema,
    instance: &'i Value,
}

impl<'s, 'i> ErrorIterator<'s, 'i> {
    fn new(schema: &'s Schema, instance: &'i Value) -> Self {
        Self {
            keywords: vec![0..schema.head_end],
            edges: vec![],
            schema,
            instance,
        }
    }
}

impl<'s, 'i> Iterator for ErrorIterator<'s, 'i> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(Range { mut start, end }) = self.keywords.pop() {
            for keyword in &self.schema.keywords[start..end] {
                // FIXME: applicators should somehow collect multiple children results, decide
                //        and bubble up the errors only in this case.
                //        Maybe create `Error` iterator for children & call recursively?
                //        In such a case it will be nice to avoid creating new `Vec` there &
                //        reuse this one
                //        E.g. applicators could get an iterator over children errors as input
                //        Maybe pass &mut Vec to `ErrorIterator`?? or just have a private struct
                //        that implements the same stuff. This way `ErrorIterator` will have only
                //        2 lifetimes
                start += 1;
                let result = if let Some(edges) = keyword.edges() {
                    for edge in &self.schema.edges[edges] {
                        self.keywords.push(edge.keywords.clone());
                    }
                    continue;
                } else {
                    // TODO: Validation keywords actually don't need schema - try to not pass it
                    keyword.validate(self.schema, self.instance)
                };
                // FIXME: It doesn't cover the `continue` above
                if start != end {
                    // Store not yet traversed keywords to get back to them later
                    self.keywords.push(start..end);
                }
                return result;
            }
        }
        None
    }
}

fn build(
    keyword_map: &KeywordMap<'_>,
    edge_map: &EdgeMap,
) -> (usize, Vec<Keyword>, Vec<MultiEdge>) {
    let mut keywords = vec![];
    let mut edges = vec![];
    let head = keyword_map.get(&0).map_or(0, |kwords| kwords.len());
    for node_keywords in keyword_map.values() {
        let mut next = keywords.len() + node_keywords.len();
        for (target, value, keyword) in node_keywords {
            match *keyword {
                "allOf" => {
                    keywords.push(applicator::AllOf::build(
                        edges.len(),
                        edges.len() + edge_map[target].len(),
                    ));
                    for edge in &edge_map[target] {
                        let end = next + keyword_map[&edge.target].len();
                        edges.push(edges::multi(edge.label.clone(), next..end));
                        next = end;
                    }
                }
                "items" => {}
                "maximum" => keywords.push(validation::Maximum::build(value.as_u64().unwrap())),
                "maxLength" => keywords.push(validation::MaxLength::build(value.as_u64().unwrap())),
                "minProperties" => {
                    keywords.push(validation::MinProperties::build(value.as_u64().unwrap()))
                }
                "properties" => {
                    keywords.push(applicator::Properties::build(
                        edges.len(),
                        edges.len() + edge_map[target].len(),
                    ));
                    // TODO: It will not work for $ref - it will point to some other keywords
                    for edge in &edge_map[target] {
                        let end = next + keyword_map[&edge.target].len();
                        edges.push(edges::multi(edge.label.clone(), next..end));
                        next = end;
                    }
                }
                "$ref" => {}
                "type" => {
                    let x = match value.as_str().unwrap() {
                        "array" => ValueType::Array,
                        "boolean" => ValueType::Boolean,
                        "integer" => ValueType::Integer,
                        "null" => ValueType::Null,
                        "number" => ValueType::Number,
                        "object" => ValueType::Object,
                        "string" => ValueType::String,
                        _ => panic!("invalid type"),
                    };
                    keywords.push(validation::Type::build(x))
                }
                _ => {}
            }
        }
    }
    (head, keywords, edges)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_reader, json as j, Value};
    use std::{fs, fs::File, io::BufReader};
    use test_case::test_case;

    pub fn read_json(filepath: &str) -> Value {
        let file = File::open(filepath).expect("Failed to open file");
        let reader = BufReader::new(file);
        from_reader(reader).expect("Invalid JSON")
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
    //     &[single(0, 1, KeywordName::Maximum)],
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
    //         single(0, 1, KeywordName::Properties),
    //         single(1, 2, "maximum"),
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
    //         single(0, 1, KeywordName::Properties),
    //         single(1, 2, "$ref"),
    //         single(2, 3, KeywordName::Maximum),
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
    //         single(0, 0, KeywordName::Ref),
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
    //         single(1, 2, KeywordName::Type),
    //         single(0, 1, KeywordName::Ref),
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
    //         single(0, 1, KeywordName::Items),
    //         single(1, 2, KeywordName::Items),
    //         single(3, 4, KeywordName::Type),
    //         single(2, 3, KeywordName::Ref),
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
    //         single(0, 1, KeywordName::Properties),
    //         single(1, 2, "list"),
    //         single(2, 3, KeywordName::Ref),
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
    //         single(2, 3, KeywordName::Type),
    //         single(1, 2, KeywordName::Ref),
    //         single(0, 1, KeywordName::Ref),
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
    //         single(0, 1, KeywordName::Properties),
    //         single(1, 2, "A"),
    //         single(2, 0, KeywordName::Ref),
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
    //         single(0, 1, KeywordName::AllOf),
    //         single(1, 2, 0),
    //         single(3, 2, KeywordName::Ref),
    //         single(2, 3, KeywordName::Ref),
    //         single(1, 3, 1),
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
    //         single(0, 1, KeywordName::Properties),
    //         single(1, 2, "nodes"),
    //         single(2, 3, KeywordName::Items),
    //         single(4, 5, KeywordName::Properties),
    //         single(5, 6, "subtree"),
    //         single(6, 0, KeywordName::Ref),
    //         single(3, 4, KeywordName::Ref),
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
    //     vec![single(0, 1, KeywordName::Maximum)],
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
    //         single(0, 1, KeywordName::Properties),
    //         single(1, 2, "A"),
    //         single(2, 3, KeywordName::Maximum),
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
    //         single(0, 1, KeywordName::Properties),
    //         single(1, 2, "A"),
    //         single(2, 3, KeywordName::Maximum),
    //         single(1, 4, "B"),
    //         single(4, 5, KeywordName::Maximum),
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
    //         single(0, 1, KeywordName::Properties),
    //         single(1, 2, "A"),
    //         single(2, 3, KeywordName::Properties),
    //         single(3, 4, "B"),
    //         single(4, 5, KeywordName::Maximum),
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
                        let (_, edge_map) = collection::collect(schema, &root, &resolvers).unwrap();
                        for (_, edges) in edge_map {
                            testing::assert_unique_edges(&edges);
                        }
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
    #[test_case(
        j!({
            "properties": {
                "A": {"maximum": 5}
            }
        }),
        j!({"A": 4}),
        j!({"A": 6})
    )]
    #[test_case(
        j!({
            "properties": {
                "A": {
                    "properties": {
                        "B": {"maximum": 5}
                    }
                }
            }
        }),
        j!({"A": {"B": 4}}),
        j!({"A": {"B": 6}})
    )]
    #[test_case(
        j!({
            "type": "object",
            "properties": {
                "A": {
                    "type": "string",
                    "maxLength": 5
                },
                "B": {
                    "allOf": [
                        {"type": "integer"},
                        {"type": "array"}
                    ]
                },
            },
            "minProperties": 1
        }),
        j!({"A": "ABC"}),
        j!({"A": 4, "B": 9})
    )]
    #[test_case(
        j!({
            "type": "object",
            "properties": {
                "A": {
                    "type": "string",
                    "maxLength": 5
                },
            },
            "allOf": [
                {
                    "type": "object"
                },
                {
                    "type": "object"
                }
            ]
        }),
        j!({"A": "ABC"}),
        j!({"A": 4})
    )]
    fn is_valid(schema: Value, valid: Value, invalid: Value) {
        let compiled = Schema::new(&schema).unwrap();
        testing::assert_graph(&compiled.keywords, &compiled.edges);
        assert!(compiled.is_valid(&valid));
        assert!(!compiled.is_valid(&invalid));
    }

    #[test]
    fn validate() {
        let compiled = Schema::new(&j!({
            "type": "object",
            "properties": {
                "A": {
                    "type": "string",
                    "maxLength": 5
                },
                "B": {
                    "allOf": [
                        {"type": "integer"},
                        {"type": "array"}
                    ]
                },
            },
            "minProperties": 1
        }))
        .unwrap();
        let instance = j!({"A": 4});
        let outcome = compiled.validate(&instance);
        for error in outcome.errors() {
            dbg!(error);
        }
    }
}
