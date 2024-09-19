//! Facilities for working with paths within schemas or validated instances.
use std::{fmt, fmt::Write, slice::Iter};

/// JSON Pointer as a wrapper around individual path components.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct JsonPointer(Vec<PathChunk>);

#[deprecated(
    since = "0.20.0",
    note = "Use `JsonPointer` instead. This type will be removed in a future release."
)]
/// Use [`JsonPointer`] instead. This type will be removed in a future release.
pub type JSONPointer = JsonPointer;

impl JsonPointer {
    /// JSON pointer as a vector of strings. Each component is casted to `String`.
    #[must_use]
    pub fn into_vec(self) -> Vec<String> {
        self.0
            .into_iter()
            .map(|item| match item {
                PathChunk::Property(value) => value.into_string(),
                PathChunk::Index(idx) => idx.to_string(),
                PathChunk::Keyword(keyword) => keyword.to_string(),
            })
            .collect()
    }

    /// Return an iterator over the underlying vector of path components.
    pub fn iter(&self) -> Iter<'_, PathChunk> {
        self.0.iter()
    }
    /// Take the last pointer chunk.
    #[must_use]
    #[inline]
    pub fn last(&self) -> Option<&PathChunk> {
        self.0.last()
    }

    pub(crate) fn clone_with(&self, chunk: impl Into<PathChunk>) -> Self {
        let mut new = self.clone();
        new.0.push(chunk.into());
        new
    }

    pub(crate) fn extend_with(&self, chunks: &[PathChunk]) -> Self {
        let mut new = self.clone();
        new.0.extend_from_slice(chunks);
        new
    }
}

impl serde::Serialize for JsonPointer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl fmt::Display for JsonPointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.0.is_empty() {
            for chunk in &self.0 {
                f.write_char('/')?;
                match chunk {
                    PathChunk::Property(value) => {
                        for ch in value.chars() {
                            match ch {
                                '/' => f.write_str("~1")?,
                                '~' => f.write_str("~0")?,
                                _ => f.write_char(ch)?,
                            }
                        }
                    }
                    PathChunk::Index(idx) => f.write_str(itoa::Buffer::new().format(*idx))?,
                    PathChunk::Keyword(keyword) => f.write_str(keyword)?,
                }
            }
        }
        Ok(())
    }
}

/// A key within a JSON object or an index within a JSON array.
/// A sequence of chunks represents a valid path within a JSON value.
///
/// Example:
/// ```json
/// {
///    "cmd": ["ls", "-lh", "/home"]
/// }
/// ```
///
/// To extract "/home" from the JSON object above, we need to take two steps:
/// 1. Go into property "cmd". It corresponds to `PathChunk::Property("cmd".to_string())`.
/// 2. Take the 2nd value from the array - `PathChunk::Index(2)`
///
/// The primary purpose of this enum is to avoid converting indexes to strings during validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PathChunk {
    /// Property name within a JSON object.
    Property(Box<str>),
    /// Index within a JSON array.
    Index(usize),
    /// JSON Schema keyword.
    Keyword(&'static str),
}

/// A borrowed variant of [`PathChunk`].
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PathChunkRef<'a> {
    /// Property name within a JSON object.
    Property(&'a str),
    /// JSON Schema keyword.
    Index(usize),
}

/// A node in a linked list representing a JSON pointer.
///
/// [`JsonPointerNode`] is used to build a JSON pointer incrementally during the JSON Schema validation process.
/// Each node contains a segment of the JSON pointer and a reference to its parent node, forming
/// a linked list.
///
/// The linked list representation allows for efficient traversal and manipulation of the JSON pointer
/// without the need for memory allocation.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JsonPointerNode<'a, 'b> {
    pub(crate) segment: PathChunkRef<'a>,
    pub(crate) parent: Option<&'b JsonPointerNode<'b, 'a>>,
}

impl Default for JsonPointerNode<'_, '_> {
    fn default() -> Self {
        JsonPointerNode::new()
    }
}

impl<'a, 'b> JsonPointerNode<'a, 'b> {
    /// Create a root node of a JSON pointer.
    pub const fn new() -> Self {
        JsonPointerNode {
            // The value does not matter, it will never be used
            segment: PathChunkRef::Index(0),
            parent: None,
        }
    }

    /// Push a new segment to the JSON pointer.
    #[inline]
    pub fn push(&'a self, segment: impl Into<PathChunkRef<'a>>) -> Self {
        JsonPointerNode {
            segment: segment.into(),
            parent: Some(self),
        }
    }

    /// Convert the JSON pointer node to a vector of path segments.
    pub fn to_vec(&'a self) -> Vec<PathChunk> {
        // Walk the linked list to calculate the capacity
        let mut capacity = 0;
        let mut head = self;
        while let Some(next) = head.parent {
            head = next;
            capacity += 1;
        }
        // Callect the segments from the head to the tail
        let mut buffer = Vec::with_capacity(capacity);
        let mut head = self;
        if head.parent.is_some() {
            buffer.push(head.segment.into())
        }
        while let Some(next) = head.parent {
            head = next;
            if head.parent.is_some() {
                buffer.push(head.segment.into());
            }
        }
        // Reverse the buffer to get the segments in the correct order
        buffer.reverse();
        buffer
    }
}

impl IntoIterator for JsonPointer {
    type Item = PathChunk;
    type IntoIter = <Vec<PathChunk> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a JsonPointer {
    type Item = &'a PathChunk;
    type IntoIter = Iter<'a, PathChunk>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl From<String> for PathChunk {
    #[inline]
    fn from(value: String) -> Self {
        PathChunk::Property(value.into_boxed_str())
    }
}

impl From<&'static str> for PathChunk {
    #[inline]
    fn from(value: &'static str) -> Self {
        PathChunk::Keyword(value)
    }
}

impl From<usize> for PathChunk {
    #[inline]
    fn from(value: usize) -> Self {
        PathChunk::Index(value)
    }
}

impl<'a> From<&'a str> for PathChunkRef<'a> {
    #[inline]
    fn from(value: &'a str) -> PathChunkRef<'a> {
        PathChunkRef::Property(value)
    }
}

impl From<usize> for PathChunkRef<'_> {
    #[inline]
    fn from(value: usize) -> Self {
        PathChunkRef::Index(value)
    }
}

impl<'a> From<PathChunkRef<'a>> for PathChunk {
    #[inline]
    fn from(value: PathChunkRef<'a>) -> Self {
        match value {
            PathChunkRef::Property(value) => PathChunk::Property(value.into()),
            PathChunkRef::Index(value) => PathChunk::Index(value),
        }
    }
}

impl<'a, 'b> From<&'a JsonPointerNode<'a, 'b>> for JsonPointer {
    #[inline]
    fn from(path: &'a JsonPointerNode<'a, 'b>) -> Self {
        JsonPointer(path.to_vec())
    }
}

impl From<JsonPointerNode<'_, '_>> for JsonPointer {
    #[inline]
    fn from(path: JsonPointerNode<'_, '_>) -> Self {
        JsonPointer(path.to_vec())
    }
}

impl From<&[&str]> for JsonPointer {
    #[inline]
    fn from(path: &[&str]) -> Self {
        JsonPointer(
            path.iter()
                .map(|item| PathChunk::Property((*item).into()))
                .collect(),
        )
    }
}
impl From<&[PathChunk]> for JsonPointer {
    #[inline]
    fn from(path: &[PathChunk]) -> Self {
        JsonPointer(path.to_vec())
    }
}

impl From<&str> for JsonPointer {
    fn from(value: &str) -> Self {
        JsonPointer(vec![value.to_string().into()])
    }
}

#[cfg(test)]
mod tests {
    use super::JsonPointer;
    use serde_json::json;

    #[test]
    fn json_pointer_to_string() {
        let chunks = ["/", "~"];
        let pointer = JsonPointer::from(&chunks[..]).to_string();
        assert_eq!(pointer, "/~1/~0");
        let data = json!({"/": {"~": 42}});
        assert_eq!(data.pointer(&pointer), Some(&json!(42)))
    }
}
