use crate::{
    compiler,
    error::{no_error, ErrorIterator, ValidationError},
    keywords::{boolean::FalseValidator, CompilationResult},
    node::SchemaNode,
    paths::{LazyLocation, Location},
    primitive_type::{PrimitiveType, PrimitiveTypesBitMap},
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct AdditionalItemsObjectValidator {
    node: SchemaNode,
    items_count: usize,
}
impl AdditionalItemsObjectValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        ctx: &compiler::Context,
        schema: &'a Value,
        items_count: usize,
    ) -> CompilationResult<'a> {
        let node = compiler::compile(ctx, ctx.as_resource_ref(schema))?;
        Ok(Box::new(AdditionalItemsObjectValidator {
            node,
            items_count,
        }))
    }
}
impl Validate for AdditionalItemsObjectValidator {
    #[allow(clippy::needless_collect)]
    fn iter_errors<'i>(&self, instance: &'i Value, location: &LazyLocation) -> ErrorIterator<'i> {
        if let Value::Array(items) = instance {
            let errors: Vec<_> = items
                .iter()
                .enumerate()
                .skip(self.items_count)
                .flat_map(|(idx, item)| self.node.iter_errors(item, &location.push(idx)))
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }

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

    fn validate<'i>(
        &self,
        instance: &'i Value,
        location: &LazyLocation,
    ) -> Result<(), ValidationError<'i>> {
        if let Value::Array(items) = instance {
            for (idx, item) in items.iter().enumerate().skip(self.items_count) {
                self.node.validate(item, &location.push(idx))?;
            }
        }
        Ok(())
    }
}

pub(crate) struct AdditionalItemsBooleanValidator {
    items_count: usize,
    location: Location,
}
impl AdditionalItemsBooleanValidator {
    #[inline]
    pub(crate) fn compile<'a>(items_count: usize, location: Location) -> CompilationResult<'a> {
        Ok(Box::new(AdditionalItemsBooleanValidator {
            items_count,
            location,
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

    fn validate<'i>(
        &self,
        instance: &'i Value,
        location: &LazyLocation,
    ) -> Result<(), ValidationError<'i>> {
        if let Value::Array(items) = instance {
            if items.len() > self.items_count {
                return Err(ValidationError::additional_items(
                    self.location.clone(),
                    location.into(),
                    instance,
                    self.items_count,
                ));
            }
        }
        Ok(())
    }
}

#[inline]
pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    parent: &Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    if let Some(items) = parent.get("items") {
        match items {
            Value::Object(_) => None,
            Value::Array(items) => {
                let kctx = ctx.new_at_location("additionalItems");
                let items_count = items.len();
                match schema {
                    Value::Object(_) => Some(AdditionalItemsObjectValidator::compile(
                        &kctx,
                        schema,
                        items_count,
                    )),
                    Value::Bool(false) => Some(AdditionalItemsBooleanValidator::compile(
                        items_count,
                        kctx.location().clone(),
                    )),
                    _ => None,
                }
            }
            Value::Bool(value) => {
                if *value {
                    None
                } else {
                    let location = ctx.location().join("additionalItems");
                    Some(FalseValidator::compile(location))
                }
            }
            _ => Some(Err(ValidationError::multiple_type_error(
                Location::new(),
                ctx.location().clone(),
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
    use referencing::Draft;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"additionalItems": false, "items": false}), &json!([1]), "/additionalItems")]
    #[test_case(&json!({"additionalItems": false, "items": [{}]}), &json!([1, 2]), "/additionalItems")]
    #[test_case(&json!({"additionalItems": {"type": "string"}, "items": [{}]}), &json!([1, 2]), "/additionalItems/type")]
    fn location(schema: &Value, instance: &Value, expected: &str) {
        let validator = crate::options()
            .with_draft(Draft::Draft7)
            .build(schema)
            .expect("Invalid schema");
        let error = validator.validate(instance).expect_err("Should fail");
        assert_eq!(error.schema_path.as_str(), expected);
    }
}
