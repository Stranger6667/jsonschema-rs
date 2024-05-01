use crate::{
    compilation::{compile_validators, context::CompilationContext},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    paths::{JSONPointer, JsonPointerNode},
    schema_node::SchemaNode,
    validator::{format_validators, PartialApplication, Validate},
};
use serde_json::{Map, Value};

pub(crate) struct PropertyNamesObjectValidator {
    node: SchemaNode,
}

impl PropertyNamesObjectValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let keyword_context = context.with_path("propertyNames");
        Ok(Box::new(PropertyNamesObjectValidator {
            node: compile_validators(schema, &keyword_context)?,
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
impl core::fmt::Display for PropertyNamesObjectValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "propertyNames: {}",
            format_validators(self.node.validators())
        )
    }
}

pub(crate) struct PropertyNamesBooleanValidator {
    schema_path: JSONPointer,
}

impl PropertyNamesBooleanValidator {
    #[inline]
    pub(crate) fn compile<'a>(context: &CompilationContext) -> CompilationResult<'a> {
        let schema_path = context.as_pointer_with("propertyNames");
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

impl core::fmt::Display for PropertyNamesBooleanValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "propertyNames: false".fmt(f)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    match schema {
        Value::Object(_) => Some(PropertyNamesObjectValidator::compile(schema, context)),
        Value::Bool(false) => Some(PropertyNamesBooleanValidator::compile(context)),
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
