use crate::paths::{LazyLocation, Location};

use crate::{
    error::{error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use serde_json::Value;

pub(crate) struct FalseValidator {
    location: Location,
}
impl FalseValidator {
    #[inline]
    pub(crate) fn compile<'a>(location: Location) -> CompilationResult<'a> {
        Ok(Box::new(FalseValidator { location }))
    }
}
impl Validate for FalseValidator {
    fn is_valid(&self, _: &Value) -> bool {
        false
    }

    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &LazyLocation,
    ) -> ErrorIterator<'instance> {
        error(ValidationError::false_schema(
            self.location.clone(),
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
    fn location() {
        tests_util::assert_schema_location(&json!(false), &json!(1), "")
    }
}
