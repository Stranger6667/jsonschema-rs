use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{CompilationError, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use num_cmp::NumCmp;
use serde_json::{Map, Value};

pub struct ExclusiveMaximumU64Validator {
    limit: u64,
}
pub struct ExclusiveMaximumI64Validator {
    limit: i64,
}
pub struct ExclusiveMaximumF64Validator {
    limit: f64,
}

macro_rules! validate {
    ($validator: ty) => {
        impl Validate for $validator {
            fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
                #[allow(trivial_numeric_casts)]
                ValidationError::exclusive_maximum(instance, self.limit as f64)
            }

            fn name(&self) -> String {
                format!("exclusiveMaximum: {}", self.limit)
            }

            #[inline]
            fn is_valid_number(&self, _: &JSONSchema, _: &Value, instance_value: f64) -> bool {
                NumCmp::num_lt(instance_value, self.limit)
            }
            #[inline]
            fn is_valid_signed_integer(
                &self,
                _: &JSONSchema,
                _: &Value,
                instance_value: i64,
            ) -> bool {
                NumCmp::num_lt(instance_value, self.limit)
            }
            #[inline]
            fn is_valid_unsigned_integer(
                &self,
                _: &JSONSchema,
                _: &Value,
                instance_value: u64,
            ) -> bool {
                NumCmp::num_lt(instance_value, self.limit)
            }
        }
    };
}

validate!(ExclusiveMaximumU64Validator);
validate!(ExclusiveMaximumI64Validator);
validate!(ExclusiveMaximumF64Validator);

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    if let Value::Number(limit) = schema {
        return if let Some(limit) = limit.as_u64() {
            Some(Ok(Box::new(ExclusiveMaximumU64Validator { limit })))
        } else if let Some(limit) = limit.as_i64() {
            Some(Ok(Box::new(ExclusiveMaximumI64Validator { limit })))
        } else {
            let limit = limit.as_f64().expect("Always valid");
            Some(Ok(Box::new(ExclusiveMaximumF64Validator { limit })))
        };
    }
    Some(Err(CompilationError::SchemaError))
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!({"exclusiveMaximum": 1u64 << 54}), json!(1u64 << 54))]
    #[test_case(json!({"exclusiveMaximum": 1i64 << 54}), json!(1i64 << 54))]
    #[test_case(json!({"exclusiveMaximum": 1u64 << 54}), json!(1u64 << 54 + 1))]
    #[test_case(json!({"exclusiveMaximum": 1i64 << 54}), json!(1i64 << 54 + 1))]
    fn is_not_valid(schema: Value, instance: Value) {
        tests_util::is_not_valid(schema, instance)
    }
}
