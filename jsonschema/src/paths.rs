//! Facilities for working with paths within schemas or validated instances.
use std::{fmt, fmt::Write};

#[derive(Clone, Debug, Eq, PartialEq)]
/// JSON Pointer as a wrapper around individual path components
pub struct JSONPointer(Vec<PathChunk>);

impl JSONPointer {
    /// JSON pointer as a vector of strings. Each component is casted to `String`. Consumes `JSONPointer`.
    pub fn into_vec(self) -> Vec<String> {
        self.0
            .into_iter()
            .map(|item| match item {
                PathChunk::Name(value) => value,
                PathChunk::Index(idx) => idx.to_string(),
            })
            .collect()
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
                    PathChunk::Name(value) => f.write_str(value)?,
                    PathChunk::Index(idx) => itoa::fmt(&mut f, *idx)?,
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PathChunk {
    Name(String),
    Index(usize),
}

#[derive(Debug)]
pub(crate) struct InstancePath<'a> {
    pub(crate) chunk: Option<PathChunk>,
    pub(crate) parent: Option<&'a InstancePath<'a>>,
}

impl<'a> InstancePath<'a> {
    pub(crate) fn new() -> Self {
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

impl From<String> for PathChunk {
    #[inline]
    fn from(value: String) -> Self {
        PathChunk::Name(value)
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
                .map(|item| PathChunk::Name(item.to_string()))
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
