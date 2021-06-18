use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{no_error, ErrorIterator, ValidationError},
    keywords::{format_key_value_validators, required, CompilationResult, Validators},
    paths::InstancePath,
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct DependenciesValidator {
    dependencies: Vec<(String, Validators)>,
}

impl DependenciesValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        if let Value::Object(map) = schema {
            let keyword_context = context.with_path("dependencies");
            let mut dependencies = Vec::with_capacity(map.len());
            for (key, subschema) in map {
                let item_context = keyword_context.with_path(key.to_string());
                let s = match subschema {
                    Value::Array(_) => {
                        vec![required::compile_with_path(
                            subschema,
                            (&keyword_context.schema_path).into(),
                        )
                        .expect("The required validator compilation does not return None")?]
                    }
                    _ => compile_validators(subschema, &item_context)?,
                };
                dependencies.push((key.clone(), s))
            }
            Ok(Box::new(DependenciesValidator { dependencies }))
        } else {
            Err(ValidationError::schema(schema))
        }
    }
}

impl Validate for DependenciesValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            self.dependencies
                .iter()
                .filter(|(property, _)| item.contains_key(property))
                .all(move |(_, validators)| {
                    validators
                        .iter()
                        .all(move |validator| validator.is_valid(schema, instance))
                })
        } else {
            true
        }
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            let errors: Vec<_> = self
                .dependencies
                .iter()
                .filter(|(property, _)| item.contains_key(property))
                .flat_map(move |(_, validators)| {
                    validators.iter().flat_map(move |validator| {
                        validator.validate(schema, instance, instance_path)
                    })
                })
                .collect();
            // TODO. custom error message for "required" case
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl ToString for DependenciesValidator {
    fn to_string(&self) -> String {
        format!(
            "dependencies: {{{}}}",
            format_key_value_validators(&self.dependencies)
        )
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(DependenciesValidator::compile(schema, context))
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"dependencies": {"bar": ["foo"]}}), &json!({"bar": 1}), "/dependencies")]
    #[test_case(&json!({"dependencies": {"bar": {"type": "string"}}}), &json!({"bar": 1}), "/dependencies/bar/type")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
