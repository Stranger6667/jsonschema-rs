use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{no_error, CompilationError, ErrorIterator},
    keywords::{
        basic::required::RequiredValidator, format_key_value_validators, CompilationResult,
        Validators,
    },
    validator::Validate,
};
use serde_json::{Map, Value};

pub struct DependenciesValidator {
    dependencies: Vec<(String, Validators)>,
}

impl DependenciesValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        if let Value::Object(map) = schema {
            let mut dependencies = Vec::with_capacity(map.len());
            for (key, subschema) in map {
                let s = match subschema {
                    Value::Array(_) => vec![RequiredValidator::compile(subschema)?],
                    _ => compile_validators(subschema, context)?,
                };
                dependencies.push((key.clone(), s))
            }
            return Ok(Box::new(DependenciesValidator { dependencies }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for DependenciesValidator {
    #[inline]
    fn is_valid_object(
        &self,
        schema: &JSONSchema,
        instance: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        self.dependencies
            .iter()
            .filter(|(property, _)| instance_value.contains_key(property))
            .all(move |(_, validators)| {
                validators.iter().all(move |validator| {
                    validator.is_valid_object(schema, instance, instance_value)
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
        instance: &'a Value,
        instance_value: &'a Map<String, Value>,
    ) -> ErrorIterator<'a> {
        // TODO. custom error message for "required" case
        Box::new(
            self.dependencies
                .iter()
                .filter(|(property, _)| instance_value.contains_key(property))
                .flat_map(move |(_, validators)| {
                    validators.iter().flat_map(move |validator| {
                        validator.validate_object(schema, instance, instance_value)
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
impl ToString for DependenciesValidator {
    fn to_string(&self) -> String {
        format!(
            "dependencies: {{{}}}",
            format_key_value_validators(&self.dependencies)
        )
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(DependenciesValidator::compile(schema, context))
}
