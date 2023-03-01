use crate::{
    compilation::{compile_validators, context::CompilationContext},
    error::{error, ErrorIterator},
    keywords::CompilationResult,
    paths::{InstancePath, JSONPointer},
    primitive_type::PrimitiveType,
    resolver::Resolver,
    schema_node::SchemaNode,
    validator::Validate,
    CompilationOptions, Draft, ValidationError,
};
use parking_lot::RwLock;
use serde_json::{Map, Value};
use std::sync::Arc;
use url::Url;

pub(crate) struct RefValidator {
    original_reference: String,
    reference: Url,
    /// Precomputed validators.
    /// They are behind a RwLock as is not possible to compute them
    /// at compile time without risking infinite loops of references
    /// and at the same time during validation we iterate over shared
    /// references (&self) and not owned references (&mut self).
    sub_nodes: RwLock<Option<SchemaNode>>,
    schema_path: JSONPointer,
    config: Arc<CompilationOptions>,
    pub(crate) resolver: Arc<Resolver>,
}

impl RefValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        reference: &str,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        Ok(Box::new(RefValidator {
            original_reference: reference.to_string(),
            reference: context.build_url(reference)?,
            sub_nodes: RwLock::new(None),
            schema_path: context.schema_path.clone().into(),
            config: Arc::clone(&context.config),
            resolver: Arc::clone(&context.resolver),
        }))
    }
}

impl Validate for RefValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Some(sub_nodes) = self.sub_nodes.read().as_ref() {
            return sub_nodes.is_valid(instance);
        }
        if let Ok((scope, resolved)) = self.resolver.resolve_fragment(
            self.config.draft(),
            &self.reference,
            &self.original_reference,
        ) {
            let context = CompilationContext::new(
                scope.into(),
                Arc::clone(&self.config),
                Arc::clone(&self.resolver),
            );
            if let Ok(node) = compile_validators(&resolved, &context) {
                let result = node.is_valid(instance);
                *self.sub_nodes.write() = Some(node);
                return result;
            }
        };
        false
    }

    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'instance> {
        if let Some(node) = self.sub_nodes.read().as_ref() {
            return Box::new(
                node.validate(instance, instance_path)
                    .collect::<Vec<_>>()
                    .into_iter(),
            );
        }
        match self.resolver.resolve_fragment(
            self.config.draft(),
            &self.reference,
            &self.original_reference,
        ) {
            Ok((scope, resolved)) => {
                let context = CompilationContext::new(
                    scope.into(),
                    Arc::clone(&self.config),
                    Arc::clone(&self.resolver),
                );
                match compile_validators(&resolved, &context) {
                    Ok(node) => {
                        let result = Box::new(
                            node.err_iter(instance, instance_path)
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
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    Some(
        schema
            .as_str()
            .ok_or_else(|| {
                ValidationError::single_type_error(
                    JSONPointer::default(),
                    context.clone().into_pointer(),
                    schema,
                    PrimitiveType::String,
                )
            })
            .and_then(|reference| RefValidator::compile(reference, context)),
    )
}

pub(crate) const fn supports_adjacent_validation(draft: Draft) -> bool {
    match draft {
        #[cfg(feature = "draft201909")]
        Draft::Draft201909 => true,
        #[cfg(feature = "draft202012")]
        Draft::Draft202012 => true,
        _ => false,
    }
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
