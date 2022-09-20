use std::ops::Range;

/// A label on an edge between two JSON values.
/// It could be either a key name or an index.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum EdgeLabel {
    /// # Example
    ///
    /// `{"name": "Test"}` could be represented as:
    ///
    ///           name
    /// object ---------> "Test"
    ///
    /// The label for the edge between the top-level object and string "Test" is `name` if it is not
    /// a JSON Schema keyword.
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

impl From<usize> for EdgeLabel {
    fn from(value: usize) -> Self {
        EdgeLabel::Index(value)
    }
}

impl From<&str> for EdgeLabel {
    fn from(value: &str) -> Self {
        EdgeLabel::Key(value.to_string().into_boxed_str())
    }
}

impl From<&String> for EdgeLabel {
    fn from(value: &String) -> Self {
        EdgeLabel::Key(value.to_owned().into_boxed_str())
    }
}

/// An edge between two JSON values stored in a non-compressed graph.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct RawEdge {
    pub(crate) source: usize,
    pub(crate) target: usize,
    pub(crate) label: EdgeLabel,
}

impl RawEdge {
    pub(crate) const fn new(source: usize, target: usize, label: EdgeLabel) -> Self {
        Self {
            source,
            target,
            label,
        }
    }
}

/// An edge between two JSON values stored in a non-compressed graph.
#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct Edge {
    pub(crate) label: EdgeLabel,
    pub(crate) keywords: Range<usize>,
}

impl Edge {
    pub(crate) fn new(label: impl Into<EdgeLabel>, keywords: Range<usize>) -> Self {
        Self {
            label: label.into(),
            keywords,
        }
    }
}
