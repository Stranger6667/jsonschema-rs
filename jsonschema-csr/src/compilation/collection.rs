use super::{
    super::vocabularies::KeywordName,
    edges::{EdgeLabel, RawEdge},
    error::Result,
    references::{self, Reference},
    resolving::{id_of_object, with_folders, Resolver},
    ValueReference,
};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

// TODO:
//   - Document all things
//   - Properly change scope

/// The main goal of this phase is to collect all nodes from the input schema and its remote
/// dependencies into a single graph where each vertex is a reference to a JSON value from these
/// schemas. Each edge is represented as a pair indexes into the vertex vector.
///
/// This representation format is efficient to construct the schema graph, but not for input
/// validation. Input requires arbitrary traversal order from the root node, because it depends on
/// the input value - certain schema branches are needed only for certain types or values.
/// For example, `properties` sub-schemas are needed only if the input contains matching keys.
pub(crate) fn collect<'s>(
    schema: &'s Value,
    root_resolver: &'s Resolver,
    resolvers: &'s HashMap<&str, Resolver>,
) -> Result<(Vec<ValueReference<'s>>, Vec<RawEdge>)> {
    Collector::new(schema, root_resolver, resolvers).collect()
}

// TODO. maybe it is easier to check the parent? I.e. if it is `properties`, then
/// Defines JSON Schema specific semantic for the currently traversed value.
enum ScopeKind {
    Default,
    /// The traversed object is a JSON Schema.
    /// All its keys should be treated as JSON Schema keywords.
    Schema,
}

struct CollectionScope<'s> {
    kind: ScopeKind,
    parent: Option<(usize, EdgeLabel)>,
    folders: Vec<&'s str>,
    resolver: &'s Resolver<'s>,
    node: &'s Value,
}

impl<'s> CollectionScope<'s> {
    pub(crate) fn track_folder(&mut self, object: &'s Map<String, Value>) {
        // Some objects may change `$ref` behavior via the `$id` keyword
        if let Some(id) = id_of_object(object) {
            self.folders.push(id);
        }
    }
}

struct CollectionStack<'s>(Vec<CollectionScope<'s>>);

impl<'s> CollectionStack<'s> {
    pub(crate) fn new(node: &'s Value, resolver: &'s Resolver<'s>) -> Self {
        Self(vec![CollectionScope {
            kind: ScopeKind::Schema,
            parent: None,
            folders: vec![],
            resolver,
            node,
        }])
    }
    pub(crate) fn pop(&mut self) -> Option<CollectionScope<'s>> {
        self.0.pop()
    }
    pub(crate) fn push(&mut self, scope: CollectionScope<'s>) {
        self.0.push(scope)
    }
}

/// Storage for intermediate collection data.
pub(crate) struct Collector<'s> {
    stack: CollectionStack<'s>,
    resolvers: &'s HashMap<&'s str, Resolver<'s>>,
    /// Nodes of the input schema.
    nodes: Vec<ValueReference<'s>>,
    /// Edges between graph nodes.
    edges: Vec<RawEdge>,
    /// Nodes already seen during collection.
    seen: HashSet<*const Value>,
}

impl<'s> Collector<'s> {
    /// Create a new collector.
    pub(crate) fn new(
        schema: &'s Value,
        root_resolver: &'s Resolver,
        resolvers: &'s HashMap<&str, Resolver>,
    ) -> Self {
        Self {
            stack: CollectionStack::new(schema, root_resolver),
            resolvers,
            nodes: vec![],
            edges: vec![],
            seen: HashSet::default(),
        }
    }

    fn add_concrete(&mut self, value: &'s Value) -> usize {
        let node_id = self.nodes.len();
        self.nodes.push(ValueReference::Concrete(value));
        node_id
    }
    fn add_virtual(&mut self, value: &'s Value) -> usize {
        let node_id = self.nodes.len();
        self.nodes.push(ValueReference::Virtual(value));
        node_id
    }
    fn add_edge(&mut self, source: usize, target: usize, label: impl Into<EdgeLabel>) {
        self.edges.push(RawEdge::new(source, target, label.into()));
    }

    #[inline]
    fn mark_seen(&mut self, value: &'s Value) -> bool {
        self.seen.insert(value as *const _)
    }

    fn push(
        &mut self,
        kind: ScopeKind,
        node: &'s Value,
        index: usize,
        label: impl Into<EdgeLabel>,
        folders: Vec<&'s str>,
        resolver: &'s Resolver<'s>,
    ) {
        if self.mark_seen(node) {
            self.stack.push(CollectionScope {
                kind,
                parent: Some((index, label.into())),
                folders,
                resolver,
                node,
            });
        }
    }

    pub(crate) fn collect(mut self) -> Result<(Vec<ValueReference<'s>>, Vec<RawEdge>)> {
        // Explore all nodes in the tree via a DFS traversal
        while let Some(mut scope) = self.stack.pop() {
            // Mark this node as seen to prevent re-traversing it later. As JSON Schema has
            // the `$ref` keyword, it is possible to reach this node from multiple nodes
            self.mark_seen(scope.node);
            let node_id = self.add_concrete(scope.node);
            // Traverse composite variants
            match scope.node {
                Value::Object(object) => {
                    match &scope.parent {
                        // Some keywords expect values to be schemas.
                        // For example, all values inside the `properties` keyword.
                        Some((_, EdgeLabel::Keyword(KeywordName::Properties))) => {}
                        // Parent is not a keyword - do not traverse it.
                        Some(_) => {}
                        // Root - it is a schema
                        None => self.collect_schema(object, node_id, &mut scope)?,
                    }
                }
                Value::Array(array) => self.collect_array(array, node_id, &scope),
                _ => {}
            };
            // Add an edge between the parent node and this one
            if let Some((parent_id, label)) = scope.parent {
                self.add_edge(parent_id, node_id, label);
            }
        }
        Ok((self.nodes, self.edges))
    }

    fn collect_schema(
        &mut self,
        object: &'s Map<String, Value>,
        parent_id: usize,
        scope: &mut CollectionScope<'s>,
    ) -> Result<()> {
        scope.track_folder(object);
        // Keyword - what is its value
        // - $ref - a schema
        // - properties - object where each value is a schema
        // - items - object/array where each value is a schema | bool
        // - anyOf - array where each value is a schema
        // - if - schema
        // - maximum - simple value
        // - enum - simple value
        for (key, value) in object {
            match key.as_str() {
                "$ref" => {
                    if let Value::String(reference) = value {
                        self.collect_reference(reference, parent_id, scope)?;
                    } else {
                        // The `$ref` value is not a string - explore it further
                        self.push(
                            ScopeKind::Schema,
                            value,
                            parent_id,
                            "$ref",
                            scope.folders.clone(),
                            scope.resolver,
                        );
                    };
                }
                "maximum" => {
                    self.push(
                        ScopeKind::Default,
                        value,
                        parent_id,
                        KeywordName::Maximum,
                        scope.folders.clone(),
                        scope.resolver,
                    );
                }
                "properties" => {
                    self.push(
                        ScopeKind::Default,
                        value,
                        parent_id,
                        KeywordName::Properties,
                        scope.folders.clone(),
                        scope.resolver,
                    );
                }
                unknown => {
                    println!("Unknown keyword: {}", unknown)
                }
            }
        }
        Ok(())
    }

    fn collect_object(
        &mut self,
        object: &'s Map<String, Value>,
        parent_id: usize,
        scope: &mut CollectionScope<'s>,
    ) {
        for (key, value) in object {
            self.push(
                ScopeKind::Schema,
                value,
                parent_id,
                key,
                scope.folders.clone(),
                scope.resolver,
            );
        }
    }

    /// Collect JSON array values.
    fn collect_array(&mut self, array: &'s [Value], parent_id: usize, scope: &CollectionScope<'s>) {
        for (id, item) in array.iter().enumerate() {
            self.push(
                ScopeKind::Default,
                item,
                parent_id,
                id,
                scope.folders.clone(),
                scope.resolver,
            );
        }
    }

    fn collect_reference(
        &mut self,
        reference: &str,
        node_id: usize,
        scope: &mut CollectionScope<'s>,
    ) -> Result<()> {
        let next_id = match Reference::try_from(reference)? {
            Reference::Absolute(location) => {
                let resolved = if let Some(resolver) = self.resolvers.get(location.as_str()) {
                    let (folders, resolved) = resolver.resolve(reference)?;
                    self.push(
                        ScopeKind::Schema,
                        resolved,
                        node_id,
                        KeywordName::Ref,
                        folders.clone(),
                        resolver,
                    );
                    resolved
                } else {
                    let (_, resolved) = scope.resolver.resolve(reference)?;
                    resolved
                };
                self.add_virtual(resolved)
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
                self.push(
                    ScopeKind::Schema,
                    resolved,
                    node_id,
                    KeywordName::Ref,
                    folders.clone(),
                    resolver,
                );
                self.add_virtual(resolved)
            }
        };
        self.add_edge(node_id, next_id, KeywordName::Ref);
        Ok(())
    }
}
