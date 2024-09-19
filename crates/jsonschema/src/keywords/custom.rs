use crate::{
    paths::{JsonPointer, JsonPointerNode},
    validator::Validate,
    ErrorIterator, ValidationError,
};
use serde_json::{Map, Value};
use std::fmt::{Display, Formatter};

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
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        self.inner.validate(instance, instance_path)
    }

    fn is_valid(&self, instance: &Value) -> bool {
        self.inner.is_valid(instance)
    }
}

/// Trait that allows implementing custom validation for keywords.
pub trait Keyword: Send + Sync {
    /// Validate instance according to a custom specification.
    ///
    /// A custom keyword validator may be used when a validation that cannot be
    /// easily or efficiently expressed in JSON schema.
    ///
    /// The custom validation is applied in addition to the JSON schema validation.
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance>;
    /// Validate instance and return a boolean result.
    ///
    /// Could be potentilly faster than [`Keyword::validate`] method.
    fn is_valid(&self, instance: &Value) -> bool;
}

pub(crate) trait KeywordFactory: Send + Sync {
    fn init<'a>(
        &self,
        parent: &'a Map<String, Value>,
        schema: &'a Value,
        path: JsonPointer,
    ) -> Result<Box<dyn Keyword>, ValidationError<'a>>;
}

impl<F> KeywordFactory for F
where
    F: for<'a> Fn(
            &'a Map<String, Value>,
            &'a Value,
            JsonPointer,
        ) -> Result<Box<dyn Keyword>, ValidationError<'a>>
        + Send
        + Sync,
{
    fn init<'a>(
        &self,
        parent: &'a Map<String, Value>,
        schema: &'a Value,
        path: JsonPointer,
    ) -> Result<Box<dyn Keyword>, ValidationError<'a>> {
        self(parent, schema, path)
    }
}
