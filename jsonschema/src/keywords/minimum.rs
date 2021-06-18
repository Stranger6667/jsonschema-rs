use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    paths::{InstancePath, JSONPointer},
    validator::Validate,
};
use num_cmp::NumCmp;
use serde_json::{Map, Value};

pub(crate) struct MinimumU64Validator {
    limit: u64,
    schema_path: JSONPointer,
}
pub(crate) struct MinimumI64Validator {
    limit: i64,
    schema_path: JSONPointer,
}
pub(crate) struct MinimumF64Validator {
    limit: f64,
    schema_path: JSONPointer,
}

macro_rules! validate {
    ($validator: ty) => {
        impl Validate for $validator {
            fn validate<'a>(
                &self,
                schema: &'a JSONSchema,
                instance: &'a Value,
                instance_path: &InstancePath,
            ) -> ErrorIterator<'a> {
                if self.is_valid(schema, instance) {
                    no_error()
                } else {
                    error(ValidationError::minimum(
                        self.schema_path.clone(),
                        instance_path.into(),
                        instance,
                        self.limit as f64,
                    )) // do not cast
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
        }
        impl ToString for $validator {
            fn to_string(&self) -> String {
                format!("minimum: {}", self.limit)
            }
        }
    };
}

validate!(MinimumU64Validator);
validate!(MinimumI64Validator);

impl Validate for MinimumF64Validator {
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

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::minimum(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
                self.limit,
            ))
        }
    }
}
impl ToString for MinimumF64Validator {
    fn to_string(&self) -> String {
        format!("minimum: {}", self.limit)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    if let Value::Number(limit) = schema {
        let schema_path = context.as_pointer_with("minimum");
        if let Some(limit) = limit.as_u64() {
            Some(Ok(Box::new(MinimumU64Validator { limit, schema_path })))
        } else if let Some(limit) = limit.as_i64() {
            Some(Ok(Box::new(MinimumI64Validator { limit, schema_path })))
        } else {
            let limit = limit.as_f64().expect("Always valid");
            Some(Ok(Box::new(MinimumF64Validator { limit, schema_path })))
        }
    } else {
        Some(Err(ValidationError::schema(schema)))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"minimum": 1_u64 << 54}), &json!((1_u64 << 54) - 1))]
    #[test_case(&json!({"minimum": 1_i64 << 54}), &json!((1_i64 << 54) - 1))]
    fn is_not_valid(schema: &Value, instance: &Value) {
        tests_util::is_not_valid(schema, instance)
    }

    #[test_case(&json!({"minimum": 5}), &json!(1), "/minimum")]
    #[test_case(&json!({"minimum": 6}), &json!(1), "/minimum")]
    #[test_case(&json!({"minimum": 7}), &json!(1), "/minimum")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
