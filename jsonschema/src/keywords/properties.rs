use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{no_error, ErrorIterator, ValidationError},
    keywords::{format_key_value_validators, CompilationResult, Validators},
    paths::InstancePath,
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct PropertiesValidator {
    properties: Vec<(String, Validators)>,
}

impl PropertiesValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        match schema {
            Value::Object(map) => {
                let context = context.with_path("properties".to_string());
                let mut properties = Vec::with_capacity(map.len());
                for (key, subschema) in map {
                    let property_context = context.with_path(key.clone());
                    properties.push((
                        key.clone(),
                        compile_validators(subschema, &property_context)?,
                    ));
                }
                Ok(Box::new(PropertiesValidator { properties }))
            }
            _ => Err(ValidationError::schema(schema)),
        }
    }
}

impl Validate for PropertiesValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            self.properties.iter().all(move |(name, validators)| {
                let option = item.get(name);
                option.into_iter().all(move |item| {
                    validators
                        .iter()
                        .all(move |validator| validator.is_valid(schema, item))
                })
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
                .properties
                .iter()
                .flat_map(move |(name, validators)| {
                    let option = item.get(name);
                    option.into_iter().flat_map(move |item| {
                        let instance_path = instance_path.push(name.clone());
                        validators.iter().flat_map(move |validator| {
                            validator.validate(schema, item, &instance_path)
                        })
                    })
                })
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl ToString for PropertiesValidator {
    fn to_string(&self) -> String {
        format!(
            "properties: {{{}}}",
            format_key_value_validators(&self.properties)
        )
    }
}

#[inline]
pub(crate) fn compile<'a>(
    parent: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    match parent.get("additionalProperties") {
        // This type of `additionalProperties` validator handles `properties` logic
        Some(Value::Bool(false)) | Some(Value::Object(_)) => None,
        _ => Some(PropertiesValidator::compile(schema, context)),
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::json;

    #[test]
    fn schema_path() {
        tests_util::assert_schema_path(
            &json!({"properties": {"foo": {"properties": {"bar": {"required": ["spam"]}}}}}),
            &json!({"foo": {"bar": {}}}),
            "/properties/foo/properties/bar/required",
        )
    }
}
