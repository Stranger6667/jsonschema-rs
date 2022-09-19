use super::{
    super::vocabularies::KeywordName,
    edges::{EdgeLabel, RawEdge},
    error::Result,
    references::{self, Reference},
    resolving::{id_of_object, with_folders, Resolver},
};
use serde_json::{Map, Value};
use std::collections::{hash_map::Entry, HashMap};
use KeywordName::{AllOf, Items, Maximum, Properties, Ref, Type};

// TODO:
//   - Document all things

/// The main goal of this phase is to collect all nodes from the input schema and its remote
/// dependencies into a single graph where each vertex is a reference to a JSON value from these
/// schemas. Each edge is represented as a pair indexes into the vertex vector and a label.
pub(crate) fn collect<'s>(
    schema: &'s Value,
    root_resolver: &'s Resolver,
    resolvers: &'s HashMap<&str, Resolver>,
) -> Result<(Vec<&'s Value>, Vec<RawEdge>)> {
    Collector::new(resolvers).collect(schema, root_resolver)
}

struct CollectionScope<'s> {
    folders: Vec<&'s str>,
    resolver: &'s Resolver<'s>,
}

impl<'s> CollectionScope<'s> {
    pub(crate) fn new(resolver: &'s Resolver) -> Self {
        Self::with_folders(resolver, vec![])
    }
    pub(crate) fn with_folders(resolver: &'s Resolver, folders: Vec<&'s str>) -> Self {
        Self { folders, resolver }
    }
    pub(crate) fn track_folder(&mut self, object: &'s Map<String, Value>) {
        // Some objects may change `$ref` behavior via the `$id` keyword
        if let Some(id) = id_of_object(object) {
            self.folders.push(id);
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum ValueEntry {
    Occupied,
    Vacant,
}
use ValueEntry::{Occupied, Vacant};

/// Storage for intermediate collection data.
pub(crate) struct Collector<'s> {
    resolvers: &'s HashMap<&'s str, Resolver<'s>>,
    /// Nodes of the input schema.
    nodes: Vec<&'s Value>,
    /// Edges between graph nodes.
    edges: Vec<RawEdge>,
    /// Nodes already seen during collection.
    seen: HashMap<*const Value, usize>,
}

// Validation process
//   - Iterate over head layer
// Head layer should be sorted, so it always includes keywords from the top schema
// Goal: use ranges to refer to child nodes, otherwise it will require a vector to store indexes
//
// Keywords
//  [
//    properties (0..2)
//    type
//    -- 0..2
//     * A - type
//     * A - maximum
//       -- 2..4
//     * B - type
//     * B - properties (2..3)
//       -- 4..6
//     * B1 - type
//     * B1 - maxLength
//       -- 6..8
//  ]
//
// Edges:
//  [
//    ("A", 2..4),
//    ("B", 4..6),
//    ("B1", 6..8),
//  ]
//
// 1. access the head layer
// 2. properties should know how to get to "A - type" & "A - maximum". Could be either
// Is it possible to build such a graph on the go? without sorting later
//  - For each applicator you need to know label + keyword range for each child. Can't guess keyword range upfront
//  - Maybe it could work with intermediate vectors, that then are appended to the main one.
//    For each applicator create a vec & put all keywords there there.
// How to build such a layout:
//   -

impl<'s> Collector<'s> {
    /// Create a new collector.
    pub(crate) fn new(resolvers: &'s HashMap<&str, Resolver>) -> Self {
        Self {
            resolvers,
            nodes: vec![],
            edges: vec![],
            seen: HashMap::default(),
        }
    }

    /// Push a value to the tree.
    /// If value already exists there - return its index.
    fn push(&mut self, value: &'s Value) -> (ValueEntry, usize) {
        match self.seen.entry(value as *const _) {
            Entry::Occupied(entry) => (Occupied, *entry.get()),
            Entry::Vacant(entry) => {
                let node_id = self.nodes.len();
                self.nodes.push(value);
                entry.insert(node_id);
                (Vacant, node_id)
            }
        }
    }
    fn add_edge(&mut self, source: usize, target: usize, label: impl Into<EdgeLabel>) {
        self.edges.push(RawEdge::new(source, target, label.into()));
    }
    fn add_value(
        &mut self,
        source: usize,
        value: &'s Value,
        label: impl Into<EdgeLabel>,
    ) -> (ValueEntry, usize) {
        let (entry, target_id) = self.push(value);
        self.add_edge(source, target_id, label);
        (entry, target_id)
    }

    pub(crate) fn collect(
        mut self,
        node: &'s Value,
        resolver: &'s Resolver,
    ) -> Result<(Vec<&'s Value>, Vec<RawEdge>)> {
        let mut scope = CollectionScope::new(resolver);
        let (_, node_id) = self.push(node);
        self.collect_schema(node, node_id, &mut scope)?;
        Ok((self.nodes, self.edges))
    }

    fn collect_schema(
        &mut self,
        schema: &'s Value,
        parent_id: usize,
        scope: &mut CollectionScope<'s>,
    ) -> Result<()> {
        if let Value::Object(object) = schema {
            scope.track_folder(object);
            for (key, value) in object {
                match key.as_str() {
                    "$ref" => {
                        if let Value::String(reference) = value {
                            self.collect_reference(reference, parent_id, scope)?;
                        }
                    }
                    "maximum" => {
                        self.add_value(parent_id, value, Maximum);
                    }
                    "type" => {
                        self.add_value(parent_id, value, Type);
                    }
                    "allOf" => {
                        if let (Vacant, source) = self.add_value(parent_id, value, AllOf) {
                            if let Value::Array(items) = value {
                                for (id, schema) in items.iter().enumerate() {
                                    if let (Vacant, id) = self.add_value(source, schema, id) {
                                        self.collect_schema(schema, id, scope)?;
                                    }
                                }
                            }
                        }
                    }
                    "items" => {
                        if let (Vacant, id) = self.add_value(parent_id, value, Items) {
                            self.collect_schema(value, id, scope)?;
                        }
                    }
                    "properties" => {
                        if let (Vacant, source) = self.add_value(parent_id, value, Properties) {
                            if let Value::Object(object) = value {
                                for (key, schema) in object {
                                    if let (Vacant, id) = self.add_value(source, schema, key) {
                                        self.collect_schema(schema, id, scope)?;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                };
            }
        }
        Ok(())
    }

    fn collect_reference(
        &mut self,
        reference: &str,
        source: usize,
        scope: &mut CollectionScope<'s>,
    ) -> Result<()> {
        // Resolve reference & traverse it.
        let target = match Reference::try_from(reference)? {
            Reference::Absolute(location) => {
                if let Some(resolver) = self.resolvers.get(location.as_str()) {
                    let (folders, resolved) = resolver.resolve(reference)?;
                    let (_, target) = self.push(resolved);
                    let mut scope = CollectionScope::with_folders(resolver, folders);
                    self.collect_schema(resolved, target, &mut scope)?;
                    target
                } else {
                    let (_, resolved) = scope.resolver.resolve(reference)?;
                    self.push(resolved).1
                }
            }
            Reference::Relative(location) => {
                let mut resolver = scope.resolver;
                if !references::is_local(location) {
                    let location = with_folders(resolver.scope(), location, &scope.folders)?;
                    if !resolver.contains(location.as_str()) {
                        resolver = self
                            .resolvers
                            .get(location.as_str())
                            .expect("Unknown reference");
                    }
                };
                let (folders, resolved) = resolver.resolve(location)?;
                // Push the resolved value onto the stack to explore them further
                match self.push(resolved) {
                    (Vacant, target) => {
                        let mut scope = CollectionScope::with_folders(resolver, folders);
                        self.collect_schema(resolved, target, &mut scope)?;
                        target
                    }
                    (Occupied, target) => target,
                }
            }
        };
        self.add_edge(source, target, Ref);
        Ok(())
    }
}
