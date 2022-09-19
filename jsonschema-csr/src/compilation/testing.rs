use crate::compilation::edges::RawEdge;
use serde_json::Value;
use std::collections::HashMap;

/// Ensure all edges are unique.
pub(crate) fn assert_unique_edges(edges: &[RawEdge]) {
    let mut seen = HashMap::new();
    for (index, edge) in edges.iter().enumerate() {
        if let Some(existing_index) = seen.insert(edge, index) {
            panic!(
                "Edge `{} -> {:?} -> {} ` at index {} was already seen at index {}",
                edge.source, edge.label, edge.target, index, existing_index
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

// TODO. check that all edges point to proper keywords
