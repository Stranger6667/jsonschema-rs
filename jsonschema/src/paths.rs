//! Facilities for working with paths within schemas or validated instances.
use std::fmt::Write;
use std::{cell::RefCell, fmt, ops::Deref};

#[derive(Clone, Debug, Eq, PartialEq)]
/// JSON Pointer as a wrapper around individual path components
pub struct JSONPointer(Vec<PathChunk>);

impl JSONPointer {
    /// JSON pointer as a vector of strings. Each component is casted to `String`.
    pub fn into_vec(self) -> Vec<String> {
        self.0
            .iter()
            .map(|item| match item {
                PathChunk::Name(value) => value.to_string(),
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

pub(crate) type InstancePathInner = RefCell<Vec<PathChunk>>;

#[derive(Clone, Debug)]
pub(crate) struct InstancePath(InstancePathInner);

impl InstancePath {
    pub(crate) fn new(inner: InstancePathInner) -> Self {
        Self(inner)
    }
    #[inline]
    pub(crate) fn push(&self, value: impl Into<PathChunk>) {
        self.borrow_mut().push(value.into())
    }
    #[inline]
    pub(crate) fn pop(&self) {
        self.borrow_mut().pop();
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

impl Deref for InstancePath {
    type Target = InstancePathInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&InstancePath> for JSONPointer {
    #[inline]
    fn from(path: &InstancePath) -> Self {
        JSONPointer(path.0.borrow().iter().map(|item| item.to_owned()).collect())
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
