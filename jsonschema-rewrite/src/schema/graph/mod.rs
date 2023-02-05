use crate::{
    schema::{
        error::Result,
        resolving::{Reference, Resolver, Scope},
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
pub(crate) use nodes::{Node, NodeId, NodeSlot};
use serde_json::Value;
use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    ops::Range,
};

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
    pub(crate) nodes: Vec<Node<'s>>,
    pub(crate) edges: Vec<Vec<Edge>>,
    visited: HashMap<*const Value, NodeId>,
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
            Node::schema(schema),
        ));
        while let Some((mut scope, parent_id, label, node)) = queue.pop_front() {
            let slot = output.push(parent_id, label, node);
            if slot.is_new() {
                match &node.value {
                    Value::Object(object) => {
                        scope.track_folder(object);
                        for (key, value) in object {
                            // TODO: if it is not a schema, then skip ref resolving?
                            let (scope, resolved) = if key == "$ref" {
                                // TODO: If resolved node is in the tree we need to mark it as a schema
                                //       It could happen that it was discovered from a non-$ref
                                //       path and is not considered a schema
                                if let Value::String(reference) = value {
                                    let reference1 = Reference::try_from(reference.as_str())?;
                                    let (scope, resolved) =
                                        reference1.resolve(reference, &scope, resolvers)?;
                                    (scope, Node::schema(resolved))
                                } else {
                                    // TODO: What about references that are not strings?
                                    continue;
                                }
                            } else {
                                (scope.clone(), node.toggle(value))
                            };
                            queue.push_back((scope, slot.id, key.into(), resolved));
                        }
                    }
                    Value::Array(items) => {
                        for (idx, item) in items.iter().enumerate() {
                            queue.push_back((
                                scope.clone(),
                                slot.id,
                                idx.into(),
                                node.toggle(item),
                            ));
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
            nodes: vec![Node::dummy()],
            edges: vec![vec![]],
            visited: HashMap::new(),
        }
    }

    /// Push a new node & an edge to it.
    fn push(&mut self, parent_id: NodeId, label: EdgeLabel, node: Node<'s>) -> NodeSlot {
        let slot = match self.visited.entry(node.value) {
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

    pub(crate) fn value_at(&self, idx: usize) -> &Value {
        &self.nodes[idx].value
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

struct VisitedMap(Vec<bool>);

impl VisitedMap {
    fn new(size: usize) -> Self {
        Self(vec![false; size])
    }
    fn insert(&mut self, node_id: usize) -> bool {
        std::mem::replace(&mut self.0[node_id], true)
    }
}

const ROOT_NODE_ID: usize = 0;

impl RangeGraph {
    fn new(input: &AdjacencyList<'_>) -> Result<Self> {
        let mut output = RangeGraph {
            nodes: vec_of_nones!(input.nodes.len()),
            edges: vec_of_nones!(input.edges.len()),
        };
        let mut visited = VisitedMap::new(input.nodes.len());
        let mut queue = VecDeque::new();
        // Start from the root node
        queue.push_back((ROOT_NODE_ID, &input.edges[ROOT_NODE_ID]));
        while let Some((id, edges)) = queue.pop_front() {
            // Skip already visited nodes
            if visited.insert(id) {
                continue;
            }
            // TODO: Maybe we can skip pushing edges from non-applicators? they will be no-op here,
            //       but could be skipped upfront
            // Traverse child nodes later
            for edge in edges {
                // `edge.target` points at index in the input graph where node & its edges live
                let target_id = edge.target.value();
                queue.push_back((target_id, &input.edges[target_id]));
            }
            if id == ROOT_NODE_ID {
                // Set edges of the root node
                output.set_edges(&input.edges[id], input);
            } else {
                if !input.nodes[id].is_schema() {
                    continue;
                }
                for edge in edges {
                    let target_id = edge.target.value();
                    // Edge points to this JSON value
                    let value = input.value_at(target_id);
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
                            output.set_node(id, Ref::build(nodes));
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
            let nodes = if input.value_at(id).get("$ref").is_some() {
                id..id + 1
            } else {
                input.range_of(id)
            };
            self.edges[id] = Some(RangedEdge::new(edge.label.clone(), nodes));
        }
    }

    fn root_offset(&self) -> usize {
        // INVARIANT: 1th edge always is present as it denotes edges of the root node
        self.edges[1]
            .as_ref()
            .map(|edge| edge.nodes.end)
            // There could be no edges from here in empty schemas
            .unwrap_or(0)
    }

    /// Move edges and nodes into a new graph skipping empty slots and adjusting indexes.
    fn compress(mut self) -> CompressedRangeGraph {
        let mut nodes = vec![];
        let mut edges = vec![];

        // For each node, take their edges and push them into a new vector, skipping empty ones
        // and updating node's edges to their new locations
        for range in self
            .nodes
            .iter_mut()
            .flatten()
            .filter_map(|node| node.edges_mut())
        {
            let node_edges = &self.edges[range.clone()];
            // Now node points to the range in the new vector that holds node's edges
            *range = edges.len()..edges.len() + node_edges.len();
            edges.extend(
                node_edges
                    .iter()
                    .map(|edge| edge.as_ref().expect("Missing edge"))
                    .cloned(),
            );
        }
        // Store where nodes ranges are moved
        let mut moved_nodes: HashMap<Range<usize>, Range<usize>> = HashMap::new();
        // Move all nodes to the new vector and update all edges that point to them with nodes' new locations
        for edge in [self.edges[1].as_mut().unwrap()]
            .into_iter()
            .chain(edges.iter_mut())
        {
            let start = nodes.len();
            let mut end = start;
            for node in self
                .nodes
                .splice(
                    edge.nodes.clone(),
                    std::iter::repeat(None::<Keyword>).take(edge.nodes.len()),
                )
                .flatten()
            {
                end += 1;
                nodes.push(node);
            }
            moved_nodes.insert(edge.nodes.clone(), start..end);
            edge.nodes = start..end;
        }
        // References are updated once everything is moved
        for node in nodes.iter_mut() {
            if let Keyword::Ref(reference) = node {
                reference.nodes = moved_nodes[&reference.nodes].clone();
            }
        }
        CompressedRangeGraph {
            root_offset: self.root_offset(),
            nodes,
            edges,
        }
    }
}

#[derive(Debug)]
pub(crate) struct CompressedRangeGraph {
    pub(crate) root_offset: usize,
    pub(crate) nodes: Vec<Keyword>,
    pub(crate) edges: Vec<RangedEdge>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        schema::resolving,
        testing::{assert_adjacency_list, assert_compressed_graph, assert_range_graph, load_case},
    };
    use test_case::test_case;
    // TODO: distinguish boolean false & true schemas. now they both lead to empty nodes & edges

    #[test_case("boolean")]
    #[test_case("maximum")]
    #[test_case("properties")]
    #[test_case("properties-empty")]
    #[test_case("properties-many")]
    #[test_case("properties-nested")]
    #[test_case("multiple-nodes-each-layer")]
    // TODO: check stuff inside `$defs` / anything references via $ref
    #[test_case("not-a-keyword-validation")]
    #[test_case("not-a-keyword-ref")]
    #[test_case("not-a-keyword-nested")]
    #[test_case("ref-recursive-absolute")]
    #[test_case("ref-recursive-self")]
    #[test_case("ref-recursive-between-schemas")]
    #[test_case("ref-multiple-same-target")]
    #[test_case("ref-remote-pointer")]
    #[test_case("ref-remote-nested")]
    #[test_case("ref-remote-base-uri-change")]
    #[test_case("ref-remote-base-uri-change-folder")]
    #[test_case("ref-remote-base-uri-change-in-subschema")]
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
        // The check above would verify references only if they are present in the compressed graph
        // Hence check them manually too.
        match name {
            "ref-recursive-absolute" => {
                assert_eq!(compressed.nodes[2], Keyword::Ref(Ref { nodes: 0..2 }));
                assert_eq!(compressed.nodes[3], Keyword::Ref(Ref { nodes: 0..2 }));
                assert_eq!(compressed.edges[0].nodes, 2..3);
                assert_eq!(compressed.edges[1].nodes, 3..4);
            }
            "ref-recursive-self" => {
                assert_eq!(compressed.nodes[0], Keyword::Ref(Ref { nodes: 0..1 }));
                assert!(compressed.edges.is_empty());
            }
            "ref-recursive-between-schemas" => {
                assert_eq!(compressed.nodes[2], Keyword::Ref(Ref { nodes: 0..1 }));
                assert_eq!(compressed.edges[1].nodes, 2..3);
            }
            "ref-multiple-same-target" => {
                assert_eq!(compressed.nodes[1], Keyword::Ref(Ref { nodes: 2..3 }));
                assert_eq!(compressed.nodes[2], Keyword::Ref(Ref { nodes: 1..2 }));
                assert_eq!(compressed.edges[0].nodes, 1..2);
                assert_eq!(compressed.edges[1].nodes, 2..3);
            }
            "ref-remote-pointer" => {
                todo!("missing nodes")
            }
            _ => {}
        }
    }
}
