use crate::compilation::context::CompilationContext;
use crate::keywords::CompilationResult;
use crate::paths::{InstancePath, JSONPointer, PathChunk};
use crate::validator::Validate;
use crate::{ErrorIterator, ValidationError};
use serde_json::Value;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

/// Custom keyword validation implemented by user provided validation functions.
pub(crate) struct CompiledCustomKeywordValidator {
    schema: Arc<Value>,
    subschema: Arc<Value>,
    subschema_path: JSONPointer,
    validator: Box<dyn CustomKeywordValidator>,
}

impl Display for CompiledCustomKeywordValidator {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Validate for CompiledCustomKeywordValidator {
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'instance> {
        self.validator.validate(
            instance,
            instance_path.into(),
            self.subschema.clone(),
            self.subschema_path.clone(),
            self.schema.clone(),
        )
    }

    fn is_valid(&self, instance: &Value) -> bool {
        self.validator
            .is_valid(instance, &self.subschema, &self.schema)
    }
}

pub(crate) fn compile<'a>(
    context: &CompilationContext,
    keyword: impl Into<PathChunk>,
    validator: Box<dyn CustomKeywordValidator>,
    subschema: Value,
    schema: Value,
) -> CompilationResult<'a> {
    let subschema_path = context.as_pointer_with(keyword);
    Ok(Box::new(CompiledCustomKeywordValidator {
        schema: Arc::new(schema),
        subschema: Arc::new(subschema),
        subschema_path,
        validator,
    }))
}

mod sealed {
    pub trait Sealed {}
}

pub(crate) struct CustomKeyword {
    inner: Box<dyn Keyword>,
}

impl CustomKeyword {
    pub(crate) fn new(inner: Box<dyn Keyword>) -> Self {
        Self { inner }
    }
}

impl Display for CustomKeyword {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Validate for CustomKeyword {
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'instance> {
        self.inner.validate(instance, instance_path)
    }

    fn is_valid(&self, instance: &Value) -> bool {
        self.inner.is_valid(instance)
    }
}

/// Trait that allows implementing custom validation for keywords.
pub trait Keyword: Send + Sync {
    fn is_valid(&self, instance: &Value) -> bool;
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'instance>;
}

pub trait CustomKeywordValidator: Send + Sync {
    /// Validate [instance](Value) according to a custom specification
    ///
    /// A custom keyword validator may be used when a validation that cannot, or
    /// cannot be be easily or efficiently expressed in JSON schema.
    ///
    /// The custom validation is applied in addition to the JSON schema validation.
    /// Validate an instance returning any and all detected validation errors
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: JSONPointer,
        subschema: Arc<Value>,
        subschema_path: JSONPointer,
        schema: Arc<Value>,
    ) -> ErrorIterator<'instance>;
    /// Determine if an instance is valid
    fn is_valid<'schema>(
        &self,
        instance: &Value,
        subschema: &'schema Value,
        schema: &'schema Value,
    ) -> bool;
}

pub trait KeywordFactory: Send + Sync + sealed::Sealed {
    fn init<'a>(
        &self,
        schema: &'a Value,
        path: JSONPointer,
    ) -> Result<Box<dyn Keyword>, ValidationError<'a>>;
}

impl<F> sealed::Sealed for F where
    F: for<'a> Fn(&'a Value, JSONPointer) -> Result<Box<dyn Keyword>, ValidationError<'a>>
        + Send
        + Sync
{
}

impl<F> KeywordFactory for F
where
    F: for<'a> Fn(&'a Value, JSONPointer) -> Result<Box<dyn Keyword>, ValidationError<'a>>
        + Send
        + Sync,
{
    fn init<'a>(
        &self,
        schema: &'a Value,
        path: JSONPointer,
    ) -> Result<Box<dyn Keyword>, ValidationError<'a>> {
        self(schema, path)
    }
}
