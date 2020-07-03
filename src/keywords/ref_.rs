use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{error, ErrorIterator, ValidationError},
    keywords::{CompilationResult, Validators},
    validator::Validate,
};
use parking_lot::RwLock;
use serde_json::{Map, Value};
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
    pub(crate) fn compile(reference: &str, context: &CompilationContext) -> CompilationResult {
        let reference = context.build_url(reference)?;
        Ok(Box::new(RefValidator {
            reference,
            validators: RwLock::new(None),
        }))
    }

    /// Ensure that validators are built and built once.
    #[inline]
    fn ensure_validators<'a>(&self, schema: &'a JSONSchema) -> Result<(), ValidationError<'a>> {
        if self.validators.read().is_none() {
            let (scope, resolved) = schema.resolver.resolve_fragment(
                schema.context.config.draft(),
                &self.reference,
                schema.schema,
            )?;
            let context = CompilationContext::new(scope, Cow::Borrowed(&schema.context.config));
            let validators = compile_validators(&resolved, &context)?;

            // Inject the validators into self.validators
            *self.validators.write() = Some(validators);
        }
        Ok(())
    }
}

macro_rules! ref_impl_is_valid {
    ($method_suffix:tt, $instance_type: ty) => {
        paste::item! {
            #[inline]
            fn [<is_valid_ $method_suffix>](
                &self,
                schema: &JSONSchema,
                instance: &Value,
                instance_value: $instance_type,
            ) -> bool {
                if self.ensure_validators(schema).is_err() {
                    false
                } else {
                    self.validators
                        .read()
                        .as_ref()
                        .expect("ensure_validators guarantees the presence of the validators")
                        .iter()
                        .all(move |validator| {
                            validator.[<is_valid_ $method_suffix>](schema, instance, instance_value)
                        })
                }
            }
        }
    };
}
macro_rules! ref_impl_validate {
    ($method_suffix:tt, $instance_type: ty) => {
        paste::item! {
            #[inline]
            fn [<validate_ $method_suffix>]<'a>(
                &self,
                schema: &'a JSONSchema,
                instance: &'a Value,
                instance_value: $instance_type,
            ) -> ErrorIterator<'a> {
                if let Err(err) = self.ensure_validators(schema) {
                    error(err)
                } else {
                    Box::new(
                        self.validators
                            .read()
                            .as_ref()
                            .expect("ensure_validators guarantees the presence of the validators")
                            .iter()
                            .flat_map(move |validator| {
                                validator.[<validate_ $method_suffix>](schema, instance, instance_value)
                            })
                            .collect::<Vec<_>>()
                            .into_iter(),
                    )
                }
            }
        }
    };
}

impl Validate for RefValidator {
    ref_impl_is_valid!(array, &[Value]);
    ref_impl_is_valid!(boolean, bool);
    ref_impl_is_valid!(null, ());
    ref_impl_is_valid!(number, f64);
    ref_impl_is_valid!(object, &Map<String, Value>);
    ref_impl_is_valid!(signed_integer, i64);
    ref_impl_is_valid!(string, &str);
    ref_impl_is_valid!(unsigned_integer, u64);

    ref_impl_validate!(array, &'a [Value]);
    ref_impl_validate!(boolean, bool);
    ref_impl_validate!(null, ());
    ref_impl_validate!(number, f64);
    ref_impl_validate!(object, &'a Map<String, Value>);
    ref_impl_validate!(signed_integer, i64);
    ref_impl_validate!(string, &'a str);
    ref_impl_validate!(unsigned_integer, u64);
}
impl ToString for RefValidator {
    fn to_string(&self) -> String {
        format!("$ref: {}", self.reference)
    }
}

#[inline]
pub(crate) fn compile(
    _: &Value,
    reference: &str,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(RefValidator::compile(reference, context))
}
