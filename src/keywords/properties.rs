use super::{CompilationResult, ValidationResult};
use super::{Validate, Validators};
use crate::context::CompilationContext;
use crate::error::CompilationError;
use crate::validator::{compile_validators, JSONSchema};
use serde_json::{Map, Value};

pub struct PropertiesValidator<'a> {
    properties: Vec<(&'a String, Validators<'a>)>,
}

impl<'a> PropertiesValidator<'a> {
    pub(crate) fn compile(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        match schema {
            Value::Object(map) => {
                let mut properties = Vec::with_capacity(map.len());
                for (key, subschema) in map {
                    properties.push((key, compile_validators(subschema, context)?));
                }
                Ok(Box::new(PropertiesValidator { properties }))
            }
            _ => Err(CompilationError::SchemaError),
        }
    }
}

impl<'a> Validate<'a> for PropertiesValidator<'a> {
    fn validate(&self, schema: &JSONSchema, instance: &Value) -> ValidationResult {
        if let Value::Object(item) = instance {
            for (name, validators) in self.properties.iter() {
                if let Some(item) = item.get(*name) {
                    for validator in validators {
                        validator.validate(schema, item)?
                    }
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> String {
        //        "".to_string()
        format!("<properties: {:?}>", self.properties)
    }
}
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(PropertiesValidator::compile(schema, context))
}
