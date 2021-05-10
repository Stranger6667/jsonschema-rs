use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    paths::InstancePath,
    validator::Validate,
};
use num_cmp::NumCmp;
use serde_json::{Map, Value};

pub(crate) struct ExclusiveMinimumU64Validator {
    limit: u64,
}
pub(crate) struct ExclusiveMinimumI64Validator {
    limit: i64,
}
pub(crate) struct ExclusiveMinimumF64Validator {
    limit: f64,
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
                    error(ValidationError::exclusive_minimum(
                        instance_path.into(),
                        instance,
                        self.limit as f64,
                    ))
                }
            }

            fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
                if let Value::Number(item) = instance {
                    return if let Some(item) = item.as_u64() {
                        NumCmp::num_gt(item, self.limit)
                    } else if let Some(item) = item.as_i64() {
                        NumCmp::num_gt(item, self.limit)
                    } else {
                        let item = item.as_f64().expect("Always valid");
                        NumCmp::num_gt(item, self.limit)
                    };
                }
                true
            }
        }
        impl ToString for $validator {
            fn to_string(&self) -> String {
                format!("exclusiveMinimum: {}", self.limit)
            }
        }
    };
}

validate!(ExclusiveMinimumU64Validator);
validate!(ExclusiveMinimumI64Validator);

impl Validate for ExclusiveMinimumF64Validator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Number(item) = instance {
            return if let Some(item) = item.as_u64() {
                NumCmp::num_gt(item, self.limit)
            } else if let Some(item) = item.as_i64() {
                NumCmp::num_gt(item, self.limit)
            } else {
                let item = item.as_f64().expect("Always valid");
                NumCmp::num_gt(item, self.limit)
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
            error(ValidationError::exclusive_minimum(
                instance_path.into(),
                instance,
                self.limit,
            ))
        }
    }
}
impl ToString for ExclusiveMinimumF64Validator {
    fn to_string(&self) -> String {
        format!("exclusiveMinimum: {}", self.limit)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &mut CompilationContext,
) -> Option<CompilationResult<'a>> {
    if let Value::Number(limit) = schema {
        if let Some(limit) = limit.as_u64() {
            Some(Ok(Box::new(ExclusiveMinimumU64Validator { limit })))
        } else if let Some(limit) = limit.as_i64() {
            Some(Ok(Box::new(ExclusiveMinimumI64Validator { limit })))
        } else {
            let limit = limit.as_f64().expect("Always valid");
            Some(Ok(Box::new(ExclusiveMinimumF64Validator { limit })))
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

    #[test_case(&json!({"exclusiveMinimum": 1_u64 << 54}), &json!(1_u64 << 54))]
    #[test_case(&json!({"exclusiveMinimum": 1_i64 << 54}), &json!(1_i64 << 54))]
    #[test_case(&json!({"exclusiveMinimum": 1_u64 << 54}), &json!((1_u64 << 54) - 1))]
    #[test_case(&json!({"exclusiveMinimum": 1_i64 << 54}), &json!((1_i64 << 54) - 1))]
    fn is_not_valid(schema: &Value, instance: &Value) {
        tests_util::is_not_valid(schema, instance)
    }
}
