use crate::{
    compiler,
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    paths::{JsonPointer, JsonPointerNode},
    primitive_type::PrimitiveType,
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct RequiredValidator {
    required: Vec<String>,
    schema_path: JsonPointer,
}

impl RequiredValidator {
    #[inline]
    pub(crate) fn compile(items: &[Value], schema_path: JsonPointer) -> CompilationResult {
        let mut required = Vec::with_capacity(items.len());
        for item in items {
            match item {
                Value::String(string) => required.push(string.clone()),
                _ => {
                    return Err(ValidationError::single_type_error(
                        JsonPointer::default(),
                        schema_path,
                        item,
                        PrimitiveType::String,
                    ))
                }
            }
        }
        Ok(Box::new(RequiredValidator {
            required,
            schema_path,
        }))
    }
}

impl Validate for RequiredValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            self.required
                .iter()
                .all(|property_name| item.contains_key(property_name))
        } else {
            true
        }
    }

    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if let Value::Object(item) = instance {
            let mut errors = vec![];
            for property_name in &self.required {
                if !item.contains_key(property_name) {
                    errors.push(ValidationError::required(
                        self.schema_path.clone(),
                        instance_path.into(),
                        instance,
                        // Value enum is needed for proper string escaping
                        Value::String(property_name.clone()),
                    ));
                }
            }
            if !errors.is_empty() {
                return Box::new(errors.into_iter());
            }
        }
        no_error()
    }
}

pub(crate) struct SingleItemRequiredValidator {
    value: String,
    schema_path: JsonPointer,
}

impl SingleItemRequiredValidator {
    #[inline]
    pub(crate) fn compile(value: &str, schema_path: JsonPointer) -> CompilationResult {
        Ok(Box::new(SingleItemRequiredValidator {
            value: value.to_string(),
            schema_path,
        }))
    }
}

impl Validate for SingleItemRequiredValidator {
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if !self.is_valid(instance) {
            return error(ValidationError::required(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
                // Value enum is needed for proper string escaping
                Value::String(self.value.clone()),
            ));
        }
        no_error()
    }

    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            item.contains_key(&self.value)
        } else {
            true
        }
    }
}

#[inline]
pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    _: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    let schema_path = ctx.as_pointer_with("required");
    compile_with_path(schema, schema_path)
}

#[inline]
pub(crate) fn compile_with_path(
    schema: &Value,
    schema_path: JsonPointer,
) -> Option<CompilationResult> {
    // IMPORTANT: If this function will ever return `None`, adjust `dependencies.rs` accordingly
    match schema {
        Value::Array(items) => {
            if items.len() == 1 {
                let item = &items[0];
                if let Value::String(item) = item {
                    Some(SingleItemRequiredValidator::compile(item, schema_path))
                } else {
                    Some(Err(ValidationError::single_type_error(
                        JsonPointer::default(),
                        schema_path,
                        item,
                        PrimitiveType::String,
                    )))
                }
            } else {
                Some(RequiredValidator::compile(items, schema_path))
            }
        }
        _ => Some(Err(ValidationError::single_type_error(
            JsonPointer::default(),
            schema_path,
            schema,
            PrimitiveType::Array,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"required": ["a"]}), &json!({}), "/required")]
    #[test_case(&json!({"required": ["a", "b"]}), &json!({}), "/required")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
