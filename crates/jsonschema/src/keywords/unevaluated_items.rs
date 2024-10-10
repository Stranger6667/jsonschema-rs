use serde_json::{Map, Value};

use crate::{
    compiler,
    error::no_error,
    node::SchemaNode,
    paths::{JsonPointerNode, Location},
    validator::Validate,
    ErrorIterator, ValidationError,
};

use super::CompilationResult;

pub(crate) struct UnevaluatedItemsValidator {
    unevaluated: SchemaNode,
}

impl UnevaluatedItemsValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        ctx: &compiler::Context,
        _parent: &'a Map<String, Value>,
        schema: &'a Value,
    ) -> CompilationResult<'a> {
        let kctx = ctx.new_at_location("unevaluatedItems");
        Ok(Box::new(UnevaluatedItemsValidator {
            unevaluated: compiler::compile(&kctx, kctx.as_resource_ref(schema))?,
        }))
    }
}

impl Validate for UnevaluatedItemsValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            for item in items {
                if !self.unevaluated.is_valid(item) {
                    return false;
                }
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
            let mut unevaluated = vec![];
            for item in items {
                if !self.unevaluated.is_valid(item) {
                    unevaluated.push(item.to_string());
                }
            }
            if !unevaluated.is_empty() {
                return Box::new(
                    vec![ValidationError::unevaluated_items(
                        self.unevaluated.location().clone(),
                        instance_path.into(),
                        instance,
                        items.iter().map(|item| item.to_string()).collect(),
                    )]
                    .into_iter(),
                );
            }
        }
        no_error()
    }
}

pub(crate) struct FalseValidator {
    location: Location,
}

impl FalseValidator {
    #[inline]
    pub(crate) fn compile<'a>(location: Location) -> CompilationResult<'a> {
        Ok(Box::new(FalseValidator { location }))
    }
}

impl Validate for FalseValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Array(item) = instance {
            item.is_empty()
        } else {
            true
        }
    }

    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if let Value::Array(items) = instance {
            if !items.is_empty() {
                return Box::new(
                    vec![ValidationError::unevaluated_items(
                        self.location.clone(),
                        instance_path.into(),
                        instance,
                        items.iter().map(|item| item.to_string()).collect(),
                    )]
                    .into_iter(),
                );
            }
        }
        no_error()
    }
}

pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    parent: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    match schema.as_bool() {
        Some(true) => None,
        Some(false) => Some(FalseValidator::compile(
            ctx.location().join("unevaluatedItems"),
        )),
        None => {
            if !schema["items"].is_null() {
                None
            } else {
                Some(UnevaluatedItemsValidator::compile(ctx, parent, schema))
            }
        }
    }
}
