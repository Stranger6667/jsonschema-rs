use crate::{
    compiler,
    error::{no_error, ErrorIterator, ValidationError},
    keywords::{required, unique_items, CompilationResult},
    node::SchemaNode,
    paths::{JsonPointer, JsonPointerNode},
    primitive_type::PrimitiveType,
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct DependenciesValidator {
    dependencies: Vec<(String, SchemaNode)>,
}

impl DependenciesValidator {
    #[inline]
    pub(crate) fn compile<'a>(ctx: &compiler::Context, schema: &'a Value) -> CompilationResult<'a> {
        if let Value::Object(map) = schema {
            let kctx = ctx.with_path("dependencies");
            let mut dependencies = Vec::with_capacity(map.len());
            for (key, subschema) in map {
                let ctx = kctx.with_path(key.as_str());
                let s = match subschema {
                    Value::Array(_) => {
                        let validators =
                            vec![required::compile_with_path(subschema, (&kctx.path).into())
                                .expect(
                                    "The required validator compilation does not return None",
                                )?];
                        SchemaNode::from_array(&kctx, validators)
                    }
                    _ => compiler::compile(&ctx, ctx.as_resource_ref(subschema))?,
                };
                dependencies.push((key.clone(), s))
            }
            Ok(Box::new(DependenciesValidator { dependencies }))
        } else {
            Err(ValidationError::single_type_error(
                JsonPointer::default(),
                ctx.clone().into_pointer(),
                schema,
                PrimitiveType::Object,
            ))
        }
    }
}

impl Validate for DependenciesValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            self.dependencies
                .iter()
                .filter(|(property, _)| item.contains_key(property))
                .all(move |(_, node)| node.is_valid(instance))
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
        if let Value::Object(item) = instance {
            let errors: Vec<_> = self
                .dependencies
                .iter()
                .filter(|(property, _)| item.contains_key(property))
                .flat_map(move |(_, node)| node.validate(instance, instance_path))
                .collect();
            // TODO. custom error message for "required" case
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

pub(crate) struct DependentRequiredValidator {
    dependencies: Vec<(String, SchemaNode)>,
}

impl DependentRequiredValidator {
    #[inline]
    pub(crate) fn compile<'a>(ctx: &compiler::Context, schema: &'a Value) -> CompilationResult<'a> {
        if let Value::Object(map) = schema {
            let kctx = ctx.with_path("dependentRequired");
            let mut dependencies = Vec::with_capacity(map.len());
            for (key, subschema) in map {
                let ictx = kctx.with_path(key.as_str());
                if let Value::Array(dependency_array) = subschema {
                    if !unique_items::is_unique(dependency_array) {
                        return Err(ValidationError::unique_items(
                            JsonPointer::default(),
                            ictx.clone().into_pointer(),
                            subschema,
                        ));
                    }
                    let validators =
                        vec![required::compile_with_path(subschema, (&kctx.path).into())
                            .expect("The required validator compilation does not return None")?];
                    dependencies.push((key.clone(), SchemaNode::from_array(&kctx, validators)));
                } else {
                    return Err(ValidationError::single_type_error(
                        JsonPointer::default(),
                        ictx.clone().into_pointer(),
                        subschema,
                        PrimitiveType::Array,
                    ));
                }
            }
            Ok(Box::new(DependentRequiredValidator { dependencies }))
        } else {
            Err(ValidationError::single_type_error(
                JsonPointer::default(),
                ctx.clone().into_pointer(),
                schema,
                PrimitiveType::Object,
            ))
        }
    }
}
impl Validate for DependentRequiredValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            self.dependencies
                .iter()
                .filter(|(property, _)| item.contains_key(property))
                .all(move |(_, node)| node.is_valid(instance))
        } else {
            true
        }
    }
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if let Value::Object(item) = instance {
            let errors: Vec<_> = self
                .dependencies
                .iter()
                .filter(|(property, _)| item.contains_key(property))
                .flat_map(move |(_, node)| node.validate(instance, instance_path))
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

pub(crate) struct DependentSchemasValidator {
    dependencies: Vec<(String, SchemaNode)>,
}
impl DependentSchemasValidator {
    #[inline]
    pub(crate) fn compile<'a>(ctx: &compiler::Context, schema: &'a Value) -> CompilationResult<'a> {
        if let Value::Object(map) = schema {
            let ctx = ctx.with_path("dependentSchemas");
            let mut dependencies = Vec::with_capacity(map.len());
            for (key, subschema) in map {
                let ctx = ctx.with_path(key.as_str());
                let schema_nodes = compiler::compile(&ctx, ctx.as_resource_ref(subschema))?;
                dependencies.push((key.clone(), schema_nodes));
            }
            Ok(Box::new(DependentSchemasValidator { dependencies }))
        } else {
            Err(ValidationError::single_type_error(
                JsonPointer::default(),
                ctx.clone().into_pointer(),
                schema,
                PrimitiveType::Object,
            ))
        }
    }
}
impl Validate for DependentSchemasValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            self.dependencies
                .iter()
                .filter(|(property, _)| item.contains_key(property))
                .all(move |(_, node)| node.is_valid(instance))
        } else {
            true
        }
    }
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if let Value::Object(item) = instance {
            let errors: Vec<_> = self
                .dependencies
                .iter()
                .filter(|(property, _)| item.contains_key(property))
                .flat_map(move |(_, node)| node.validate(instance, instance_path))
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

#[inline]
pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    _: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    Some(DependenciesValidator::compile(ctx, schema))
}
#[inline]
pub(crate) fn compile_dependent_required<'a>(
    ctx: &compiler::Context,
    _: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    Some(DependentRequiredValidator::compile(ctx, schema))
}
#[inline]
pub(crate) fn compile_dependent_schemas<'a>(
    ctx: &compiler::Context,
    _: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    Some(DependentSchemasValidator::compile(ctx, schema))
}
#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"dependencies": {"bar": ["foo"]}}), &json!({"bar": 1}), "/dependencies")]
    #[test_case(&json!({"dependencies": {"bar": {"type": "string"}}}), &json!({"bar": 1}), "/dependencies/bar/type")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
