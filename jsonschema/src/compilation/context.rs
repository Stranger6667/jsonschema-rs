use super::options::CompilationOptions;
use crate::{
    paths::{InstancePath, JSONPointer, PathChunk},
    schemas,
};
use parking_lot::Mutex;
use serde_json::Value;
use std::{borrow::Cow, sync::Arc};
use url::{ParseError, Url};

/// Context holds information about used draft and current scope.
#[derive(Debug)]
pub(crate) struct CompilationContext<'a> {
    pub(crate) scope: Cow<'a, Url>,
    pub(crate) config: Cow<'a, CompilationOptions>,
    pub(crate) schema_path: InstancePath<'a>,
    validators: Arc<Mutex<Vec<ValidatorBuf>>>,
}

impl<'a> CompilationContext<'a> {
    pub(crate) fn new(scope: Url, config: Cow<'a, CompilationOptions>) -> Self {
        CompilationContext {
            scope: Cow::Owned(scope),
            config,
            schema_path: InstancePath::new(),
            validators: Default::default(),
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
                validators: self.validators.clone(),
            })
        } else {
            Ok(CompilationContext {
                scope: Cow::Borrowed(self.scope.as_ref()),
                config: Cow::Borrowed(&self.config),
                schema_path: self.schema_path.clone(),
                validators: self.validators.clone(),
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
            validators: self.validators.clone(),
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

    #[inline]
    pub(crate) fn add_validator(&self, validator: ValidatorBuf) -> ValidatorRef {
        let r = ValidatorRef::new(&validator);
        self.validators.lock().push(validator);
        r
    }
}

pub(crate) use validator_types::*;

#[cfg(debug_assertions)]
mod validator_types {
    use crate::{paths::InstancePath, validator::Validate, ErrorIterator, JSONSchema};
    use serde_json::Value;
    use std::sync::{Arc, Weak};

    #[repr(transparent)]
    pub(crate) struct ValidatorBuf(pub(super) Arc<dyn Validate + Send + Sync>);

    impl ValidatorBuf {
        pub(crate) fn new<V: Validate + Send + Sync + 'static>(validator: V) -> Self {
            Self(Arc::new(validator))
        }
    }

    impl core::fmt::Debug for ValidatorBuf {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            (&*self.0).fmt(f)
        }
    }

    #[repr(transparent)]
    pub(crate) struct ValidatorRef(Weak<dyn Validate + Send + Sync>);

    impl ValidatorRef {
        pub(super) fn new(validator: &ValidatorBuf) -> Self {
            Self(Arc::downgrade(&validator.0))
        }
    }

    impl core::fmt::Debug for ValidatorRef {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            (&*self.0.upgrade().unwrap()).fmt(f)
        }
    }

    impl core::fmt::Display for ValidatorRef {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            (&*self.0.upgrade().unwrap()).fmt(f)
        }
    }

    impl Validate for ValidatorRef {
        fn validate<'a>(
            &self,
            schema: &'a JSONSchema,
            instance: &'a Value,
            instance_path: &InstancePath,
        ) -> ErrorIterator<'a> {
            self.0
                .upgrade()
                .unwrap()
                .validate(schema, instance, instance_path)
        }

        fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
            self.0.upgrade().unwrap().is_valid(schema, instance)
        }
    }
}

#[cfg(not(debug_assertions))]
mod validator_types {
    use crate::{paths::InstancePath, validator::Validate, ErrorIterator, JSONSchema};
    use serde_json::Value;

    #[repr(transparent)]
    pub(crate) struct ValidatorBuf(Box<dyn Validate + Send + Sync>);

    impl ValidatorBuf {
        pub(crate) fn new<V: Validate + Send + Sync + 'static>(validator: V) -> Self {
            Self(Box::new(validator))
        }
    }

    impl core::fmt::Debug for ValidatorBuf {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            (&*self.0).fmt(f)
        }
    }

    #[repr(transparent)]
    pub(crate) struct ValidatorRef(*const (dyn Validate + Send + Sync));

    impl ValidatorRef {
        pub(super) fn new(validator: &ValidatorBuf) -> Self {
            Self(&*validator.0)
        }
    }

    impl core::fmt::Debug for ValidatorRef {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            unsafe { (&*self.0).fmt(f) }
        }
    }

    unsafe impl Send for ValidatorRef {}
    unsafe impl Sync for ValidatorRef {}

    impl core::fmt::Display for ValidatorRef {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            unsafe { (&*self.0).fmt(f) }
        }
    }

    impl Validate for ValidatorRef {
        fn validate<'a>(
            &self,
            schema: &'a JSONSchema,
            instance: &'a Value,
            instance_path: &InstancePath,
        ) -> ErrorIterator<'a> {
            unsafe { (&*self.0).validate(schema, instance, instance_path) }
        }

        fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
            unsafe { (&*self.0).is_valid(schema, instance) }
        }
    }
}
