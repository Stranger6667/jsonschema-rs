use super::options::CompilationOptions;
use crate::{
    paths::{InstancePath, JSONPointer, PathChunk},
    schemas,
};
use serde_json::Value;
use std::borrow::Cow;
use url::{ParseError, Url};

/// Context holds information about used draft and current scope.
#[derive(Debug)]
pub(crate) struct CompilationContext<'a> {
    pub(crate) scope: Cow<'a, Url>,
    pub(crate) config: Cow<'a, CompilationOptions>,
    pub(crate) schema_path: InstancePath<'a>,
}

impl<'a> CompilationContext<'a> {
    pub(crate) const fn new(scope: Url, config: Cow<'a, CompilationOptions>) -> Self {
        CompilationContext {
            scope: Cow::Owned(scope),
            config,
            schema_path: InstancePath::new(),
        }
    }

    #[allow(clippy::doc_markdown)]
    /// Push a new scope. All URLs built from the new context will have this scope in them.
    /// Before push:
    ///    scope = http://example.com/
    ///    build_url("#/definitions/foo") -> "http://example.com/#/definitions/foo"
    /// After push this schema - {"$id": "folder/", ...}
    ///    scope = http://example.com/folder/
    ///    build_url("#/definitions/foo") -> "http://example.com/folder/#/definitions/foo"
    ///
    /// In other words it keeps track of sub-folders during compilation.
    #[inline]
    pub(crate) fn push(&'a self, schema: &Value) -> Result<Self, ParseError> {
        if let Some(id) = schemas::id_of(self.config.draft(), schema) {
            let scope = Url::options().base_url(Some(&self.scope)).parse(id)?;
            Ok(CompilationContext {
                scope: Cow::Owned(scope),
                config: Cow::Borrowed(&self.config),
                schema_path: self.schema_path.clone(),
            })
        } else {
            Ok(CompilationContext {
                scope: Cow::Borrowed(self.scope.as_ref()),
                config: Cow::Borrowed(&self.config),
                schema_path: self.schema_path.clone(),
            })
        }
    }

    #[inline]
    pub(crate) fn with_path(&'a self, chunk: impl Into<PathChunk>) -> Self {
        let schema_path = self.schema_path.push(chunk);
        CompilationContext {
            scope: Cow::Borrowed(self.scope.as_ref()),
            config: Cow::Borrowed(&self.config),
            schema_path,
        }
    }

    /// Create a JSON Pointer from the current `schema_path` & a new chunk.
    #[inline]
    pub(crate) fn into_pointer(self) -> JSONPointer {
        self.schema_path.into()
    }

    /// Create a JSON Pointer from the current `schema_path` & a new chunk.
    #[inline]
    pub(crate) fn as_pointer_with(&self, chunk: impl Into<PathChunk>) -> JSONPointer {
        self.schema_path.push(chunk).into()
    }

    /// Build a new URL. Used for `ref` compilation to keep their full paths.
    pub(crate) fn build_url(&self, reference: &str) -> Result<Url, ParseError> {
        Url::options().base_url(Some(&self.scope)).parse(reference)
    }
}
