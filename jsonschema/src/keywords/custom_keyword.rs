use crate::compilation::context::CompilationContext;
use crate::compilation::options::CustomKeywordValidator;
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

pub(crate) fn compile_custom_keyword_validator<'a>(
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
