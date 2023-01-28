use crate::{
    schema::graph::{AdjacencyList, CompressedRangeGraph, EdgeLabel, RangeGraph, SingleEdge},
    vocabularies::Keyword,
};
use once_cell::sync::Lazy;
use serde_json::Value;
use std::{collections::HashMap, fs::File, io::BufReader};

/// JSON Schema instances and (in)valid examples for them.
///
/// These samples are specific to this implementation.
pub(crate) static SCHEMAS: Lazy<Value> = Lazy::new(|| {
    let file = File::open("src/data/tests.json").expect("Missing file");
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).expect("Invalid JSON")
});

pub(crate) fn load_case(name: &str) -> &Value {
    &SCHEMAS[name]
}

/// Ensure all edges are unique.
pub(crate) fn assert_unique_edges(edges: &[SingleEdge]) {
    let mut seen = HashMap::new();
    for (index, edge) in edges.iter().enumerate() {
        if let Some(existing_index) = seen.insert(edge, index) {
            panic!(
                "Edge `{:?} -> {:?} ` at index {} was already seen at index {}",
                edge.label, edge.target, index, existing_index
            )
        }
    }
}

/// Display value references in a slice.
pub(crate) fn print_values(values: &[&Value]) {
    for (id, value) in values.iter().enumerate() {
        println!("[{}]: {}", id, value)
    }
}

/// Ensure that all edges & nodes are in the right boundaries.
pub(crate) fn assert_compressed_graph(graph: &CompressedRangeGraph) {
    for edge in graph.edges.iter() {
        assert!(graph.nodes.get(edge.nodes.clone()).is_some());
    }
    for node in graph.nodes.iter() {
        if let Some(range) = node.edges() {
            assert!(graph.edges.get(range).is_some());
        }
    }
}

/// Ensure all edges are pointing to valid nodes.
pub(crate) fn assert_adjacency_list(graph: &AdjacencyList) {
    for (node_id, (node, edges)) in graph.nodes.iter().zip(graph.edges.iter()).enumerate() {
        for edge in edges {
            let by_target_id = graph.nodes[edge.target.value()];
            // 0th node is a dummy node, there is no valid label coming from it
            if node_id != 0 {
                let by_label = match &edge.label {
                    EdgeLabel::Key(key) => {
                        if &**key == "$ref" {
                            // Skip references.
                            // The resolved value should be the same as `by_target_id`, but ensuring
                            // that will require implementing resolving here
                            continue;
                        } else {
                            node.get(&**key)
                        }
                    }
                    EdgeLabel::Index(id) => node.get(id),
                }
                .unwrap_or_else(|| panic!("Value does not exist: {} -> {:?}", node_id, edge.label));
                // Nodes resolved different ways should point to the same value
                assert!(
                    std::ptr::eq(by_label, by_target_id),
                    "Edges do not point to the same node: {} vs {}",
                    by_label,
                    by_target_id
                );
            }
        }
    }
}

pub(crate) fn assert_range_graph(graph: &RangeGraph) {
    for node in graph.nodes.iter().flatten() {
        if let Some(edges) = node.edges() {
            if !edges.is_empty() {
                assert!(
                    graph.edges[edges.clone()].iter().any(|edge| edge.is_some()),
                    "No edges at {:?}",
                    edges
                );
            }
        }
    }
    for edge in graph.edges.iter().flatten() {
        // There should be at least one node in this range (some of them might be skipped)
        let mut exists = false;
        for node in graph.nodes[edge.nodes.clone()].iter().flatten() {
            exists = true;
            if let Keyword::Ref(reference) = node {
                assert!(
                    graph.nodes[reference.nodes.clone()]
                        .iter()
                        .any(|n| n.is_some()),
                    "Reference to invalid nodes"
                )
            }
        }
        assert!(exists, "No nodes at {:?}", edge.nodes);
    }
}
