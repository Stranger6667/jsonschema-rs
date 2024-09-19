use crate::{
    compiler,
    error::{no_error, ErrorIterator},
    keywords::CompilationResult,
    node::SchemaNode,
    paths::JsonPointerNode,
    validator::{PartialApplication, Validate},
};
use serde_json::{Map, Value};

pub(crate) struct ItemsArrayValidator {
    items: Vec<SchemaNode>,
}
impl ItemsArrayValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        ctx: &compiler::Context,
        schemas: &'a [Value],
    ) -> CompilationResult<'a> {
        let kctx = ctx.with_path("items");
        let mut items = Vec::with_capacity(schemas.len());
        for (idx, item) in schemas.iter().enumerate() {
            let ictx = kctx.with_path(idx);
            let validators = compiler::compile(&ictx, ictx.as_resource_ref(item))?;
            items.push(validators)
        }
        Ok(Box::new(ItemsArrayValidator { items }))
    }
}
impl Validate for ItemsArrayValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            items
                .iter()
                .zip(self.items.iter())
                .all(move |(item, node)| node.is_valid(item))
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
                .zip(self.items.iter())
                .enumerate()
                .flat_map(move |(idx, (item, node))| node.validate(item, &instance_path.push(idx)))
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

pub(crate) struct ItemsObjectValidator {
    node: SchemaNode,
}

impl ItemsObjectValidator {
    #[inline]
    pub(crate) fn compile<'a>(ctx: &compiler::Context, schema: &'a Value) -> CompilationResult<'a> {
        let ctx = ctx.with_path("items");
        let node = compiler::compile(&ctx, ctx.as_resource_ref(schema))?;
        Ok(Box::new(ItemsObjectValidator { node }))
    }
}
impl Validate for ItemsObjectValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            items.iter().all(|i| self.node.is_valid(i))
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
                .flat_map(move |(idx, item)| self.node.validate(item, &instance_path.push(idx)))
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
        if let Value::Array(items) = instance {
            let mut results = Vec::with_capacity(items.len());
            for (idx, item) in items.iter().enumerate() {
                let path = instance_path.push(idx);
                results.push(self.node.apply_rooted(item, &path));
            }
            let mut output: PartialApplication = results.into_iter().collect();
            // Per draft 2020-12 section https://json-schema.org/draft/2020-12/json-schema-core.html#rfc.section.10.3.1.2
            // we must produce an annotation with a boolean value indicating whether the subschema
            // was applied to any positions in the underlying array. Since the struct
            // `ItemsObjectValidator` is not used when prefixItems is defined, this is true if
            // there are any items in the instance.
            let schema_was_applied = !items.is_empty();
            output.annotate(serde_json::json!(schema_was_applied).into());
            output
        } else {
            PartialApplication::valid_empty()
        }
    }
}

pub(crate) struct ItemsObjectSkipPrefixValidator {
    node: SchemaNode,
    skip_prefix: usize,
}

impl ItemsObjectSkipPrefixValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        skip_prefix: usize,
        ctx: &compiler::Context,
    ) -> CompilationResult<'a> {
        let ctx = ctx.with_path("items");
        let node = compiler::compile(&ctx, ctx.as_resource_ref(schema))?;
        Ok(Box::new(ItemsObjectSkipPrefixValidator {
            node,
            skip_prefix,
        }))
    }
}

impl Validate for ItemsObjectSkipPrefixValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            items
                .iter()
                .skip(self.skip_prefix)
                .all(|i| self.node.is_valid(i))
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
                .skip(self.skip_prefix)
                .enumerate()
                .flat_map(move |(idx, item)| {
                    self.node
                        .validate(item, &instance_path.push(idx + self.skip_prefix))
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
        if let Value::Array(items) = instance {
            let mut results = Vec::with_capacity(items.len().saturating_sub(self.skip_prefix));
            for (idx, item) in items.iter().enumerate().skip(self.skip_prefix) {
                let path = instance_path.push(idx);
                results.push(self.node.apply_rooted(item, &path));
            }
            let mut output: PartialApplication = results.into_iter().collect();
            // Per draft 2020-12 section https://json-schema.org/draft/2020-12/json-schema-core.html#rfc.section.10.3.1.2
            // we must produce an annotation with a boolean value indicating whether the subschema
            // was applied to any positions in the underlying array.
            let schema_was_applied = items.len() > self.skip_prefix;
            output.annotate(serde_json::json!(schema_was_applied).into());
            output
        } else {
            PartialApplication::valid_empty()
        }
    }
}

#[inline]
pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    parent: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    match schema {
        Value::Array(items) => Some(ItemsArrayValidator::compile(ctx, items)),
        Value::Object(_) | Value::Bool(false) => {
            if let Some(Value::Array(prefix_items)) = parent.get("prefixItems") {
                return Some(ItemsObjectSkipPrefixValidator::compile(
                    schema,
                    prefix_items.len(),
                    ctx,
                ));
            }
            Some(ItemsObjectValidator::compile(ctx, schema))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"items": false}), &json!([1]), "/items")]
    #[test_case(&json!({"items": {"type": "string"}}), &json!([1]), "/items/type")]
    #[test_case(&json!({"items": [{"type": "string"}]}), &json!([1]), "/items/0/type")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
