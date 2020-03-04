use crate::schemas::{id_of, Draft};
use serde_json::Value;
use std::borrow::Cow;
use url::{ParseError, Url};

/// Context holds information about used draft and current scope.
#[derive(Debug)]
pub struct CompilationContext<'a> {
    pub(crate) scope: Cow<'a, Url>,
    pub(crate) draft: Draft,
}

impl<'a> CompilationContext<'a> {
    pub(crate) fn new(scope: Url, draft: Draft) -> Self {
        CompilationContext {
            scope: Cow::Owned(scope),
            draft,
        }
    }

    /// Push a new scope. All URLs built from the new context will have this scope in them.
    /// Before push:
    ///    scope = http://example.com/
    ///    build_url("#/definitions/foo") -> "http://example.com/#/definitions/foo"
    /// After push this schema - {"$id": "folder/", ...}
    ///    scope = http://example.com/folder/
    ///    build_url("#/definitions/foo") -> "http://example.com/folder/#/definitions/foo"
    ///
    /// In other words it keeps track of sub-folders during compilation.
    pub(crate) fn push(&'a self, schema: &Value) -> Self {
        match id_of(self.draft, schema) {
            Some(id) => {
                let scope = Url::options()
                    .base_url(Some(&self.scope))
                    .parse(id)
                    .unwrap();
                CompilationContext {
                    scope: Cow::Owned(scope),
                    draft: self.draft,
                }
            }
            None => CompilationContext {
                scope: Cow::Borrowed(self.scope.as_ref()),
                draft: self.draft,
            },
        }
    }

    /// Build a new URL. Used for `ref` compilation to keep their full paths.
    pub(crate) fn build_url(&self, reference: &str) -> Result<Url, ParseError> {
        Url::options().base_url(Some(&self.scope)).parse(reference)
    }
}
