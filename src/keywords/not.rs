use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::ValidationError,
    keywords::{format_validators, CompilationResult, Validators},
    validator::Validate,
};
use serde_json::{Map, Value};

pub struct NotValidator {
    // needed only for error representation
    original: Value,
    validators: Validators,
}

impl NotValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        Ok(Box::new(NotValidator {
            original: schema.clone(),
            validators: compile_validators(schema, context)?,
        }))
    }
}

macro_rules! not_impl_is_valid {
    ($method_suffix:tt, $instance_type: ty) => {
        paste::item! {
            #[inline]
            fn [<is_valid_ $method_suffix>](
                &self,
                schema: &JSONSchema,
                instance: &Value,
                instance_value: $instance_type,
            ) -> bool {
                !self
                .validators
                .iter()
                .all(|validator| validator.[<is_valid_ $method_suffix>](schema, instance, instance_value))
            }
        }
    };
}
impl Validate for NotValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::not(instance, self.original.clone())
    }

    not_impl_is_valid!(array, &[Value]);
    not_impl_is_valid!(boolean, bool);
    not_impl_is_valid!(null, ());
    not_impl_is_valid!(number, f64);
    not_impl_is_valid!(object, &Map<String, Value>);
    not_impl_is_valid!(signed_integer, i64);
    not_impl_is_valid!(string, &str);
    not_impl_is_valid!(unsigned_integer, u64);
}
impl ToString for NotValidator {
    fn to_string(&self) -> String {
        format!("not: {}", format_validators(&self.validators))
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(NotValidator::compile(schema, context))
}
