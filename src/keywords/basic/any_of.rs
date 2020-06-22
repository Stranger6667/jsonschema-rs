use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{CompilationError, ValidationError},
    keywords::{format_vec_of_validators, CompilationResult, Validators},
    validator::Validate,
};
use serde_json::{Map, Value};

pub struct AnyOfValidator {
    schemas: Vec<Validators>,
}

impl AnyOfValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        if let Value::Array(items) = schema {
            let mut schemas = Vec::with_capacity(items.len());
            for item in items {
                let validators = compile_validators(item, context)?;
                schemas.push(validators)
            }
            return Ok(Box::new(AnyOfValidator { schemas }));
        }
        Err(CompilationError::SchemaError)
    }
}

macro_rules! any_of_impl_is_valid {
    ($method_suffix:tt, $instance_type: ty) => {
        paste::item! {
            #[inline]
            fn [<is_valid_ $method_suffix>](
                &self,
                schema: &JSONSchema,
                instance: &Value,
                instance_value: $instance_type,
            ) -> bool {
                self.schemas.iter().any(move |validators| {
                    validators.iter().all(move |validator| {
                        validator.[<is_valid_ $method_suffix>](schema, instance, instance_value)
                    })
                })
            }
        }
    };
}

impl Validate for AnyOfValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::any_of(instance)
    }

    any_of_impl_is_valid!(array, &[Value]);
    any_of_impl_is_valid!(boolean, bool);
    any_of_impl_is_valid!(null, ());
    any_of_impl_is_valid!(number, f64);
    any_of_impl_is_valid!(object, &Map<String, Value>);
    any_of_impl_is_valid!(signed_integer, i64);
    any_of_impl_is_valid!(string, &str);
    any_of_impl_is_valid!(unsigned_integer, u64);
}
impl ToString for AnyOfValidator {
    fn to_string(&self) -> String {
        format!("anyOf: [{}]", format_vec_of_validators(&self.schemas))
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(AnyOfValidator::compile(schema, context))
}
