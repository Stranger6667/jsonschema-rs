use super::options::CompilationOptions;
use crate::{
    compilation::DEFAULT_SCOPE,
    paths::{JSONPointer, JsonPointerNode, PathChunkRef},
    resolver::Resolver,
    schemas,
};
use serde_json::Value;
use std::{borrow::Cow, sync::Arc};
use url::{ParseError, Url};

static DEFAULT_SCHEME: &str = "json-schema";

/// Context holds information about used draft and current scope.
#[derive(Debug, Clone)]
pub(crate) struct CompilationContext<'a> {
    base_uri: BaseUri<'a>,
    pub(crate) config: Arc<CompilationOptions>,
    pub(crate) resolver: Arc<Resolver>,
    pub(crate) schema_path: JsonPointerNode<'a, 'a>,
}

#[derive(Debug, Clone)]
pub(crate) enum BaseUri<'a> {
    /// A base URL was specified, either because we know a reasonable base URI where we retrieved the
    /// schema or because a URI was specified via $id
    Known(Cow<'a, Url>),
    /// We don't know a base URI for this schema
    Unknown,
}

impl<'a> BaseUri<'a> {
    pub(crate) fn with_new_scope(&self, new_scope: &str) -> Result<Self, ParseError> {
        let options = match self {
            BaseUri::Known(u) => Url::options().base_url(Some(u)),
            BaseUri::Unknown => Url::options().base_url(Some(&DEFAULT_SCOPE)),
        };
        Ok(options.parse(new_scope)?.into())
    }
}

impl<'a> From<Option<Url>> for BaseUri<'a> {
    fn from(u: Option<Url>) -> Self {
        u.map_or(BaseUri::Unknown, Into::into)
    }
}

impl<'a> From<&'a Url> for BaseUri<'a> {
    fn from(u: &'a Url) -> Self {
        if u.scheme() == DEFAULT_SCHEME {
            BaseUri::Unknown
        } else {
            BaseUri::Known(Cow::Borrowed(u))
        }
    }
}

impl<'a> From<Url> for BaseUri<'a> {
    fn from(u: Url) -> Self {
        if u.scheme() == DEFAULT_SCHEME {
            BaseUri::Unknown
        } else {
            BaseUri::Known(Cow::Owned(u))
        }
    }
}

impl<'a> From<&'a BaseUri<'a>> for Cow<'a, Url> {
    fn from(uri: &BaseUri<'a>) -> Self {
        match uri {
            BaseUri::Unknown => Cow::Borrowed(&DEFAULT_SCOPE),
            BaseUri::Known(u) => u.clone(),
        }
    }
}

impl<'a> CompilationContext<'a> {
    pub(crate) const fn new(
        scope: BaseUri<'a>,
        config: Arc<CompilationOptions>,
        resolver: Arc<Resolver>,
    ) -> Self {
        CompilationContext {
            base_uri: scope,
            config,
            resolver,
            schema_path: JsonPointerNode::new(),
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
            Ok(CompilationContext {
                base_uri: self.base_uri.with_new_scope(id)?,
                config: Arc::clone(&self.config),
                resolver: Arc::clone(&self.resolver),
                schema_path: self.schema_path.clone(),
            })
        } else {
            Ok(CompilationContext {
                base_uri: self.base_uri.clone(),
                config: Arc::clone(&self.config),
                resolver: Arc::clone(&self.resolver),
                schema_path: self.schema_path.clone(),
            })
        }
    }

    #[inline]
    pub(crate) fn with_path(&'a self, chunk: impl Into<PathChunkRef<'a>>) -> Self {
        let schema_path = self.schema_path.push(chunk);
        CompilationContext {
            base_uri: self.base_uri.clone(),
            config: Arc::clone(&self.config),
            resolver: Arc::clone(&self.resolver),
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
    pub(crate) fn as_pointer_with(&'a self, chunk: impl Into<PathChunkRef<'a>>) -> JSONPointer {
        self.schema_path.push(chunk).into()
    }

    /// Build a new URL. Used for `ref` compilation to keep their full paths.
    pub(crate) fn build_url(&self, reference: &str) -> Result<Url, ParseError> {
        let cowbase: Cow<Url> = (&self.base_uri).into();
        Url::options()
            .base_url(Some(cowbase.as_ref()))
            .parse(reference)
    }

    pub(crate) fn base_uri(&self) -> Option<Url> {
        match &self.base_uri {
            BaseUri::Known(u) => Some(u.as_ref().clone()),
            BaseUri::Unknown => None,
        }
    }
}
