use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{no_error, CompilationError, ErrorIterator},
    keywords::{format_key_value_validators, CompilationResult, Validators},
    validator::Validate,
};
use serde_json::{Map, Value};

pub struct PropertiesValidator {
    properties: Vec<(String, Validators)>,
}

impl PropertiesValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        match schema {
            Value::Object(map) => {
                let mut properties = Vec::with_capacity(map.len());
                for (key, subschema) in map {
                    properties.push((key.clone(), compile_validators(subschema, context)?));
                }
                Ok(Box::new(PropertiesValidator { properties }))
            }
            _ => Err(CompilationError::SchemaError),
        }
    }
}

impl Validate for PropertiesValidator {
    #[inline]
    fn is_valid_object(
        &self,
        schema: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        self.properties.iter().all(|(name, validators)| {
            instance_value.get(name).into_iter().all(|sub_value| {
                validators
                    .iter()
                    .all(|validator| validator.is_valid(schema, sub_value))
            })
        })
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(instance_value) = instance {
            self.is_valid_object(schema, instance, instance_value)
        } else {
            true
        }
    }

    #[inline]
    fn validate_object<'a>(
        &self,
        schema: &'a JSONSchema,
        _: &'a Value,
        instance_value: &'a Map<String, Value>,
    ) -> ErrorIterator<'a> {
        Box::new(
            self.properties
                .iter()
                .flat_map(move |(name, validators)| {
                    instance_value
                        .get(name)
                        .into_iter()
                        .flat_map(move |sub_value| {
                            validators
                                .iter()
                                .flat_map(move |validator| validator.validate(schema, sub_value))
                        })
                })
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(instance_value) = instance {
            self.validate_object(schema, instance, instance_value)
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
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(PropertiesValidator::compile(schema, context))
}
