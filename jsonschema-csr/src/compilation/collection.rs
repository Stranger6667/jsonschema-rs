use super::edges::RawEdge;
use super::error::Result;
use super::resolver::is_local_reference;
use super::resolver::with_folders;
use super::resolver::Reference;
use super::resolver::Resolver;
use super::resolver::{id_of_object, parse_reference};
use super::ValueReference;
use crate::compilation::edges::EdgeLabel;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

pub(crate) fn collect<'s>(
    schema: &'s Value,
    root_resolver: &'s Resolver,
    resolvers: &'s HashMap<&str, Resolver>,
) -> Result<(Vec<ValueReference<'s>>, Vec<RawEdge>)> {
    Collector::new().collect(schema, root_resolver, resolvers)
}

enum ScopeKind {
    Default,
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
    pub(crate) fn push_schema(&mut self) {}
}

/// Storage for intermediate collection data.
pub(crate) struct Collector<'s> {
    /// Nodes of the input schema.
    nodes: Vec<ValueReference<'s>>,
    /// Edges between graph nodes.
    edges: Vec<RawEdge>,
    /// Nodes already seen during collection.
    seen: HashSet<*const Value>,
}

impl<'s> Collector<'s> {
    /// Create a new collector.
    pub(crate) fn new() -> Self {
        Self {
            nodes: vec![],
            edges: vec![],
            seen: HashSet::default(),
        }
    }

    fn add_concrete(&mut self, value: &'s Value) {
        self.nodes.push(ValueReference::Concrete(value))
    }
    fn add_virtual(&mut self, value: &'s Value) {
        self.nodes.push(ValueReference::Virtual(value))
    }
    fn add_edge(&mut self, source: usize, target: usize, label: impl Into<EdgeLabel>) {
        self.edges.push(RawEdge::new(source, target, label.into()));
    }

    #[inline]
    fn mark_seen(&mut self, value: &'s Value) -> bool {
        self.seen.insert(value as *const _)
    }

    #[inline]
    fn current_node_id(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn collect(
        mut self,
        schema: &'s Value,
        root_resolver: &'s Resolver,
        resolvers: &'s HashMap<&str, Resolver>,
    ) -> Result<(Vec<ValueReference<'s>>, Vec<RawEdge>)> {
        let mut stack = CollectionStack::new(schema, root_resolver);
        while let Some(mut scope) = stack.pop() {
            let mut resolver = scope.resolver;
            let node_idx = self.current_node_id();
            // Mark this value as seen to prevent re-traversing it if any reference leads to it
            self.mark_seen(scope.node);
            self.add_concrete(scope.node);
            match scope.node {
                Value::Object(object) => {
                    scope.track_folder(object);
                    for (key, value) in object {
                        match key.as_str() {
                            "$ref" => {
                                if let Value::String(ref_string) = value {
                                    match parse_reference(ref_string)? {
                                        Reference::Absolute(location) => {
                                            let resolved = if let Some(resolver) =
                                                resolvers.get(location.as_str())
                                            {
                                                let (folders, resolved) =
                                                    resolver.resolve(ref_string)?;
                                                // push_schema!(
                                                //     stack, node_idx, resolved, REF, seen, resolver,
                                                //     folders
                                                // );
                                                resolved
                                            } else {
                                                let (_, resolved) = resolver.resolve(ref_string)?;
                                                resolved
                                            };
                                            self.add_virtual(resolved);
                                        }
                                        Reference::Relative(location) => {
                                            if !is_local_reference(location) {
                                                let location = with_folders(
                                                    resolver.scope(),
                                                    location,
                                                    &scope.folders,
                                                )?;
                                                if !resolver.contains(location.as_str()) {
                                                    resolver = resolvers
                                                        .get(location.as_str())
                                                        .expect("Unknown reference");
                                                }
                                            };
                                            let (folders, resolved) = resolver.resolve(location)?;
                                            self.add_virtual(resolved);
                                            // Push the resolved value onto the stack to explore them further
                                            // push_schema!(
                                            //     stack, node_idx, resolved, REF, seen, resolver,
                                            //     folders
                                            // );
                                        }
                                    };
                                    self.add_edge(node_idx, self.nodes.len() - 1, "$ref");
                                } else {
                                    // The `$ref` value is not a string - explore it further
                                    // push_schema!(
                                    //     stack, node_idx, value, REF, seen, resolver, folders
                                    // );
                                };
                            }
                            "maximum" => {
                                // push_not_schema!(
                                //     stack,
                                //     node_idx,
                                //     value,
                                //     KeywordName::Maximum,
                                //     seen,
                                //     resolver,
                                //     folders
                                // );
                            }
                            "properties" => {
                                // push_not_schema!(
                                //     stack,
                                //     node_idx,
                                //     value,
                                //     KeywordName::Properties,
                                //     seen,
                                //     resolver,
                                //     folders
                                // );
                            }
                            unknown => {
                                println!("Unknown keyword: {}", unknown)
                            }
                        }
                    }
                }
                Value::Array(items) => {
                    for (idx, item) in items.iter().enumerate() {
                        // push_schema!(stack, node_idx, item, idx, seen, resolver, folders);
                    }
                }
                _ => {}
            };
            if let Some((parent_idx, label)) = scope.parent {
                self.add_edge(parent_idx, node_idx, label);
            }
        }
        Ok((self.nodes, self.edges))
    }
}
