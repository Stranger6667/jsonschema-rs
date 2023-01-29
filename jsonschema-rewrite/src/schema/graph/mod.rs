use crate::{
    schema::{
        error::Result,
        resolving::{id_of_object, is_local, Reference, Resolver},
    },
    value_type::ValueType,
    vocabularies::{
        applicator::{AllOf, Items, Properties},
        references::Ref,
        validation::{MaxLength, Maximum, MinProperties, Type},
        Keyword,
    },
};
mod edges;
mod nodes;

pub(crate) use edges::{Edge, EdgeLabel, RangedEdge};
pub(crate) use nodes::{NodeId, NodeSlot};
use serde_json::{Map, Value};
use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    ops::Range,
};
use url::Url;

pub(crate) type VisitedMap = HashMap<*const Value, NodeId>;

/// Build a packed graph to represent JSON Schema.
pub(crate) fn build<'s>(
    schema: &'s Value,
    root: &'s Resolver,
    resolvers: &'s HashMap<&str, Resolver>,
) -> Result<CompressedRangeGraph> {
    // Convert `Value` to an adjacency list and add all remote nodes reachable from the root
    let adjacency_list = AdjacencyList::new(schema, root, resolvers)?;
    // Each JSON Schema is a set of keywords that may contain nested sub-schemas. As all of nodes
    // are ordered by the BFS traversal order, we can address each schema by a range of indexes:
    //   * Create nodes with the same structure as the adjacency list but put corresponding
    //     `Some(Keyword)` instances at places containing valid JSON Schema keywords and fill
    //     everything else with `None`.
    //   * Convert edges, so they point to ranges of nodes
    let range_graph = RangeGraph::new(&adjacency_list)?;
    // Remove empty nodes and adjust all indexes
    Ok(range_graph.compress())
}

#[derive(Debug)]
pub(crate) struct AdjacencyList<'s> {
    pub(crate) nodes: Vec<&'s Value>,
    pub(crate) edges: Vec<Vec<Edge>>,
    visited: VisitedMap,
}

impl<'s> AdjacencyList<'s> {
    fn new(
        schema: &'s Value,
        root: &'s Resolver,
        resolvers: &'s HashMap<&str, Resolver>,
    ) -> Result<Self> {
        let mut output = AdjacencyList::empty();
        // This is a Breadth-First-Search routine
        let mut queue = VecDeque::new();
        queue.push_back((
            Scope::new(root),
            NodeId::new(0),
            EdgeLabel::Index(0),
            schema,
        ));
        while let Some((mut scope, parent_id, label, node)) = queue.pop_front() {
            let slot = output.push(parent_id, label, node);
            if slot.is_new() {
                match node {
                    Value::Object(object) => {
                        scope.track_folder(object);
                        // FIXME: track schema / non schema properly. Maybe extend scope?
                        for (key, value) in object {
                            if key == "$ref" {
                                if let Value::String(reference) = value {
                                    match Reference::try_from(reference.as_str())? {
                                        Reference::Absolute(location) => {
                                            if let Some(resolver) = resolvers.get(location.as_str())
                                            {
                                                let (folders, resolved) =
                                                    resolver.resolve(reference)?;
                                                queue.push_back((
                                                    Scope::with_folders(resolver, folders),
                                                    slot.id,
                                                    key.into(),
                                                    resolved,
                                                ));
                                            } else {
                                                let (_, resolved) =
                                                    scope.resolver.resolve(reference)?;
                                                queue.push_back((
                                                    scope.clone(),
                                                    slot.id,
                                                    key.into(),
                                                    resolved,
                                                ));
                                            }
                                        }
                                        Reference::Relative(location) => {
                                            let mut resolver = scope.resolver;
                                            if !is_local(location) {
                                                let location =
                                                    scope.build_url(resolver.scope(), location)?;
                                                if !resolver.contains(location.as_str()) {
                                                    resolver = resolvers
                                                        .get(location.as_str())
                                                        .expect("Unknown reference");
                                                }
                                            };
                                            let (folders, resolved) = resolver.resolve(location)?;
                                            queue.push_back((
                                                Scope::with_folders(resolver, folders),
                                                slot.id,
                                                key.into(),
                                                resolved,
                                            ));
                                        }
                                    };
                                }
                            } else {
                                queue.push_back((scope.clone(), slot.id, key.into(), value));
                            }
                        }
                    }
                    Value::Array(items) => {
                        for (idx, item) in items.iter().enumerate() {
                            queue.push_back((scope.clone(), slot.id, idx.into(), item));
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(output)
    }

    /// Create an empty adjacency list.
    fn empty() -> Self {
        Self {
            // For simpler BFS implementation we put a dummy node in the beginning
            // This way we can assume there is always a parent node, even for the schema root
            nodes: vec![&Value::Null],
            edges: vec![vec![]],
            visited: VisitedMap::new(),
        }
    }

    /// Push a new node & an edge to it.
    fn push(&mut self, parent_id: NodeId, label: EdgeLabel, node: &'s Value) -> NodeSlot {
        let slot = match self.visited.entry(node) {
            Entry::Occupied(entry) => NodeSlot::seen(*entry.get()),
            Entry::Vacant(entry) => {
                // Insert a new node & empty edges for it
                let node_id = NodeId::new(self.nodes.len());
                self.nodes.push(node);
                self.edges.push(vec![]);
                entry.insert(node_id);
                NodeSlot::new(node_id)
            }
        };
        // Insert a new edge from `parent_id` to this node
        self.edges[parent_id.value()].push(Edge::new(label, slot.id));
        slot
    }

    pub(crate) fn range_of(&self, target_id: usize) -> Range<usize> {
        let (start, end) = match self.edges[target_id].as_slice() {
            // Node has no edges
            [] => return 0..0,
            [edge] => (edge, edge),
            [start, .., end] => (start, end),
        };
        // We use non-inclusive ranges, but edges point to precise indexes, hence add 1
        start.target.value()..end.target.value() + 1
    }
}
// TODO: What about specialization? When should it happen? RangeGraph?

#[derive(Debug)]
pub(crate) struct RangeGraph {
    pub(crate) nodes: Vec<Option<Keyword>>,
    pub(crate) edges: Vec<Option<RangedEdge>>,
}

macro_rules! vec_of_nones {
    ($size:expr) => {
        (0..$size).map(|_| None).collect()
    };
}

impl RangeGraph {
    fn new(input: &AdjacencyList<'_>) -> Result<Self> {
        let mut output = RangeGraph {
            nodes: vec_of_nones!(input.nodes.len()),
            edges: vec_of_nones!(input.edges.len()),
        };
        let mut visited = vec![false; input.nodes.len()];
        let mut queue = VecDeque::new();
        queue.push_back((NodeId::new(0), &input.edges[0]));
        while let Some((node_id, node_edges)) = queue.pop_front() {
            if visited[node_id.value()] {
                continue;
            }
            visited[node_id.value()] = true;
            // TODO: Properly track scope of schema/nonschema.
            //       Likely $ref should be schema -> schema, and others are schema -> non-schema
            // TODO: Maybe we can skip pushing edges from non-applicators? they will be no-op here,
            //       but could be skipped upfront
            for edge in node_edges {
                queue.push_back((edge.target, &input.edges[edge.target.value()]));
            }
            if !node_id.is_root() {
                for edge in node_edges {
                    let target_id = edge.target.value();
                    let value = input.nodes[target_id];
                    match edge.label.as_key() {
                        Some("maximum") => {
                            output.set_node(target_id, Maximum::build(value.as_u64().unwrap()));
                        }
                        Some("maxLength") => {
                            output.set_node(target_id, MaxLength::build(value.as_u64().unwrap()));
                        }
                        Some("minProperties") => {
                            output
                                .set_node(target_id, MinProperties::build(value.as_u64().unwrap()));
                        }
                        Some("type") => {
                            let type_value = match value.as_str().unwrap() {
                                "array" => ValueType::Array,
                                "boolean" => ValueType::Boolean,
                                "integer" => ValueType::Integer,
                                "null" => ValueType::Null,
                                "number" => ValueType::Number,
                                "object" => ValueType::Object,
                                "string" => ValueType::String,
                                _ => panic!("invalid type"),
                            };
                            output.set_node(target_id, Type::build(type_value));
                        }
                        Some("properties") => {
                            let edges = input.range_of(target_id);
                            output.set_node(target_id, Properties::build(edges));
                            output.set_edges(&input.edges[target_id], input);
                        }
                        Some("items") => {
                            // TODO: properly set edges & node
                            output.set_node(target_id, Items::build());
                        }
                        Some("allOf") => {
                            let edges = input.range_of(target_id);
                            output.set_node(target_id, AllOf::build(edges));
                            output.set_edges(&input.edges[target_id], input);
                        }
                        Some("$ref") => {
                            // TODO: Inline reference
                            let nodes = input.range_of(target_id);
                            output.set_node(target_id, Ref::build(nodes));
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(output)
    }
}

impl RangeGraph {
    fn set_node(&mut self, id: usize, keyword: Keyword) {
        self.nodes[id] = Some(keyword)
    }
    fn set_edges(&mut self, edges: &[Edge], input: &AdjacencyList) {
        for edge in edges {
            let id = edge.target.value();
            let nodes = input.range_of(id);
            self.edges[id] = Some(RangedEdge::new(edge.label.clone(), nodes));
        }
    }
    fn compress(self) -> CompressedRangeGraph {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct CompressedRangeGraph {
    pub(crate) nodes: Vec<Keyword>,
    pub(crate) edges: Vec<RangedEdge>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum BuildScope {
    Schema,
    NonSchema,
}

#[derive(Clone)]
struct Scope<'s> {
    folders: Vec<&'s str>,
    resolver: &'s Resolver<'s>,
}

impl<'s> Scope<'s> {
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

    pub(crate) fn build_url(&self, scope: &Url, reference: &str) -> Result<Url> {
        let folders = &self.folders;
        let mut location = scope.clone();
        if folders.len() > 1 {
            for folder in folders.iter().skip(1) {
                location = location.join(folder)?;
            }
        }
        Ok(location.join(reference)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        schema::resolving,
        testing::{assert_adjacency_list, assert_compressed_graph, assert_range_graph, load_case},
    };
    use test_case::test_case;

    #[test_case("boolean")]
    #[test_case("maximum")]
    #[test_case("properties")]
    #[test_case("properties-empty")]
    #[test_case("nested-properties")]
    #[test_case("multiple-nodes-each-layer")]
    #[test_case("not-a-keyword-validation")]
    #[test_case("not-a-keyword-ref")]
    #[test_case("ref-recursive-absolute")]
    #[test_case("ref-recursive-self")]
    #[test_case("ref-recursive-between-schemas")]
    #[test_case("ref-remote-pointer")]
    #[test_case("ref-remote-nested")]
    #[test_case("ref-remote-base-uri-change")]
    #[test_case("ref-remote-base-uri-change-folder")]
    #[test_case("ref-remote-base-uri-change-in-subschema")]
    #[test_case("ref-multiple-same-target")]
    fn internal_structure(name: &str) {
        let schema = &load_case(name)["schema"];
        let (root, external) = resolving::resolve(schema).unwrap();
        let resolvers = resolving::build_resolvers(&external);
        let adjacency_list = AdjacencyList::new(schema, &root, &resolvers).unwrap();
        assert_adjacency_list(&adjacency_list);
        let range_graph = RangeGraph::new(&adjacency_list).unwrap();
        assert_range_graph(&range_graph);
        let compressed = range_graph.compress();
        assert_compressed_graph(&compressed);
    }
}
