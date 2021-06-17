use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{error, ErrorIterator},
    keywords::{CompilationResult, Validators},
    paths::InstancePath,
    validator::Validate,
};
use parking_lot::RwLock;
use serde_json::Value;
use std::borrow::Cow;
use url::Url;

pub(crate) struct RefValidator {
    reference: Url,
    /// Precomputed validators.
    /// They are behind a RwLock as is not possible to compute them
    /// at compile time without risking infinite loops of references
    /// and at the same time during validation we iterate over shared
    /// references (&self) and not owned references (&mut self).
    validators: RwLock<Option<Validators>>,
}

impl RefValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        reference: &str,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let reference = context.build_url(reference)?;
        Ok(Box::new(RefValidator {
            reference,
            validators: RwLock::new(None),
        }))
    }
}

impl Validate for RefValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Some(validators) = self.validators.read().as_ref() {
            return validators
                .iter()
                .all(move |validator| validator.is_valid(schema, instance));
        }
        if let Ok((scope, resolved)) = schema.resolver.resolve_fragment(
            schema.context.config.draft(),
            &self.reference,
            schema.schema,
        ) {
            let context = CompilationContext::new(scope, Cow::Borrowed(&schema.context.config));
            if let Ok(validators) = compile_validators(&resolved, &context) {
                let result = validators
                    .iter()
                    .all(move |validator| validator.is_valid(schema, instance));
                *self.validators.write() = Some(validators);
                return result;
            }
        };
        false
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Some(validators) = self.validators.read().as_ref() {
            return Box::new(
                validators
                    .iter()
                    .flat_map(move |validator| validator.validate(schema, instance, instance_path))
                    .collect::<Vec<_>>()
                    .into_iter(),
            );
        }
        match schema.resolver.resolve_fragment(
            schema.context.config.draft(),
            &self.reference,
            schema.schema,
        ) {
            Ok((scope, resolved)) => {
                let context = CompilationContext::new(scope, Cow::Borrowed(&schema.context.config));
                match compile_validators(&resolved, &context) {
                    Ok(validators) => {
                        let result = Box::new(
                            validators
                                .iter()
                                .flat_map(move |validator| {
                                    validator.validate(schema, instance, instance_path)
                                })
                                .collect::<Vec<_>>()
                                .into_iter(),
                        );
                        *self.validators.write() = Some(validators);
                        result
                    }
                    Err(err) => error(err.into_owned()),
                }
            }
            Err(err) => error(err),
        }
    }
}

impl ToString for RefValidator {
    fn to_string(&self) -> String {
        format!("$ref: {}", self.reference)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Value,
    reference: &'a str,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(RefValidator::compile(reference, context))
}
