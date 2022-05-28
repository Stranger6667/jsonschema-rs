/// A label on an edge between two JSON values.
/// It could be either a key name or an index.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum EdgeLabel {
    /// # Example
    ///
    /// `{"name": "Test"}` could be represented as:
    ///
    ///           name
    /// object ---------> "Test"
    ///
    /// The label for the edge between the top-level object and string "Test" is `name`.
    Key(String),
    ///
    /// `["Test"]` could be represented as:
    ///
    ///          0
    /// array ------> "Test"
    ///
    /// The label for the edge between the top-level array and string "Test" is `0`.
    Index(usize),
    // TODO. store keywords as a separate variant?
}

/// Create a new `EdgeLabel`.
///
/// We can use the `Into` trait directly, but a separate constructor allows us to avoid
/// specifying types on containers.
pub(crate) fn label(value: impl Into<EdgeLabel>) -> EdgeLabel {
    value.into()
}

impl From<usize> for EdgeLabel {
    fn from(value: usize) -> Self {
        EdgeLabel::Index(value)
    }
}

impl From<&str> for EdgeLabel {
    fn from(value: &str) -> Self {
        EdgeLabel::Key(value.to_string())
    }
}

impl From<&String> for EdgeLabel {
    fn from(value: &String) -> Self {
        EdgeLabel::Key(value.to_owned())
    }
}

/// An edge between two JSON values stored in a non-compressed graph.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct RawEdge {
    pub(crate) source: usize,
    pub(crate) target: usize,
    pub(crate) label: EdgeLabel,
}

impl RawEdge {
    pub(crate) fn new(source: usize, target: usize, label: EdgeLabel) -> Self {
        Self {
            source,
            target,
            label,
        }
    }
    pub(crate) fn compress(&self) -> CompressedEdge {
        CompressedEdge {
            target: self.target,
            label: self.label.clone(),
        }
    }
}

/// An edge between two JSON values stored in the compressed sparse row format.
#[derive(Debug)]
pub(crate) struct CompressedEdge {
    pub(crate) target: usize,
    pub(crate) label: EdgeLabel,
}
