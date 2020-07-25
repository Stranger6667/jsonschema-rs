use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::{format_vec_of_validators, CompilationResult, Validators},
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct OneOfValidator {
    schemas: Vec<Validators>,
}

impl OneOfValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        if let Value::Array(items) = schema {
            let mut schemas = Vec::with_capacity(items.len());
            for item in items {
                schemas.push(compile_validators(item, context)?)
            }
            Ok(Box::new(OneOfValidator { schemas }))
        } else {
            Err(CompilationError::SchemaError)
        }
    }
}

macro_rules! one_of_impl_is_valid {
    ($method_suffix:tt, $instance_type: ty) => {
        paste::item! {
            #[inline]
            fn [<is_valid_ $method_suffix>](
                &self,
                schema: &JSONSchema,
                instance: &Value,
                instance_value: $instance_type,
            ) -> bool {
                let mut valid_schema_iterator = self.schemas
                    .iter()
                    .filter(|validators| {
                        validators
                            .iter()
                            .all(|validator| validator.[<is_valid_ $method_suffix>](schema, instance, instance_value))
                    });

                if valid_schema_iterator.next().is_some() {
                    // If one schema is valid we need to ensure that there are no other valid schemas
                    valid_schema_iterator.next().is_none()
                } else {
                    false
                }
            }
        }
    };
}
macro_rules! one_of_impl_validate {
    ($method_suffix:tt, $instance_type: ty) => {
        paste::item! {
            #[inline]
            fn [<validate_ $method_suffix>]<'a>(
                &self,
                schema: &'a JSONSchema,
                instance: &'a Value,
                instance_value: $instance_type,
            ) -> ErrorIterator<'a> {
                let mut valid_schema_iterator = self.schemas
                    .iter()
                    .filter(|validators| {
                        validators
                            .iter()
                            .all(|validator| validator.[<is_valid_ $method_suffix>](schema, instance, instance_value))
                    });

                if valid_schema_iterator.next().is_some() {
                    // If one schema is valid we need to ensure that there are no other valid schemas
                    if valid_schema_iterator.next().is_none() {
                        no_error()
                    } else {
                        error(ValidationError::one_of_multiple_valid(instance))
                    }
                } else {
                    error(ValidationError::one_of_not_valid(instance))
                }
            }
        }
    };
}
impl Validate for OneOfValidator {
    one_of_impl_is_valid!(array, &[Value]);
    one_of_impl_is_valid!(boolean, bool);
    one_of_impl_is_valid!(null, ());
    one_of_impl_is_valid!(number, f64);
    one_of_impl_is_valid!(object, &Map<String, Value>);
    one_of_impl_is_valid!(signed_integer, i64);
    one_of_impl_is_valid!(string, &str);
    one_of_impl_is_valid!(unsigned_integer, u64);

    one_of_impl_validate!(array, &[Value]);
    one_of_impl_validate!(boolean, bool);
    one_of_impl_validate!(null, ());
    one_of_impl_validate!(number, f64);
    one_of_impl_validate!(object, &Map<String, Value>);
    one_of_impl_validate!(signed_integer, i64);
    one_of_impl_validate!(string, &str);
    one_of_impl_validate!(unsigned_integer, u64);
}
impl ToString for OneOfValidator {
    fn to_string(&self) -> String {
        format!("oneOf: [{}]", format_vec_of_validators(&self.schemas))
    }
}

#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(OneOfValidator::compile(schema, context))
}
