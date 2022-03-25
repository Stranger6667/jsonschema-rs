use crate::compilation::context::CompilationContext;
use crate::compilation::options::KeywordDefinition;
use crate::error::no_error;
use crate::keywords::CompilationResult;
use crate::paths::{InstancePath, JSONPointer, PathChunk};
use crate::validator::Validate;
use crate::{ErrorIterator, JSONSchema, ValidationError};
use serde_json::{Map, Value};
use std::fmt::{Display, Formatter};

pub(crate) struct CustomKeywordSchemaValidator {
    schema_path: JSONPointer,
    keyword_schema: Option<Value>,
    json_schema: Option<JSONSchema>,
}

impl CustomKeywordSchemaValidator {
    #[inline]
    pub(crate) fn compile(
        _: &Value,
        schema_path: JSONPointer,
        keyword_schema: Value,
    ) -> CompilationResult {
        let mut validator = CustomKeywordSchemaValidator {
            schema_path,
            keyword_schema: Some(keyword_schema),
            json_schema: None,
        };
        validator.compile_schema().map_err(|e| e.into_owned())?;
        Ok(Box::new(validator))
    }

    fn compile_schema(&mut self) -> Result<(), ValidationError> {
        if let Some(keyword_schema) = &self.keyword_schema {
            self.json_schema = Some(JSONSchema::compile(keyword_schema)?);
        }
        Ok(())
    }
}

impl Display for CustomKeywordSchemaValidator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl Validate for CustomKeywordSchemaValidator {
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'instance> {
        if let Some(schema) = &self.json_schema {
            return match schema.validate(&instance) {
                Ok(_) => no_error(),
                Err(mut error_iter) => {
                    let errors: Vec<ValidationError> = error_iter
                        .map(|validation_error| {
                            let mapped_error = ValidationError {
                                instance_path: JSONPointer::from(instance_path)
                                    .extend_with(validation_error.instance_path.as_slice()),
                                schema_path: self
                                    .schema_path
                                    .extend_with(validation_error.schema_path.as_slice()),
                                ..validation_error
                            };
                            mapped_error.into_owned()
                        })
                        .collect();
                    Box::new(errors.into_iter())
                }
            };
        }

        no_error()
    }

    fn is_valid(&self, instance: &Value) -> bool {
        if let Some(schema) = &self.json_schema {
            return schema.is_valid(instance);
        }

        true
    }
}

#[inline]
pub(crate) fn compile_custom_keyword_validator<'a>(
    _: &Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
    keyword: impl Into<PathChunk>,
    keyword_definition: KeywordDefinition,
) -> CompilationResult<'a> {
    let schema_path = context.as_pointer_with(keyword);
    match keyword_definition {
        KeywordDefinition::Schema(keyword_schema) => {
            CustomKeywordSchemaValidator::compile(schema, schema_path, keyword_schema)
        }
    }
}
