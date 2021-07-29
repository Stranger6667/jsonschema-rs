use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{error, ErrorIterator},
    keywords::CompilationResult,
    paths::{InstancePath, JSONPointer},
    schema_node::SchemaNode,
    validator::Validate,
};
use parking_lot::RwLock;
use serde_json::Value;
use url::Url;

pub(crate) struct RefValidator {
    reference: Url,
    /// Precomputed validators.
    /// They are behind a RwLock as is not possible to compute them
    /// at compile time without risking infinite loops of references
    /// and at the same time during validation we iterate over shared
    /// references (&self) and not owned references (&mut self).
    sub_nodes: RwLock<Option<SchemaNode>>,
    schema_path: JSONPointer,
}

impl RefValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        reference: &str,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let reference = context.build_url(reference)?;
        Ok(Box::new(RefValidator {
            reference,
            sub_nodes: RwLock::new(None),
            schema_path: context.schema_path.clone().into(),
        }))
    }
}

impl Validate for RefValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Some(sub_nodes) = self.sub_nodes.read().as_ref() {
            return sub_nodes.is_valid(schema, instance);
        }
        if let Ok((scope, resolved)) = schema
            .resolver
            .resolve_fragment(schema.draft(), &self.reference)
        {
            let context = CompilationContext::new(scope.into(), schema.config());
            if let Ok(node) = compile_validators(&resolved, &context) {
                let result = node.is_valid(schema, instance);
                *self.sub_nodes.write() = Some(node);
                return result;
            }
        };
        false
    }

    fn validate<'a, 'b>(
        &self,
        schema: &'a JSONSchema,
        instance: &'b Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'b> {
        if let Some(node) = self.sub_nodes.read().as_ref() {
            return Box::new(
                node.validate(schema, instance, instance_path)
                    .collect::<Vec<_>>()
                    .into_iter(),
            );
        }
        match schema
            .resolver
            .resolve_fragment(schema.draft(), &self.reference)
        {
            Ok((scope, resolved)) => {
                let context = CompilationContext::new(scope.into(), schema.config());
                match compile_validators(&resolved, &context) {
                    Ok(node) => {
                        let result = Box::new(
                            node.err_iter(schema, instance, instance_path)
                                .map(move |mut error| {
                                    let schema_path = self.schema_path.clone();
                                    error.schema_path =
                                        schema_path.extend_with(error.schema_path.as_slice());
                                    error
                                })
                                .collect::<Vec<_>>()
                                .into_iter(),
                        );
                        *self.sub_nodes.write() = Some(node);
                        result
                    }
                    Err(err) => error(err.into_owned()),
                }
            }
            Err(err) => error(err.into_owned()),
        }
    }
}

impl core::fmt::Display for RefValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "$ref: {}", self.reference)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Value,
    reference: &'a str,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(RefValidator::compile(reference, context))
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::json;

    #[test]
    fn schema_path() {
        tests_util::assert_schema_path(
            &json!({"properties": {"foo": {"$ref": "#/definitions/foo"}}, "definitions": {"foo": {"type": "string"}}}),
            &json!({"foo": 42}),
            "/properties/foo/type",
        )
    }
}
