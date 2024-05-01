use crate::{
    compilation::{compile_validators, context::CompilationContext},
    error::{no_error, ErrorIterator, ValidationError},
    paths::{JSONPointer, JsonPointerNode},
    primitive_type::PrimitiveType,
    schema_node::SchemaNode,
    validator::{format_iter_of_validators, PartialApplication, Validate},
};
use serde_json::{Map, Value};

use super::CompilationResult;

pub(crate) struct PrefixItemsValidator {
    schemas: Vec<SchemaNode>,
}

impl PrefixItemsValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        items: &'a [Value],
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let keyword_context = context.with_path("prefixItems");
        let mut schemas = Vec::with_capacity(items.len());
        for (idx, item) in items.iter().enumerate() {
            let item_context = keyword_context.with_path(idx);
            let validators = compile_validators(item, &item_context)?;
            schemas.push(validators)
        }
        Ok(Box::new(PrefixItemsValidator { schemas }))
    }
}

impl Validate for PrefixItemsValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            self.schemas
                .iter()
                .zip(items.iter())
                .all(|(n, i)| n.is_valid(i))
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
            let errors: Vec<_> = self
                .schemas
                .iter()
                .zip(items.iter())
                .enumerate()
                .flat_map(|(idx, (n, i))| n.validate(i, &instance_path.push(idx)))
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
            if !items.is_empty() {
                let validate_total = self.schemas.len();
                let mut results = Vec::with_capacity(validate_total);
                let mut max_index_applied = 0;
                for (idx, (schema_node, item)) in self.schemas.iter().zip(items.iter()).enumerate()
                {
                    let path = instance_path.push(idx);
                    results.push(schema_node.apply_rooted(item, &path));
                    max_index_applied = idx;
                }
                // Per draft 2020-12 section https://json-schema.org/draft/2020-12/json-schema-core.html#rfc.section.10.3.1.1
                // we must produce an annotation with the largest index of the underlying
                // array which the subschema was applied. The value MAY be a boolean true if
                // a subschema was applied to every index of the instance.
                let schema_was_applied: Value = if results.len() == items.len() {
                    true.into()
                } else {
                    max_index_applied.into()
                };
                let mut output: PartialApplication = results.into_iter().collect();
                output.annotate(schema_was_applied.into());
                return output;
            }
        }
        PartialApplication::valid_empty()
    }
}

impl core::fmt::Display for PrefixItemsValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "prefixItems: [{}]",
            format_iter_of_validators(self.schemas.iter().map(SchemaNode::validators))
        )
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    if let Value::Array(items) = schema {
        Some(PrefixItemsValidator::compile(items, context))
    } else {
        Some(Err(ValidationError::single_type_error(
            JSONPointer::default(),
            context.clone().into_pointer(),
            schema,
            PrimitiveType::Array,
        )))
    }
}

#[cfg(all(test, feature = "draft202012"))]
mod tests {
    use crate::{compilation::JSONSchema, tests_util};
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"prefixItems": [{"type": "integer"}, {"maximum": 5}]}), &json!(["string"]), "/prefixItems/0/type")]
    #[test_case(&json!({"prefixItems": [{"type": "integer"}, {"maximum": 5}]}), &json!([42, 42]), "/prefixItems/1/maximum")]
    #[test_case(&json!({"prefixItems": [{"type": "integer"}, {"maximum": 5}], "items": {"type": "boolean"}}), &json!([42, 1, 42]), "/items/type")]
    #[test_case(&json!({"prefixItems": [{"type": "integer"}, {"maximum": 5}], "items": {"type": "boolean"}}), &json!([42, 42, true]), "/prefixItems/1/maximum")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }

    #[test_case(&json!({"prefixItems": [{"type": "integer"}, {"maximum": 5}]}), "prefixItems: [{type: integer}, {maximum: 5}]")]
    fn debug_representation(schema: &Value, expected: &str) {
        let compiled = JSONSchema::compile(schema).unwrap();
        assert_eq!(
            format!("{:?}", compiled.node.validators().next().unwrap()),
            expected
        );
    }

    #[test_case{
        &json!({
            "type": "array",
            "prefixItems": [
                {
                    "type": "string"
                }
            ]
        }),
        &json!{[]},
        &json!({
            "valid": true,
            "annotations": []
        }); "valid prefixItems empty array"
    }]
    #[test_case{
        &json!({
            "type": "array",
            "prefixItems": [
                {
                    "type": "string"
                },
                {
                    "type": "number"
                }
            ]
        }),
        &json!{["string", 1]},
        &json!({
            "valid": true,
            "annotations": [
                {
                    "keywordLocation": "/prefixItems",
                    "instanceLocation": "",
                    "annotations": true
                },
            ]
        }); "prefixItems valid items"
    }]
    #[test_case{
        &json!({
            "type": "array",
            "prefixItems": [
                {
                    "type": "string"
                }
            ]
        }),
        &json!{["string", 1]},
        &json!({
            "valid": true,
            "annotations": [
                {
                    "keywordLocation": "/prefixItems",
                    "instanceLocation": "",
                    "annotations": 0
                },
            ]
        }); "prefixItems valid mixed items"
    }]
    #[test_case{
        &json!({
            "type": "array",
            "items": {
                "type": "number",
                "annotation": "value"
            },
            "prefixItems": [
                {
                    "type": "string"
                },
                {
                    "type": "boolean"
                }
            ]
        }),
        &json!{["string", true, 2, 3]},
        &json!({
            "valid": true,
            "annotations": [
                {
                    "keywordLocation": "/prefixItems",
                    "instanceLocation": "",
                    "annotations": 1
                },
                {
                    "keywordLocation": "/items",
                    "instanceLocation": "",
                    "annotations": true
                },
                {
                    "annotations": {"annotation": "value" },
                    "instanceLocation": "/2",
                    "keywordLocation": "/items"
                },
                {
                    "annotations": {"annotation": "value" },
                    "instanceLocation": "/3",
                    "keywordLocation": "/items"
                }
            ]
        }); "valid prefixItems with mixed items"
    }]
    fn test_basic_output(schema_json: &Value, instance: &Value, expected_output: &Value) {
        let schema = JSONSchema::options().compile(schema_json).unwrap();
        let output_json = serde_json::to_value(schema.apply(instance).basic()).unwrap();
        assert_eq!(&output_json, expected_output);
    }
}
