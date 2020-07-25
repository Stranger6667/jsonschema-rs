use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{CompilationError, ErrorIterator},
    keywords::{format_vec_of_validators, CompilationResult, Validators},
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

macro_rules! all_of_impl_is_valid {
    ($method_suffix:tt, $instance_type: ty) => {
        paste::item! {
            #[inline]
            fn [<is_valid_ $method_suffix>](
                &self,
                schema: &JSONSchema,
                instance: &Value,
                instance_value: $instance_type,
            ) -> bool {
                self.schemas.iter().all(move |validators| {
                    validators.iter().all(move |validator| {
                        validator.[<is_valid_ $method_suffix>](schema, instance, instance_value)
                    })
                })
            }
        }
    };
}
macro_rules! all_of_impl_validate {
    ($method_suffix:tt, $instance_type: ty) => {
        paste::item! {
            #[inline]
            fn [<validate_ $method_suffix>]<'a>(
                &self,
                schema: &'a JSONSchema,
                instance: &'a Value,
                instance_value: $instance_type,
            ) -> ErrorIterator<'a> {
                Box::new(
                    self.schemas
                        .iter()
                        .flat_map(move |validators| {
                            validators.iter().flat_map(move |validator| {
                                validator.[<validate_ $method_suffix>](schema, instance, instance_value)
                            })
                        })
                        .collect::<Vec<_>>()
                        .into_iter(),
                )
            }
        }
    };
}

impl Validate for AllOfValidator {
    all_of_impl_is_valid!(array, &[Value]);
    all_of_impl_is_valid!(boolean, bool);
    all_of_impl_is_valid!(null, ());
    all_of_impl_is_valid!(number, f64);
    all_of_impl_is_valid!(object, &Map<String, Value>);
    all_of_impl_is_valid!(signed_integer, i64);
    all_of_impl_is_valid!(string, &str);
    all_of_impl_is_valid!(unsigned_integer, u64);

    all_of_impl_validate!(array, &'a [Value]);
    all_of_impl_validate!(boolean, bool);
    all_of_impl_validate!(null, ());
    all_of_impl_validate!(number, f64);
    all_of_impl_validate!(object, &'a Map<String, Value>);
    all_of_impl_validate!(signed_integer, i64);
    all_of_impl_validate!(string, &'a str);
    all_of_impl_validate!(unsigned_integer, u64);
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
