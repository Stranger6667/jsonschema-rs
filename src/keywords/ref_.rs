use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{error, ErrorIterator, ValidationError},
    keywords::{CompilationResult, Validators},
    validator::Validate,
};
use parking_lot::RwLock;
use serde_json::Value;
use url::Url;

pub struct RefValidator {
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
    pub(crate) fn compile(reference: &str, context: &CompilationContext) -> CompilationResult {
        let reference = context.build_url(reference)?;
        Ok(Box::new(RefValidator {
            reference,
            validators: RwLock::new(None),
        }))
    }

    /// Ensure that validators are built and built once.
    fn ensure_validators<'a>(&self, schema: &'a JSONSchema) -> Result<(), ValidationError<'a>> {
        if self.validators.read().is_none() {
            let (scope, resolved) =
                schema
                    .resolver
                    .resolve_fragment(schema.draft, &self.reference, schema.schema)?;
            let context = CompilationContext::new(scope, schema.draft);
            let validators = compile_validators(&resolved, &context)?;

            // Inject the validators into self.validators
            *self.validators.write() = Some(validators);
        }
        Ok(())
    }
}

impl Validate for RefValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Err(err) = self.ensure_validators(schema) {
            error(err)
        } else {
            Box::new(
                self.validators
                    .read()
                    .as_ref()
                    .expect("ensure_validators guarantees the presence of the validators")
                    .iter()
                    .flat_map(move |validator| validator.validate(schema, instance))
                    .collect::<Vec<_>>()
                    .into_iter(),
            )
        }
    }

    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if self.ensure_validators(schema).is_err() {
            false
        } else {
            self.validators
                .read()
                .as_ref()
                .expect("ensure_validators guarantees the presence of the validators")
                .iter()
                .all(move |validator| validator.is_valid(schema, instance))
        }
    }

    fn name(&self) -> String {
        format!("$ref: {}", self.reference)
    }
}

#[inline]
pub fn compile(
    _: &Value,
    reference: &str,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(RefValidator::compile(reference, context))
}
