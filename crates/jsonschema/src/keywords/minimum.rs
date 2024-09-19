use crate::{
    compiler,
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    paths::{JsonPointer, JsonPointerNode},
    primitive_type::PrimitiveType,
    validator::Validate,
};
use num_cmp::NumCmp;
use serde_json::{Map, Value};

pub(crate) struct MinimumU64Validator {
    limit: u64,
    limit_val: Value,
    schema_path: JsonPointer,
}
pub(crate) struct MinimumI64Validator {
    limit: i64,
    limit_val: Value,
    schema_path: JsonPointer,
}
pub(crate) struct MinimumF64Validator {
    limit: f64,
    limit_val: Value,
    schema_path: JsonPointer,
}

macro_rules! validate {
    ($validator: ty) => {
        impl Validate for $validator {
            fn validate<'instance>(
                &self,
                instance: &'instance Value,
                instance_path: &JsonPointerNode,
            ) -> ErrorIterator<'instance> {
                if self.is_valid(instance) {
                    no_error()
                } else {
                    error(ValidationError::minimum(
                        self.schema_path.clone(),
                        instance_path.into(),
                        instance,
                        self.limit_val.clone(),
                    )) // do not cast
                }
            }

            fn is_valid(&self, instance: &Value) -> bool {
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
    };
}

validate!(MinimumU64Validator);
validate!(MinimumI64Validator);

impl Validate for MinimumF64Validator {
    fn is_valid(&self, instance: &Value) -> bool {
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

    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if self.is_valid(instance) {
            no_error()
        } else {
            error(ValidationError::minimum(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
                self.limit_val.clone(),
            ))
        }
    }
}

#[inline]
pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    _: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    if let Value::Number(limit) = schema {
        let schema_path = ctx.as_pointer_with("minimum");
        if let Some(limit) = limit.as_u64() {
            Some(Ok(Box::new(MinimumU64Validator {
                limit,
                limit_val: schema.clone(),
                schema_path,
            })))
        } else if let Some(limit) = limit.as_i64() {
            Some(Ok(Box::new(MinimumI64Validator {
                limit,
                limit_val: schema.clone(),
                schema_path,
            })))
        } else {
            let limit = limit.as_f64().expect("Always valid");
            Some(Ok(Box::new(MinimumF64Validator {
                limit,
                limit_val: schema.clone(),
                schema_path,
            })))
        }
    } else {
        Some(Err(ValidationError::single_type_error(
            JsonPointer::default(),
            ctx.clone().into_pointer(),
            schema,
            PrimitiveType::Number,
        )))
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
