use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{no_error, ErrorIterator, ValidationError},
    keywords::{format_key_value_validators, ValidationResult, Validators},
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
        context: &'a CompilationContext,
    ) -> ValidationResult<'a> {
        match schema {
            Value::Object(map) => {
                let mut properties = Vec::with_capacity(map.len());
                for (key, subschema) in map {
                    properties.push((key.clone(), compile_validators(subschema, context)?));
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
                        let instance_path = instance_path.push(name.to_owned());
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
    context: &'a CompilationContext,
) -> Option<ValidationResult<'a>> {
    match parent.get("additionalProperties") {
        // This type of `additionalProperties` validator handles `properties` logic
        Some(Value::Bool(false)) | Some(Value::Object(_)) => None,
        _ => Some(PropertiesValidator::compile(schema, context)),
    }
}
