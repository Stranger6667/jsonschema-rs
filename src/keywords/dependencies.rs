use super::{CompilationResult, ValidationResult};
use super::{Validate, Validators};
use crate::context::CompilationContext;
use crate::error::CompilationError;
use crate::keywords::required::RequiredValidator;
use crate::validator::compile_validators;
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct DependenciesValidator<'a> {
    dependencies: Vec<(&'a String, Validators<'a>)>,
}

impl<'a> DependenciesValidator<'a> {
    pub(crate) fn compile(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        match schema.as_object() {
            Some(map) => {
                let mut dependencies = Vec::with_capacity(map.len());
                for (key, subschema) in map {
                    let s = match subschema {
                        Value::Array(_) => vec![RequiredValidator::compile(subschema)?],
                        _ => compile_validators(subschema, context)?,
                    };
                    dependencies.push((key, s))
                }
                Ok(Box::new(DependenciesValidator { dependencies }))
            }
            None => Err(CompilationError::SchemaError),
        }
    }
}

impl<'a> Validate<'a> for DependenciesValidator<'a> {
    fn validate(&self, schema: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Object(item) = instance {
            for (property, validators) in self.dependencies.iter() {
                if !item.contains_key(*property) {
                    continue;
                }
                // TODO. custom error message for "required" case
                for validator in validators.iter() {
                    validator.validate(schema, instance)?
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> String {
        format!("<dependencies: {:?}>", self.dependencies)
    }
}

pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(DependenciesValidator::compile(schema, context))
}
