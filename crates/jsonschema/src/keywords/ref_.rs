use std::{collections::VecDeque, rc::Rc, sync::Arc};

use crate::{
    compiler,
    error::ErrorIterator,
    keywords::CompilationResult,
    node::SchemaNode,
    paths::{JsonPointer, JsonPointerNode},
    primitive_type::PrimitiveType,
    validator::Validate,
    ValidationError, ValidationOptions,
};
use once_cell::sync::OnceCell;
use referencing::{uri, Draft, Registry, Resource, Uri};
use serde_json::{Map, Value};

pub(crate) enum RefValidator {
    Default { inner: SchemaNode },
    Lazy(LazyRefValidator),
}

impl RefValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        ctx: &compiler::Context,
        reference: &str,
        is_recursive: bool,
    ) -> CompilationResult<'a> {
        if let Some((base_uri, scopes, resource)) =
            ctx.lookup_maybe_recursive(reference, is_recursive)?
        {
            Ok(Box::new(RefValidator::Lazy(LazyRefValidator {
                resource,
                config: Arc::clone(ctx.config()),
                registry: Arc::clone(&ctx.registry),
                base_uri,
                scopes,
                draft: ctx.draft(),
                inner: OnceCell::default(),
            })))
        } else {
            let (contents, resolver, draft) = ctx.lookup(reference)?.into_inner();
            let resource_ref = draft.create_resource_ref(contents);
            let ctx = ctx.with_resolver_and_draft(resolver, resource_ref.draft());
            let inner =
                compiler::compile_with(&ctx, resource_ref).map_err(|err| err.into_owned())?;
            // TODO: Should ctx include `$ref`?
            Ok(Box::new(RefValidator::Default { inner }))
        }
    }
}

/// Lazily evaluated validator used for recursive references.
///
/// The validator tree nodes can't be arbitrary looked up in the current
/// implementation to build a cycle, therefore recursive references are validated
/// by building and caching the next subtree lazily. Though, other memory
/// representation for the validation tree may allow building cycles easier and
/// lazy evaluation won't be needed.
pub(crate) struct LazyRefValidator {
    resource: Resource,
    config: Arc<ValidationOptions>,
    registry: Arc<Registry>,
    scopes: VecDeque<Uri<String>>,
    base_uri: Uri<String>,
    draft: Draft,
    inner: OnceCell<SchemaNode>,
}

impl LazyRefValidator {
    #[inline]
    pub(crate) fn compile<'a>(ctx: &compiler::Context) -> CompilationResult<'a> {
        let scopes = ctx.scopes();
        let resolved = ctx.lookup_recursive_reference()?;
        let resource = ctx.draft().create_resource(resolved.contents().clone());
        let mut base_uri = resolved.resolver().base_uri().to_owned();
        if let Some(id) = resource.id() {
            base_uri = uri::resolve_against(&base_uri.borrow(), id)?;
        };
        Ok(Box::new(LazyRefValidator {
            resource,
            config: Arc::clone(ctx.config()),
            registry: Arc::clone(&ctx.registry),
            base_uri,
            scopes,
            draft: ctx.draft(),
            inner: OnceCell::default(),
        }))
    }
    fn lazy_compile(&self) -> &SchemaNode {
        self.inner.get_or_init(|| {
            let resolver = self
                .registry
                .resolver_from_raw_parts(self.base_uri.clone(), self.scopes.clone());
            let ctx = compiler::Context::new(
                Arc::clone(&self.config),
                Arc::clone(&self.registry),
                Rc::new(resolver),
                self.draft,
            );
            // INVARIANT: This schema was already used during compilation before detecting a
            // reference cycle that lead to building this validator.
            compiler::compile(&ctx, self.resource.as_ref()).expect("Invalid schema")
        })
    }
}

impl Validate for LazyRefValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        self.lazy_compile().is_valid(instance)
    }
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        self.lazy_compile().validate(instance, instance_path)
    }
}

impl Validate for RefValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        match self {
            RefValidator::Default { inner } => inner.is_valid(instance),
            RefValidator::Lazy(lazy) => lazy.is_valid(instance),
        }
    }

    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        match self {
            RefValidator::Default { inner } => inner.validate(instance, instance_path),
            RefValidator::Lazy(lazy) => lazy.validate(instance, instance_path),
        }
    }
}

fn invalid_reference<'a>(ctx: &compiler::Context, schema: &'a Value) -> ValidationError<'a> {
    ValidationError::single_type_error(
        JsonPointer::default(),
        ctx.clone().into_pointer(),
        schema,
        PrimitiveType::String,
    )
}

#[inline]
pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    parent: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    let is_recursive = parent
        .get("$recursiveAnchor")
        .and_then(Value::as_bool)
        .unwrap_or_default();
    Some(
        schema
            .as_str()
            .ok_or_else(|| invalid_reference(ctx, schema))
            .and_then(|reference| RefValidator::compile(ctx, reference, is_recursive)),
    )
}

#[inline]
pub(crate) fn compile_recursive_ref<'a>(
    ctx: &compiler::Context,
    _: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    Some(
        schema
            .as_str()
            .ok_or_else(|| invalid_reference(ctx, schema))
            .and_then(|_| LazyRefValidator::compile(ctx)),
    )
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(
        &json!({
            "properties": {
                "foo": {"$ref": "#/definitions/foo"}
            },
            "definitions": {
                "foo": {"type": "string"}
            }
        }),
        &json!({"foo": 42}),
        "/properties/foo/type"
    )]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }

    #[test]
    fn multiple_errors_schema_paths() {
        let instance = json!({
            "things": [
                { "code": "CC" },
                { "code": "CC" },
            ]
        });
        let schema = json!({
                "type": "object",
                "properties": {
                    "things": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "code": {
                                    "type": "string",
                                    "$ref": "#/$defs/codes"
                                }
                            },
                            "required": ["code"]
                        }
                    }
                },
                "required": ["things"],
                "$defs": { "codes": { "enum": ["AA", "BB"] } }
        });
        let validator = crate::validator_for(&schema).expect("Invalid schema");
        let mut iter = validator.validate(&instance).expect_err("Should fail");
        let expected = "/properties/things/items/properties/code/enum";
        assert_eq!(
            iter.next()
                .expect("Should be present")
                .schema_path
                .to_string(),
            expected
        );
        assert_eq!(
            iter.next()
                .expect("Should be present")
                .schema_path
                .to_string(),
            expected
        );
    }

    #[test]
    fn test_relative_base_uri() {
        let schema = json!({
            "$id": "/root",
            "$ref": "#/foo",
            "foo": {
                "$id": "#/foo",
                "$ref": "#/bar"
            },
            "bar": {
                "$id": "#/bar",
                "type": "integer"
            },
        });
        let validator = crate::validator_for(&schema).expect("Invalid schema");
        assert!(validator.is_valid(&json!(2)));
        assert!(!validator.is_valid(&json!("a")));
    }
}
