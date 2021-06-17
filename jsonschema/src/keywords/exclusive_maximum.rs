use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::ValidationResult,
    paths::InstancePath,
    validator::Validate,
};
use num_cmp::NumCmp;
use serde_json::{Map, Value};

pub(crate) struct ExclusiveMaximumU64Validator {
    limit: u64,
}
pub(crate) struct ExclusiveMaximumI64Validator {
    limit: i64,
}
pub(crate) struct ExclusiveMaximumF64Validator {
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
                    error(ValidationError::exclusive_maximum(
                        instance_path.into(),
                        instance,
                        self.limit as f64,
                    ))
                }
            }

            fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
                if let Value::Number(item) = instance {
                    if let Some(item) = item.as_u64() {
                        NumCmp::num_lt(item, self.limit)
                    } else if let Some(item) = item.as_i64() {
                        NumCmp::num_lt(item, self.limit)
                    } else {
                        let item = item.as_f64().expect("Always valid");
                        NumCmp::num_lt(item, self.limit)
                    }
                } else {
                    true
                }
            }
        }
        impl ToString for $validator {
            fn to_string(&self) -> String {
                format!("exclusiveMaximum: {}", self.limit)
            }
        }
    };
}

validate!(ExclusiveMaximumU64Validator);
validate!(ExclusiveMaximumI64Validator);

impl Validate for ExclusiveMaximumF64Validator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Number(item) = instance {
            if let Some(item) = item.as_u64() {
                NumCmp::num_lt(item, self.limit)
            } else if let Some(item) = item.as_i64() {
                NumCmp::num_lt(item, self.limit)
            } else {
                let item = item.as_f64().expect("Always valid");
                NumCmp::num_lt(item, self.limit)
            }
        } else {
            true
        }
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
            error(ValidationError::exclusive_maximum(
                instance_path.into(),
                instance,
                self.limit,
            ))
        }
    }
}
impl ToString for ExclusiveMaximumF64Validator {
    fn to_string(&self) -> String {
        format!("exclusiveMaximum: {}", self.limit)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    _: &CompilationContext,
) -> Option<ValidationResult<'a>> {
    if let Value::Number(limit) = schema {
        if let Some(limit) = limit.as_u64() {
            Some(Ok(Box::new(ExclusiveMaximumU64Validator { limit })))
        } else if let Some(limit) = limit.as_i64() {
            Some(Ok(Box::new(ExclusiveMaximumI64Validator { limit })))
        } else {
            let limit = limit.as_f64().expect("Always valid");
            Some(Ok(Box::new(ExclusiveMaximumF64Validator { limit })))
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

    #[test_case(&json!({"exclusiveMaximum": 1u64 << 54}), &json!(1u64 << 54))]
    #[test_case(&json!({"exclusiveMaximum": 1i64 << 54}), &json!(1i64 << 54))]
    #[test_case(&json!({"exclusiveMaximum": 1u64 << 54}), &json!((1u64 << 54) + 1))]
    #[test_case(&json!({"exclusiveMaximum": 1i64 << 54}), &json!((1i64 << 54) + 1))]
    fn is_not_valid(schema: &Value, instance: &Value) {
        tests_util::is_not_valid(schema, instance)
    }
}
