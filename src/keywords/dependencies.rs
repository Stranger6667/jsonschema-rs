use super::CompilationResult;
use super::{Validate, Validators};
use crate::context::CompilationContext;
use crate::error::{no_error, CompilationError, ErrorIterator};
use crate::keywords::required::RequiredValidator;
use crate::validator::compile_validators;
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct DependenciesValidator {
    dependencies: Vec<(String, Validators)>,
}

impl DependenciesValidator {
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        match schema.as_object() {
            Some(map) => {
                let mut dependencies = Vec::with_capacity(map.len());
                for (key, subschema) in map {
                    let s = match subschema {
                        Value::Array(_) => vec![RequiredValidator::compile(subschema)?],
                        _ => compile_validators(subschema, context)?,
                    };
                    dependencies.push((key.clone(), s))
                }
                Ok(Box::new(DependenciesValidator { dependencies }))
            }
            None => Err(CompilationError::SchemaError),
        }
    }
}

impl Validate for DependenciesValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            let errors: Vec<_> = self
                .dependencies
                .iter()
                .filter(|(property, _)| item.contains_key(property))
                .flat_map(move |(_, validators)| {
                    validators
                        .iter()
                        .flat_map(move |validator| validator.validate(schema, instance))
                })
                .collect();
            // TODO. custom error message for "required" case
            return Box::new(errors.into_iter());
        }
        no_error()
    }

    fn name(&self) -> String {
        format!("<dependencies: {:?}>", self.dependencies)
    }
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(DependenciesValidator::compile(schema, context))
}
