use crate::{
    compiler,
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    node::SchemaNode,
    paths::{JsonPointer, JsonPointerNode},
    validator::{PartialApplication, Validate},
    Draft,
};
use serde_json::{Map, Value};

use super::helpers::map_get_u64;

pub(crate) struct ContainsValidator {
    node: SchemaNode,
    schema_path: JsonPointer,
}

impl ContainsValidator {
    #[inline]
    pub(crate) fn compile<'a>(ctx: &compiler::Context, schema: &'a Value) -> CompilationResult<'a> {
        let ctx = ctx.with_path("contains");
        Ok(Box::new(ContainsValidator {
            node: compiler::compile(&ctx, ctx.as_resource_ref(schema))?,
            schema_path: ctx.into_pointer(),
        }))
    }
}

impl Validate for ContainsValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            items.iter().any(|i| self.node.is_valid(i))
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
            if items.iter().any(|i| self.node.is_valid(i)) {
                return no_error();
            }
            error(ValidationError::contains(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
            ))
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
            let mut indices = Vec::new();
            for (idx, item) in items.iter().enumerate() {
                let path = instance_path.push(idx);
                let result = self.node.apply_rooted(item, &path);
                if result.is_valid() {
                    indices.push(idx);
                    results.push(result);
                }
            }
            let mut result: PartialApplication = results.into_iter().collect();
            if indices.is_empty() {
                result.mark_errored(
                    ValidationError::contains(
                        self.schema_path.clone(),
                        instance_path.into(),
                        instance,
                    )
                    .into(),
                );
            } else {
                result.annotate(Value::from(indices).into());
            }
            result
        } else {
            let mut result = PartialApplication::valid_empty();
            result.annotate(Value::Array(Vec::new()).into());
            result
        }
    }
}

/// `minContains` validation. Used only if there is no `maxContains` present.
///
/// Docs: <https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.4.5>
pub(crate) struct MinContainsValidator {
    node: SchemaNode,
    min_contains: u64,
    schema_path: JsonPointer,
}

impl MinContainsValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        ctx: &compiler::Context,
        schema: &'a Value,
        min_contains: u64,
    ) -> CompilationResult<'a> {
        let ctx = ctx.with_path("minContains");
        Ok(Box::new(MinContainsValidator {
            node: compiler::compile(&ctx, ctx.as_resource_ref(schema))?,
            min_contains,
            schema_path: ctx.into_pointer(),
        }))
    }
}

impl Validate for MinContainsValidator {
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if let Value::Array(items) = instance {
            // From docs:
            //   An array instance is valid against "minContains" if the number of elements
            //   that are valid against the schema for "contains" is greater than, or equal to,
            //   the value of this keyword.
            let mut matches = 0;
            for item in items {
                if self
                    .node
                    .validators()
                    .all(|validator| validator.is_valid(item))
                {
                    matches += 1;
                    // Shortcircuit - there is enough matches to satisfy `minContains`
                    if matches >= self.min_contains {
                        return no_error();
                    }
                }
            }
            if self.min_contains > 0 {
                error(ValidationError::contains(
                    self.schema_path.clone(),
                    instance_path.into(),
                    instance,
                ))
            } else {
                // No matches needed
                no_error()
            }
        } else {
            no_error()
        }
    }

    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            let mut matches = 0;
            for item in items {
                if self
                    .node
                    .validators()
                    .all(|validator| validator.is_valid(item))
                {
                    matches += 1;
                    if matches >= self.min_contains {
                        return true;
                    }
                }
            }
            self.min_contains == 0
        } else {
            true
        }
    }
}

/// `maxContains` validation. Used only if there is no `minContains` present.
///
/// Docs: <https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.4.4>
pub(crate) struct MaxContainsValidator {
    node: SchemaNode,
    max_contains: u64,
    schema_path: JsonPointer,
}

impl MaxContainsValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        ctx: &compiler::Context,
        schema: &'a Value,
        max_contains: u64,
    ) -> CompilationResult<'a> {
        let ctx = ctx.with_path("maxContains");
        Ok(Box::new(MaxContainsValidator {
            node: compiler::compile(&ctx, ctx.as_resource_ref(schema))?,
            max_contains,
            schema_path: ctx.into_pointer(),
        }))
    }
}

impl Validate for MaxContainsValidator {
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if let Value::Array(items) = instance {
            // From docs:
            //   An array instance is valid against "maxContains" if the number of elements
            //   that are valid against the schema for "contains" is less than, or equal to,
            //   the value of this keyword.
            let mut matches = 0;
            for item in items {
                if self
                    .node
                    .validators()
                    .all(|validator| validator.is_valid(item))
                {
                    matches += 1;
                    // Shortcircuit - there should be no more than `self.max_contains` matches
                    if matches > self.max_contains {
                        return error(ValidationError::contains(
                            self.schema_path.clone(),
                            instance_path.into(),
                            instance,
                        ));
                    }
                }
            }
            if matches > 0 {
                // It is also less or equal to `self.max_contains`
                // otherwise the loop above would exit early
                no_error()
            } else {
                // No matches - it should be at least one match to satisfy `contains`
                return error(ValidationError::contains(
                    self.schema_path.clone(),
                    instance_path.into(),
                    instance,
                ));
            }
        } else {
            no_error()
        }
    }

    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            let mut matches = 0;
            for item in items {
                if self
                    .node
                    .validators()
                    .all(|validator| validator.is_valid(item))
                {
                    matches += 1;
                    if matches > self.max_contains {
                        return false;
                    }
                }
            }
            matches != 0
        } else {
            true
        }
    }
}

/// `maxContains` & `minContains` validation combined.
///
/// Docs:
///   `maxContains` - <https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.4.4>
///   `minContains` - <https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.4.5>
pub(crate) struct MinMaxContainsValidator {
    node: SchemaNode,
    min_contains: u64,
    max_contains: u64,
    schema_path: JsonPointer,
}

impl MinMaxContainsValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        ctx: &compiler::Context,
        schema: &'a Value,
        min_contains: u64,
        max_contains: u64,
    ) -> CompilationResult<'a> {
        Ok(Box::new(MinMaxContainsValidator {
            node: compiler::compile(ctx, ctx.as_resource_ref(schema))?,
            min_contains,
            max_contains,
            schema_path: ctx.path.clone().into(),
        }))
    }
}

impl Validate for MinMaxContainsValidator {
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if let Value::Array(items) = instance {
            let mut matches = 0;
            for item in items {
                if self
                    .node
                    .validators()
                    .all(|validator| validator.is_valid(item))
                {
                    matches += 1;
                    // Shortcircuit - there should be no more than `self.max_contains` matches
                    if matches > self.max_contains {
                        return error(ValidationError::contains(
                            self.schema_path.clone_with("maxContains"),
                            instance_path.into(),
                            instance,
                        ));
                    }
                }
            }
            if matches < self.min_contains {
                // Not enough matches
                error(ValidationError::contains(
                    self.schema_path.clone_with("minContains"),
                    instance_path.into(),
                    instance,
                ))
            } else {
                no_error()
            }
        } else {
            no_error()
        }
    }

    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            let mut matches = 0;
            for item in items {
                if self
                    .node
                    .validators()
                    .all(|validator| validator.is_valid(item))
                {
                    matches += 1;
                    if matches > self.max_contains {
                        return false;
                    }
                }
            }
            matches <= self.max_contains && matches >= self.min_contains
        } else {
            true
        }
    }
}

#[inline]
pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    parent: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    match ctx.draft() {
        Draft::Draft4 | Draft::Draft6 | Draft::Draft7 => {
            Some(ContainsValidator::compile(ctx, schema))
        }
        Draft::Draft201909 | Draft::Draft202012 => compile_contains(ctx, parent, schema),
        _ => None,
    }
}

#[inline]
fn compile_contains<'a>(
    ctx: &compiler::Context,
    parent: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    let min_contains = match map_get_u64(parent, ctx, "minContains").transpose() {
        Ok(n) => n,
        Err(err) => return Some(Err(err)),
    };
    let max_contains = match map_get_u64(parent, ctx, "maxContains").transpose() {
        Ok(n) => n,
        Err(err) => return Some(Err(err)),
    };

    match (min_contains, max_contains) {
        (Some(min), Some(max)) => Some(MinMaxContainsValidator::compile(ctx, schema, min, max)),
        (Some(min), None) => Some(MinContainsValidator::compile(ctx, schema, min)),
        (None, Some(max)) => Some(MaxContainsValidator::compile(ctx, schema, max)),
        (None, None) => Some(ContainsValidator::compile(ctx, schema)),
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::json;

    #[test]
    fn schema_path() {
        tests_util::assert_schema_path(&json!({"contains": {"const": 2}}), &json!([]), "/contains")
    }
}
