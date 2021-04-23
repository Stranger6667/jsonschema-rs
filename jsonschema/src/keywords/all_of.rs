use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{CompilationError, ErrorIterator},
    keywords::{format_vec_of_validators, CompilationResult, InstancePath, Validators},
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct AllOfValidator {
    schemas: Vec<Validators>,
}

impl AllOfValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        if let Value::Array(items) = schema {
            let mut schemas = Vec::with_capacity(items.len());
            for item in items {
                let validators = compile_validators(item, context)?;
                schemas.push(validators)
            }
            Ok(Box::new(AllOfValidator { schemas }))
        } else {
            Err(CompilationError::SchemaError)
        }
    }
}

impl Validate for AllOfValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        self.schemas.iter().all(move |validators| {
            validators
                .iter()
                .all(move |validator| validator.is_valid(schema, instance))
        })
    }

    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        curr_instance_path: InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        let errors: Vec<_> = self
            .schemas
            .iter()
            .flat_map(move |validators| {
                let curr_instance_path = curr_instance_path.clone();
                validators.iter().flat_map(move |validator| {
                    validator.validate(schema, instance, curr_instance_path.clone())
                })
            })
            .collect();
        Box::new(errors.into_iter())
    }
}

impl ToString for AllOfValidator {
    fn to_string(&self) -> String {
        format!("allOf: [{}]", format_vec_of_validators(&self.schemas))
    }
}
#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(AllOfValidator::compile(schema, context))
}
