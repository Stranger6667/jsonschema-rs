use crate::{
    compilation::{compile_validators, context::CompilationContext},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{boolean::FalseValidator, CompilationResult},
    paths::{JSONPointer, JsonPointerNode},
    primitive_type::{PrimitiveType, PrimitiveTypesBitMap},
    schema_node::SchemaNode,
    validator::{format_validators, Validate},
};
use serde_json::{Map, Value};

pub(crate) struct AdditionalItemsObjectValidator {
    node: SchemaNode,
    items_count: usize,
}
impl AdditionalItemsObjectValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        items_count: usize,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let node = compile_validators(schema, context)?;
        Ok(Box::new(AdditionalItemsObjectValidator {
            node,
            items_count,
        }))
    }
}
impl Validate for AdditionalItemsObjectValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            items
                .iter()
                .skip(self.items_count)
                .all(|item| self.node.is_valid(item))
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
        if let Value::Array(items) = instance {
            let errors: Vec<_> = items
                .iter()
                .enumerate()
                .skip(self.items_count)
                .flat_map(|(idx, item)| self.node.validate(item, &instance_path.push(idx)))
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl core::fmt::Display for AdditionalItemsObjectValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "additionalItems: {}",
            format_validators(self.node.validators())
        )
    }
}

pub(crate) struct AdditionalItemsBooleanValidator {
    items_count: usize,
    schema_path: JSONPointer,
}
impl AdditionalItemsBooleanValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        items_count: usize,
        schema_path: JSONPointer,
    ) -> CompilationResult<'a> {
        Ok(Box::new(AdditionalItemsBooleanValidator {
            items_count,
            schema_path,
        }))
    }
}
impl Validate for AdditionalItemsBooleanValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            if items.len() > self.items_count {
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
        if let Value::Array(items) = instance {
            if items.len() > self.items_count {
                return error(ValidationError::additional_items(
                    self.schema_path.clone(),
                    instance_path.into(),
                    instance,
                    self.items_count,
                ));
            }
        }
        no_error()
    }
}
impl core::fmt::Display for AdditionalItemsBooleanValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "additionalItems: false".fmt(f)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    parent: &Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    if let Some(items) = parent.get("items") {
        match items {
            Value::Object(_) => None,
            Value::Array(items) => {
                let keyword_context = context.with_path("additionalItems");
                let items_count = items.len();
                match schema {
                    Value::Object(_) => Some(AdditionalItemsObjectValidator::compile(
                        schema,
                        items_count,
                        &keyword_context,
                    )),
                    Value::Bool(false) => Some(AdditionalItemsBooleanValidator::compile(
                        items_count,
                        keyword_context.into_pointer(),
                    )),
                    _ => None,
                }
            }
            Value::Bool(value) => {
                if *value {
                    None
                } else {
                    let schema_path = context.as_pointer_with("additionalItems");
                    Some(FalseValidator::compile(schema_path))
                }
            }
            _ => Some(Err(ValidationError::multiple_type_error(
                JSONPointer::default(),
                context.clone().into_pointer(),
                schema,
                PrimitiveTypesBitMap::new()
                    .add_type(PrimitiveType::Object)
                    .add_type(PrimitiveType::Array)
                    .add_type(PrimitiveType::Boolean),
            ))),
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"additionalItems": false, "items": false}), &json!([1]), "/additionalItems")]
    #[test_case(&json!({"additionalItems": false, "items": [{}]}), &json!([1, 2]), "/additionalItems")]
    #[test_case(&json!({"additionalItems": {"type": "string"}, "items": [{}]}), &json!([1, 2]), "/additionalItems/type")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
