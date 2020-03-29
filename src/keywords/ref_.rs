use super::CompilationResult;
use super::Validate;
use crate::context::CompilationContext;
use crate::error::{error, ErrorIterator};
use crate::validator::{compile_validators, JSONSchema};
use serde_json::Value;
use url::Url;

pub struct RefValidator {
    reference: Url,
}

impl RefValidator {
    pub(crate) fn compile(reference: &str, context: &CompilationContext) -> CompilationResult {
        let reference = context.build_url(reference)?;
        Ok(Box::new(RefValidator { reference }))
    }
}

impl Validate for RefValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        match schema
            .resolver
            .resolve_fragment(schema.draft, &self.reference, schema.schema)
        {
            Ok((scope, resolved)) => {
                let context = CompilationContext::new(scope, schema.draft);
                match compile_validators(&resolved, &context) {
                    Ok(validators) => Box::new(
                        validators
                            .into_iter()
                            .flat_map(move |validator| validator.validate(schema, instance)),
                    ),
                    Err(e) => error(e.into()),
                }
            }
            Err(e) => error(e),
        }
    }

    fn name(&self) -> String {
        format!("<ref: {}>", self.reference)
    }
}
pub(crate) fn compile(
    _: &Value,
    reference: &str,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(RefValidator::compile(reference, &context))
}
