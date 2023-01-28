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
mod error;
pub(crate) mod graph;
pub mod resolving;

use crate::{schema::graph::MultiEdge, vocabularies::Keyword};
use error::Result;
use serde_json::Value;

// TODO. Optimization ideas:
//   - Values ordering. "Cheaper" keywords might work better if they are executed first.
//     Example: `anyOf` where some items are very cheap and common to pass validation.
//     collect average distance between two subsequent array accesses to measure it
//   - Interleave values & edges in the same struct. Might improve cache locality.
//   - Order keywords, so ones with edges are stored in the end of the current level -
//     this way there will be fewer jump to other levels and back to the current one

#[derive(Debug)]
pub struct Schema {
    graph: graph::CompressedRangeGraph,
}

impl Schema {
    pub fn new(schema: &Value) -> Result<Self> {
        // Resolver for the root schema
        // needed to resolve location-independent references during the initial resolving step
        let (root, external) = resolving::resolve(schema)?;
        // Build resolvers for external schemas
        let resolvers = resolving::build_resolvers(&external);
        // Collect all values and resolve references
        Ok(Schema {
            graph: graph::build(schema, &root, &resolvers)?,
        })
    }

    pub fn is_valid(&self, instance: &Value) -> bool {
        todo!()
        // self.keywords[..self.root_offset]
        //     .iter()
        //     .all(|keyword| keyword.is_valid(self, instance))
    }

    pub(crate) fn nodes(&self) -> &[Keyword] {
        &self.graph.nodes
    }
    pub(crate) fn edges(&self) -> &[MultiEdge] {
        &self.graph.edges
    }

    // pub fn validate<'s, 'i>(&'s self, instance: &'i Value) -> ValidationResult<'s, 'i> {
    //     ValidationResult {
    //         schema: self,
    //         instance,
    //     }
    // }
}
//
// #[derive(Clone)]
// pub struct ValidationResult<'s, 'i> {
//     schema: &'s Schema,
//     instance: &'i Value,
// }
//
// impl<'s, 'i> ValidationResult<'s, 'i> {
//     pub fn errors(&self) -> ErrorIterator {
//         ErrorIterator::new(self.schema, self.instance)
//     }
// }
//
// pub struct ErrorIterator<'s, 'i> {
//     keywords: Vec<Range<usize>>,
//     edges: Vec<Range<usize>>,
//     schema: &'s Schema,
//     instance: &'i Value,
// }
//
// impl<'s, 'i> ErrorIterator<'s, 'i> {
//     fn new(schema: &'s Schema, instance: &'i Value) -> Self {
//         Self {
//             keywords: vec![0..schema.root_offset],
//             edges: vec![],
//             schema,
//             instance,
//         }
//     }
// }
//
// impl<'s, 'i> Iterator for ErrorIterator<'s, 'i> {
//     type Item = u64;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         while let Some(Range { mut start, end }) = self.keywords.pop() {
//             for keyword in &self.schema.keywords[start..end] {
//                 // FIXME: applicators should somehow collect multiple children results, decide
//                 //        and bubble up the errors only in this case.
//                 //        Maybe create `Error` iterator for children & call recursively?
//                 //        In such a case it will be nice to avoid creating new `Vec` there &
//                 //        reuse this one
//                 //        E.g. applicators could get an iterator over children errors as input
//                 //        Maybe pass &mut Vec to `ErrorIterator`?? or just have a private struct
//                 //        that implements the same stuff. This way `ErrorIterator` will have only
//                 //        2 lifetimes
//                 start += 1;
//                 let result = if let Some(edges) = keyword.edges() {
//                     for edge in &self.schema.edges[edges] {
//                         self.keywords.push(edge.keywords.clone());
//                     }
//                     continue;
//                 } else {
//                     // TODO: Validation keywords actually don't need schema - try to not pass it
//                     keyword.validate(self.schema, self.instance)
//                 };
//                 // FIXME: It doesn't cover the `continue` above
//                 if start != end {
//                     // Store not yet traversed keywords to get back to them later
//                     self.keywords.push(start..end);
//                 }
//                 return result;
//             }
//         }
//         None
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::load_case;
    use test_case::test_case;

    #[test_case("maximum")]
    #[test_case("properties")]
    #[test_case("nested-properties")]
    #[test_case("multiple-nodes-each-layer")]
    #[test_case("ref-recursive-absolute")]
    fn is_valid(name: &str) {
        let case = load_case(name);
        let compiled = Schema::new(&case["schema"]).unwrap();
        assert!(compiled.is_valid(&case["valid"]));
        assert!(!compiled.is_valid(&case["invalid"]));
    }
}
