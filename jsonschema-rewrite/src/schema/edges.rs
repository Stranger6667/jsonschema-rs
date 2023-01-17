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

/// An edge between two JSON values stored in a graph.
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
/// ("A", 0, 1) - an edge between `<properties>` and `<type: object>`
/// ("B", 0, 2) - an edge between `<properties>` and `<type: string>`
///
///          --------------- B ------------->
///         |  ------ A ---->               |
///         | |             |               |
/// `[ <properties>, <type: object>, <type: string>]`
///         0              1               2
///
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct SingleEdge {
    pub(crate) label: EdgeLabel,
    pub(crate) source: usize,
    pub(crate) target: usize,
}

impl SingleEdge {
    pub(crate) const fn new(label: EdgeLabel, source: usize, target: usize) -> Self {
        Self {
            source,
            target,
            label,
        }
    }
}

/// A convenience shortcut to create `SingleEdge`.
pub(crate) fn single(label: impl Into<EdgeLabel>, source: usize, target: usize) -> SingleEdge {
    SingleEdge::new(label.into(), source, target)
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
/// ("A", 1..3) - an edge between `<properties>` and `<type: object>` + `<maxLength: 5>`
/// ("B", 3..4) - an edge between `<properties>` and `<type: string>`
///
///          ---------------------- B ---------------------->
///         |  ------------->------ A ------>               |
///         | |             |               |               |
/// `[ <properties>, <type: object>, <maxLength: 5>, <type: string>]`
///         0              1               2               3
///
#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct MultiEdge {
    pub(crate) label: EdgeLabel,
    pub(crate) keywords: Range<usize>,
}

impl MultiEdge {
    pub(crate) fn new(label: EdgeLabel, keywords: Range<usize>) -> Self {
        Self { label, keywords }
    }
}

/// A convenience shortcut to create `MultiEdge`.
pub(crate) fn multi(label: impl Into<EdgeLabel>, keywords: Range<usize>) -> MultiEdge {
    MultiEdge::new(label.into(), keywords)
}
