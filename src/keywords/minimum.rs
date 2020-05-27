use super::{CompilationResult, Validate};
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
};
use num_cmp::NumCmp;
use serde_json::{Map, Value};

pub struct MinimumU64Validator {
    limit: u64,
}
pub struct MinimumI64Validator {
    limit: i64,
}
pub struct MinimumF64Validator {
    limit: f64,
}

macro_rules! validate {
    ($validator: ty) => {
        impl Validate for $validator {
            fn validate<'a>(
                &self,
                schema: &'a JSONSchema,
                instance: &'a Value,
            ) -> ErrorIterator<'a> {
                if self.is_valid(schema, instance) {
                    no_error()
                } else {
                    error(ValidationError::minimum(instance, self.limit as f64)) // do not cast
                }
            }

            fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
                if let Value::Number(item) = instance {
                    return if let Some(item) = item.as_u64() {
                        !NumCmp::num_lt(item, self.limit)
                    } else if let Some(item) = item.as_i64() {
                        !NumCmp::num_lt(item, self.limit)
                    } else {
                        let item = item.as_f64().expect("Always valid");
                        !NumCmp::num_lt(item, self.limit)
                    };
                }
                true
            }

            fn name(&self) -> String {
                format!("minimum: {}", self.limit)
            }
        }
    };
}

validate!(MinimumU64Validator);
validate!(MinimumI64Validator);
validate!(MinimumF64Validator);

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    if let Value::Number(limit) = schema {
        return if let Some(limit) = limit.as_u64() {
            Some(Ok(Box::new(MinimumU64Validator { limit })))
        } else if let Some(limit) = limit.as_i64() {
            Some(Ok(Box::new(MinimumI64Validator { limit })))
        } else {
            let limit = limit.as_f64().expect("Always valid");
            Some(Ok(Box::new(MinimumF64Validator { limit })))
        };
    }
    Some(Err(CompilationError::SchemaError))
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!({"minimum": 1u64 << 54}), json!(1u64 << 54 - 1))]
    #[test_case(json!({"minimum": 1i64 << 54}), json!(1i64 << 54 - 1))]
    fn is_not_valid(schema: Value, instance: Value) {
        tests_util::is_not_valid(schema, instance)
    }
}
