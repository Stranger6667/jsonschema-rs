use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{no_error, ErrorIterator},
    keywords::CompilationResult,
    paths::InstancePath,
    schema_node::SchemaNode,
    validator::{format_iter_of_validators, format_validators, PartialApplication, Validate},
};
use serde_json::{Map, Value};

pub(crate) struct ItemsArrayValidator {
    items: Vec<SchemaNode>,
}
impl ItemsArrayValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schemas: &'a [Value],
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let keyword_context = context.with_path("items");
        let mut items = Vec::with_capacity(schemas.len());
        for (idx, item) in schemas.iter().enumerate() {
            let item_context = keyword_context.with_path(idx);
            let validators = compile_validators(item, &item_context)?;
            items.push(validators)
        }
        Ok(Box::new(ItemsArrayValidator { items }))
    }
}
impl Validate for ItemsArrayValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            items
                .iter()
                .zip(self.items.iter())
                .all(move |(item, node)| node.is_valid(schema, item))
        } else {
            true
        }
    }

    #[allow(clippy::needless_collect)]
    fn validate<'a, 'b>(
        &self,
        schema: &'a JSONSchema,
        instance: &'b Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'b> {
        if let Value::Array(items) = instance {
            let errors: Vec<_> = items
                .iter()
                .zip(self.items.iter())
                .enumerate()
                .flat_map(move |(idx, (item, node))| {
                    node.validate(schema, item, &instance_path.push(idx))
                })
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl core::fmt::Display for ItemsArrayValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "items: [{}]",
            format_iter_of_validators(self.items.iter().map(SchemaNode::validators))
        )
    }
}

pub(crate) struct ItemsObjectValidator {
    node: SchemaNode,
}
impl ItemsObjectValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let keyword_context = context.with_path("items");
        let node = compile_validators(schema, &keyword_context)?;
        Ok(Box::new(ItemsObjectValidator { node }))
    }
}
impl Validate for ItemsObjectValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            items.iter().all(|i| self.node.is_valid(schema, i))
        } else {
            true
        }
    }

    #[allow(clippy::needless_collect)]
    fn validate<'a, 'b>(
        &self,
        schema: &'a JSONSchema,
        instance: &'b Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'b> {
        if let Value::Array(items) = instance {
            let errors: Vec<_> = items
                .iter()
                .enumerate()
                .flat_map(move |(idx, item)| {
                    self.node.validate(schema, item, &instance_path.push(idx))
                })
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }

    fn apply<'a>(
        &'a self,
        schema: &JSONSchema,
        instance: &Value,
        instance_path: &InstancePath,
    ) -> PartialApplication<'a> {
        if let Value::Array(items) = instance {
            let mut results = Vec::with_capacity(items.len());
            for (idx, item) in items.iter().enumerate() {
                let path = instance_path.push(idx);
                results.push(self.node.apply_rooted(schema, item, &path));
            }
            let mut output: PartialApplication = results.into_iter().collect();
            // Per draft 2020-12 section https://json-schema.org/draft/2020-12/json-schema-core.html#rfc.section.10.3.1.2
            // we must produce an annotation with a boolean value indicating whether the subschema
            // was applied to any positions in the underlying array. As we have not yet implemented
            // prefixItems this is true if there are any items in the instance.
            let schema_was_applied = !items.is_empty();
            output.annotate(serde_json::json! {schema_was_applied}.into());
            output
        } else {
            PartialApplication::valid_empty()
        }
    }
}

impl core::fmt::Display for ItemsObjectValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "items: {}", format_validators(self.node.validators()))
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    match schema {
        Value::Array(items) => Some(ItemsArrayValidator::compile(items, context)),
        Value::Object(_) => Some(ItemsObjectValidator::compile(schema, context)),
        Value::Bool(value) => {
            if *value {
                None
            } else {
                Some(ItemsObjectValidator::compile(schema, context))
            }
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
