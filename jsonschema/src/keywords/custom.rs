use crate::compilation::context::CompilationContext;
use crate::keywords::CompilationResult;
use crate::paths::{InstancePath, JSONPointer, PathChunk};
use crate::validator::Validate;
use crate::ErrorIterator;
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

/// Trait that allows implementing custom validation for keywords.
pub trait Keyword {
    fn is_valid(&self, instance: &Value) -> bool;
}

pub trait CustomKeywordValidator: Send + Sync {
    fn compile<'a>(
        parent: &'a serde_json::Map<String, Value>,
        schema: &'a Value,
        context: &CompilationContext,
    ) -> Box<dyn CustomKeywordValidator>
    where
        Self: Sized;
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
    fn init(&self, schema: &Value) -> Box<dyn Keyword>;
}

impl<F> sealed::Sealed for F where F: Fn(&Value) -> Box<dyn Keyword> + Send + Sync {}

impl<F> KeywordFactory for F
where
    F: Fn(&Value) -> Box<dyn Keyword> + Send + Sync,
{
    fn init(&self, schema: &Value) -> Box<dyn Keyword> {
        self(schema)
    }
}
