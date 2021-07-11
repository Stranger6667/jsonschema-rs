use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    paths::{InstancePath, JSONPointer},
    validator::{Validate, ValidatorBuf},
};
use serde_json::{Map, Value};

pub(crate) struct RequiredValidator {
    required: Vec<String>,
    schema_path: JSONPointer,
}

impl RequiredValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        items: &'a [Value],
        schema_path: JSONPointer,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let mut required = Vec::with_capacity(items.len());
        for item in items {
            match item {
                Value::String(string) => required.push(string.clone()),
                _ => return Err(ValidationError::schema(item)),
            }
        }
        Ok(context.add_validator(ValidatorBuf::new(RequiredValidator {
            required,
            schema_path,
        })))
    }
}

impl Validate for RequiredValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            self.required
                .iter()
                .all(|property_name| item.contains_key(property_name))
        } else {
            true
        }
    }

    fn validate<'a>(
        &self,
        _: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
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

impl core::fmt::Display for RequiredValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "required: [{}]", self.required.join(", "))
    }
}

pub(crate) struct SingleItemRequiredValidator {
    value: String,
    schema_path: JSONPointer,
}

impl SingleItemRequiredValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        value: &str,
        schema_path: JSONPointer,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        Ok(
            context.add_validator(ValidatorBuf::new(SingleItemRequiredValidator {
                value: value.to_string(),
                schema_path,
            })),
        )
    }
}

impl Validate for SingleItemRequiredValidator {
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if !self.is_valid(schema, instance) {
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

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            item.contains_key(&self.value)
        } else {
            true
        }
    }
}

impl core::fmt::Display for SingleItemRequiredValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "required: [{}]", self.value)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    let schema_path = context.as_pointer_with("required");
    compile_with_path(schema, schema_path, context)
}

#[inline]
pub(crate) fn compile_with_path<'a>(
    schema: &'a Value,
    schema_path: JSONPointer,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    // IMPORTANT: If this function will ever return `None`, adjust `dependencies.rs` accordingly
    match schema {
        Value::Array(items) => {
            if items.len() == 1 {
                if let Some(Value::String(item)) = items.iter().next() {
                    Some(SingleItemRequiredValidator::compile(
                        item,
                        schema_path,
                        context,
                    ))
                } else {
                    Some(Err(ValidationError::schema(schema)))
                }
            } else {
                Some(RequiredValidator::compile(items, schema_path, context))
            }
        }
        _ => Some(Err(ValidationError::schema(schema))),
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
