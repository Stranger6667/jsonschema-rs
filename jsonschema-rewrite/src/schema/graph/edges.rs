use std::ops::Range;

use super::nodes::NodeId;

/// A label on an edge between two JSON values.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum EdgeLabel {
    /// # Example
    ///
    /// `{"name": "Test"}` could be represented as:
    ///
    ///           name
    /// object ---------> "Test"
    ///
    /// The label for the edge between the top-level object and string "Test" is `name`.
    Key(Box<str>),
    /// # Example
    ///
    /// `["Test"]` could be represented as:
    ///
    ///          0
    /// array ------> "Test"
    ///
    /// The label for the edge between the top-level array and string "Test" is `0`.
    Index(usize),
}

impl EdgeLabel {
    pub(crate) fn as_key(&self) -> Option<&str> {
        if let EdgeLabel::Key(key) = self {
            Some(&**key)
        } else {
            None
        }
    }
}

impl From<usize> for EdgeLabel {
    fn from(value: usize) -> Self {
        EdgeLabel::Index(value)
    }
}

impl From<&String> for EdgeLabel {
    fn from(value: &String) -> Self {
        EdgeLabel::Key(value.to_owned().into_boxed_str())
    }
}

/// An edge between two JSON values stored in adjacency list.
///
/// # Example
///
/// JSON:
///
/// ```json
/// {
///     "properties": {
///         "A": {
///             "type": "object"
///         },
///         "B": {
///             "type": "string"
///         }
///     }
/// }
/// ```
///
/// ("A", 1) - an edge between `<properties>` and `<type: object>`
/// ("B", 2) - an edge between `<properties>` and `<type: string>`
///
/// ```text
///   Nodes                      Edges
///
/// [                         [
///   0 <properties>            [("A", 1), ("B", 2)]
///   1 <type: object>          []
///   2 <type: string>          []
/// ]                         ]
/// ```
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct Edge {
    pub(crate) label: EdgeLabel,
    pub(crate) target: NodeId,
}

impl Edge {
    pub(crate) fn new(label: impl Into<EdgeLabel>, target: NodeId) -> Edge {
        Edge {
            label: label.into(),
            target,
        }
    }
}

/// An edge between a single JSON value and a range of JSON values that are stored contiguously.
///
/// # Example
///
/// JSON:
///
/// ```json
/// {
///     "properties": {
///         "A": {
///             "type": "object",
///             "maxLength": 5
///         },
///         "B": {
///             "type": "string"
///         }
///     }
/// }
/// ```
///
/// ("A", 1..3) - an edge between `<properties>` and `<type: object>` & `<maxLength: 5>`
/// ("B", 3..4) - an edge between `<properties>` and `<type: string>`
///
/// ```text
///   Nodes                                                              Edges
///
/// [                                                                 [
/// -- 0..1 `/`                                    |------------>     -- 0..2 (`properties' edges)
///      <properties> -----> 0..2 ---------------->|  |<------------------ A
/// -- 1..3 `/properties/A`               <--- 1..3 <-|  |<--------------- B
///      <type: object>                                  |            ]
///      <maxLength: 5>                                  |
/// -- 3..4 `/properties/B`               <--- 3..4 <----|
///      <type: string>
/// ]
/// ```
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct RangedEdge {
    /// A label for this edge.
    pub(crate) label: EdgeLabel,
    /// A range of nodes referenced by this edge.
    pub(crate) nodes: Range<usize>,
}

impl RangedEdge {
    pub(crate) fn new(label: impl Into<EdgeLabel>, nodes: Range<usize>) -> RangedEdge {
        RangedEdge {
            label: label.into(),
            nodes,
        }
    }
}
