use crate::schema::graph::{AdjacencyList, CompressedRangeGraph, EdgeLabel, RangeGraph};
use once_cell::sync::Lazy;
use serde_json::Value;
use std::{fs::File, io::BufReader};

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
            let by_target_id = graph.nodes[edge.target.value()].value;
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
                            node.value.get(&**key)
                        }
                    }
                    EdgeLabel::Index(id) => node.value.get(id),
                }
                .unwrap_or_else(|| panic!("Value does not exist: {} -> {:?}", node_id, edge.label));
                // Nodes resolved different ways should point to the same value
                assert!(
                    std::ptr::eq(by_label, by_target_id),
                    "Edges do not point to the same node: {by_label} vs {by_target_id}"
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
                    "No edges at {edges:?}"
                );
            }
        }
    }
    // Edges may point to empty schemas, meaning that there will be no nodes at referenced ranges
}
