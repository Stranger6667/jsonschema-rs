use crate::{
    compilation::{compile_validators, context::CompilationContext},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    paths::{JSONPointer, JsonPointerNode},
    schema_node::SchemaNode,
    validator::{format_validators, PartialApplication, Validate},
    Draft,
};
use serde_json::{Map, Value};

#[cfg(any(feature = "draft201909", feature = "draft202012"))]
use super::helpers::map_get_u64;

pub(crate) struct ContainsValidator {
    node: SchemaNode,
    schema_path: JSONPointer,
}

impl ContainsValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let keyword_context = context.with_path("contains");
        Ok(Box::new(ContainsValidator {
            node: compile_validators(schema, &keyword_context)?,
            schema_path: keyword_context.into_pointer(),
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

impl core::fmt::Display for ContainsValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "contains: {}", format_validators(self.node.validators()))
    }
}

/// `minContains` validation. Used only if there is no `maxContains` present.
///
/// Docs: <https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.4.5>
pub(crate) struct MinContainsValidator {
    node: SchemaNode,
    min_contains: u64,
    schema_path: JSONPointer,
}

impl MinContainsValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        context: &CompilationContext,
        min_contains: u64,
    ) -> CompilationResult<'a> {
        let keyword_context = context.with_path("minContains");
        Ok(Box::new(MinContainsValidator {
            node: compile_validators(schema, context)?,
            min_contains,
            schema_path: keyword_context.into_pointer(),
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

impl core::fmt::Display for MinContainsValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "minContains: {}, contains: {}",
            self.min_contains,
            format_validators(self.node.validators())
        )
    }
}

/// `maxContains` validation. Used only if there is no `minContains` present.
///
/// Docs: <https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.4.4>
pub(crate) struct MaxContainsValidator {
    node: SchemaNode,
    max_contains: u64,
    schema_path: JSONPointer,
}

impl MaxContainsValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        context: &CompilationContext,
        max_contains: u64,
    ) -> CompilationResult<'a> {
        let keyword_context = context.with_path("maxContains");
        Ok(Box::new(MaxContainsValidator {
            node: compile_validators(schema, context)?,
            max_contains,
            schema_path: keyword_context.into_pointer(),
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

impl core::fmt::Display for MaxContainsValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "maxContains: {}, contains: {}",
            self.max_contains,
            format_validators(self.node.validators())
        )
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
    schema_path: JSONPointer,
}

impl MinMaxContainsValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        context: &CompilationContext,
        min_contains: u64,
        max_contains: u64,
    ) -> CompilationResult<'a> {
        Ok(Box::new(MinMaxContainsValidator {
            node: compile_validators(schema, context)?,
            min_contains,
            max_contains,
            schema_path: context.schema_path.clone().into(),
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

impl core::fmt::Display for MinMaxContainsValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "minContains: {}, maxContains: {}, contains: {}",
            self.min_contains,
            self.max_contains,
            format_validators(self.node.validators())
        )
    }
}

#[inline]
pub(crate) fn compile<'a>(
    parent: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    match context.config.draft() {
        Draft::Draft4 | Draft::Draft6 | Draft::Draft7 => {
            Some(ContainsValidator::compile(schema, context))
        }
        #[cfg(all(feature = "draft201909", feature = "draft202012"))]
        Draft::Draft201909 | Draft::Draft202012 => compile_contains(parent, schema, context),
        #[cfg(all(feature = "draft201909", not(feature = "draft202012")))]
        Draft::Draft201909 => compile_contains(parent, schema, context),
        #[cfg(all(feature = "draft202012", not(feature = "draft201909")))]
        Draft::Draft202012 => compile_contains(parent, schema, context),
    }
}

#[cfg(any(feature = "draft201909", feature = "draft202012"))]
#[inline]
fn compile_contains<'a>(
    parent: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    let min_contains = match map_get_u64(parent, context, "minContains").transpose() {
        Ok(n) => n,
        Err(err) => return Some(Err(err)),
    };
    let max_contains = match map_get_u64(parent, context, "maxContains").transpose() {
        Ok(n) => n,
        Err(err) => return Some(Err(err)),
    };

    match (min_contains, max_contains) {
        (Some(min), Some(max)) => Some(MinMaxContainsValidator::compile(schema, context, min, max)),
        (Some(min), None) => Some(MinContainsValidator::compile(schema, context, min)),
        (None, Some(max)) => Some(MaxContainsValidator::compile(schema, context, max)),
        (None, None) => Some(ContainsValidator::compile(schema, context)),
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
