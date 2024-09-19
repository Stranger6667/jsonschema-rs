use crate::{
    compiler,
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    node::SchemaNode,
    paths::{JsonPointer, JsonPointerNode},
    validator::{PartialApplication, Validate},
};
use serde_json::{Map, Value};

pub(crate) struct PropertyNamesObjectValidator {
    node: SchemaNode,
}

impl PropertyNamesObjectValidator {
    #[inline]
    pub(crate) fn compile<'a>(ctx: &compiler::Context, schema: &'a Value) -> CompilationResult<'a> {
        let ctx = ctx.with_path("propertyNames");
        Ok(Box::new(PropertyNamesObjectValidator {
            node: compiler::compile(&ctx, ctx.as_resource_ref(schema))?,
        }))
    }
}

impl Validate for PropertyNamesObjectValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Object(item) = &instance {
            item.keys().all(move |key| {
                let wrapper = Value::String(key.to_string());
                self.node.is_valid(&wrapper)
            })
        } else {
            true
        }
    }

    #[allow(clippy::needless_collect)]
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if let Value::Object(item) = &instance {
            let errors: Vec<_> = item
                .keys()
                .flat_map(move |key| {
                    let wrapper = Value::String(key.to_string());
                    let errors: Vec<_> = self
                        .node
                        .validate(&wrapper, instance_path)
                        .map(|error| {
                            ValidationError::property_names(
                                error.schema_path.clone(),
                                instance_path.into(),
                                instance,
                                error.into_owned(),
                            )
                        })
                        .collect();
                    errors.into_iter()
                })
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }

    fn apply<'a>(
        &'a self,
        instance: &Value,
        instance_path: &JsonPointerNode,
    ) -> PartialApplication<'a> {
        if let Value::Object(item) = instance {
            item.keys()
                .map(|key| {
                    let wrapper = Value::String(key.to_string());
                    self.node.apply_rooted(&wrapper, instance_path)
                })
                .collect()
        } else {
            PartialApplication::valid_empty()
        }
    }
}

pub(crate) struct PropertyNamesBooleanValidator {
    schema_path: JsonPointer,
}

impl PropertyNamesBooleanValidator {
    #[inline]
    pub(crate) fn compile<'a>(ctx: &compiler::Context) -> CompilationResult<'a> {
        let schema_path = ctx.as_pointer_with("propertyNames");
        Ok(Box::new(PropertyNamesBooleanValidator { schema_path }))
    }
}

impl Validate for PropertyNamesBooleanValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            if !item.is_empty() {
                return false;
            }
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
            error(ValidationError::false_schema(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
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
    match schema {
        Value::Object(_) => Some(PropertyNamesObjectValidator::compile(ctx, schema)),
        Value::Bool(false) => Some(PropertyNamesBooleanValidator::compile(ctx)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"propertyNames": false}), &json!({"foo": 1}), "/propertyNames")]
    #[test_case(&json!({"propertyNames": {"minLength": 2}}), &json!({"f": 1}), "/propertyNames/minLength")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
