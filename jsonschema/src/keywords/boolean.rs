use crate::paths::InstancePath;

use crate::{
    compilation::JSONSchema,
    error::{error, ErrorIterator, ValidationError},
    keywords::ValidationResult,
    validator::Validate,
};
use serde_json::Value;

pub(crate) struct FalseValidator {}
impl FalseValidator {
    #[inline]
    pub(crate) fn compile<'a>() -> ValidationResult<'a> {
        Ok(Box::new(FalseValidator {}))
    }
}
impl Validate for FalseValidator {
    fn is_valid(&self, _: &JSONSchema, _: &Value) -> bool {
        false
    }

    fn validate<'a>(
        &self,
        _: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        error(ValidationError::false_schema(
            instance_path.into(),
            instance,
        ))
    }
}

impl ToString for FalseValidator {
    fn to_string(&self) -> String {
        "false".to_string()
    }
}
