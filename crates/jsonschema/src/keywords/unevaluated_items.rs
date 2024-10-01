use serde_json::{Map, Value};

use crate::{
    compiler,
    error::no_error,
    node::SchemaNode,
    paths::{JsonPointer, JsonPointerNode},
    validator::Validate,
    ErrorIterator, ValidationError,
};

use super::CompilationResult;

pub(crate) struct UnevaluatedItemsValidator {
    schema_path: JsonPointer,
    unevaluated: SchemaNode,
    prefix_items: Option<usize>,
    //additional: Option<UnevaluatedSubvalidator>,
    //properties: Option<PropertySubvalidator>,
    //patterns: Option<PatternSubvalidator>,
    //conditional: Option<Box<ConditionalSubvalidator>>,
    //dependent: Option<DependentSchemaSubvalidator>,
    //reference: Option<ReferenceSubvalidator>,
    //subschemas: Option<Vec<SubschemaSubvalidator>>,
}

impl UnevaluatedItemsValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        ctx: &compiler::Context,
        parent: &'a Map<String, Value>,
        schema: &'a Value,
    ) -> CompilationResult<'a> {
        let kctx = ctx.with_path("unevaluatedItems");
        let schema_path = ctx.as_pointer_with("unevaluatedItems");
        if let Some(prefix_items) = parent.get("prefixItems") {}
        // TODO:
        //  - compile `$ref`, `$dynamicRef`, `prefixItems`, `if` / `then` / `else`, `contains`,
        //  `allOf`, `oneOf`, `anyOf`

        Ok(Box::new(UnevaluatedItemsValidator {
            unevaluated: compiler::compile(&kctx, kctx.as_resource_ref(schema))?,
            schema_path,
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
            // TODO: Use idx
            let mut unevaluated = vec![];
            for item in items {
                if !self.unevaluated.is_valid(item) {
                    unevaluated.push(item.to_string());
                }
            }
            if !unevaluated.is_empty() {
                return Box::new(
                    vec![ValidationError::unevaluated_items(
                        self.schema_path.clone(),
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
    schema_path: JsonPointer,
}

impl FalseValidator {
    #[inline]
    pub(crate) fn compile<'a>(schema_path: JsonPointer) -> CompilationResult<'a> {
        Ok(Box::new(FalseValidator { schema_path }))
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
                        self.schema_path.clone(),
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
            ctx.as_pointer_with("unevaluatedItems"),
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
