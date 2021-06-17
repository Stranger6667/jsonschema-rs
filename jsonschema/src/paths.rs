//! Facilities for working with paths within schemas or validated instances.
use std::{fmt, fmt::Write, slice::Iter};

#[derive(Clone, Debug, Eq, PartialEq)]
/// JSON Pointer as a wrapper around individual path components.
pub struct JSONPointer(Vec<PathChunk>);

impl JSONPointer {
    #[must_use]
    /// JSON pointer as a vector of strings. Each component is casted to `String`. Consumes `JSONPointer`.
    pub fn into_vec(self) -> Vec<String> {
        self.0
            .into_iter()
            .map(|item| match item {
                PathChunk::Property(value) => value,
                PathChunk::Index(idx) => idx.to_string(),
            })
            .collect()
    }

    #[must_use]
    /// Return an iterator over the underlying vector of path components.
    pub fn iter(&self) -> Iter<'_, PathChunk> {
        self.0.iter()
    }
}

impl Default for JSONPointer {
    fn default() -> Self {
        JSONPointer(Vec::new())
    }
}

impl fmt::Display for JSONPointer {
    fn fmt(&self, mut f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.0.is_empty() {
            for chunk in &self.0 {
                f.write_char('/')?;
                match chunk {
                    PathChunk::Property(value) => f.write_str(value)?,
                    PathChunk::Index(idx) => itoa::fmt(&mut f, *idx)?,
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
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
pub enum PathChunk {
    /// Property name within a JSON object.
    Property(String),
    /// Index within a JSON array.
    Index(usize),
}

#[derive(Debug)]
pub(crate) struct InstancePath<'a> {
    pub(crate) chunk: Option<PathChunk>,
    pub(crate) parent: Option<&'a InstancePath<'a>>,
}

impl<'a> InstancePath<'a> {
    pub(crate) const fn new() -> Self {
        InstancePath {
            chunk: None,
            parent: None,
        }
    }

    pub(crate) fn push(&'a self, chunk: impl Into<PathChunk>) -> Self {
        InstancePath {
            chunk: Some(chunk.into()),
            parent: Some(self),
        }
    }

    pub(crate) fn to_vec(&'a self) -> Vec<PathChunk> {
        // The path capacity should be the average depth so we avoid extra allocations
        let mut result = Vec::with_capacity(6);
        let mut current = self;
        if let Some(chunk) = &current.chunk {
            result.push(chunk.clone())
        }
        while let Some(next) = current.parent {
            current = next;
            if let Some(chunk) = &current.chunk {
                result.push(chunk.clone())
            }
        }
        result.reverse();
        result
    }
}

impl IntoIterator for JSONPointer {
    type Item = PathChunk;
    type IntoIter = <Vec<PathChunk> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a JSONPointer {
    type Item = &'a PathChunk;
    type IntoIter = Iter<'a, PathChunk>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl From<String> for PathChunk {
    #[inline]
    fn from(value: String) -> Self {
        PathChunk::Property(value)
    }
}
impl From<usize> for PathChunk {
    #[inline]
    fn from(value: usize) -> Self {
        PathChunk::Index(value)
    }
}

impl<'a> From<&'a InstancePath<'a>> for JSONPointer {
    #[inline]
    fn from(path: &'a InstancePath<'a>) -> Self {
        JSONPointer(path.to_vec())
    }
}

impl From<&[&str]> for JSONPointer {
    #[inline]
    fn from(path: &[&str]) -> Self {
        JSONPointer(
            path.iter()
                .map(|item| PathChunk::Property((*item).to_string()))
                .collect(),
        )
    }
}
impl From<&[PathChunk]> for JSONPointer {
    #[inline]
    fn from(path: &[PathChunk]) -> Self {
        JSONPointer(path.to_vec())
    }
}
