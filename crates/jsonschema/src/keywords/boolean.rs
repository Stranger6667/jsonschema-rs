use crate::paths::{JsonPointer, JsonPointerNode};

use crate::{
    error::{error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::Value;

pub(crate) struct FalseValidator {
    schema_path: JsonPointer,
}
impl FalseValidator {
    #[inline]
    pub(crate) fn compile<'a>(schema_path: JsonPointer) -> CompilationResult<'a> {
        Ok(Box::new(FalseValidator { schema_path }))
    }
}
impl Validate for FalseValidator {
    fn is_valid(&self, _: &Value) -> bool {
        false
    }

    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        error(ValidationError::false_schema(
            self.schema_path.clone(),
            instance_path.into(),
            instance,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::json;

    #[test]
    fn schema_path() {
        tests_util::assert_schema_path(&json!(false), &json!(1), "")
    }
}
