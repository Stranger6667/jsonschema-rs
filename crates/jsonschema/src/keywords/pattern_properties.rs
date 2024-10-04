use crate::{
    compiler, ecma,
    error::{no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    node::SchemaNode,
    output::BasicOutput,
    paths::{JsonPointer, JsonPointerNode},
    primitive_type::PrimitiveType,
    validator::{PartialApplication, Validate},
};
use fancy_regex::Regex;
use serde_json::{Map, Value};

pub(crate) struct PatternPropertiesValidator {
    patterns: Vec<(Regex, SchemaNode)>,
}

impl PatternPropertiesValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        ctx: &compiler::Context,
        map: &'a Map<String, Value>,
    ) -> CompilationResult<'a> {
        let ctx = ctx.with_path("patternProperties");
        let mut patterns = Vec::with_capacity(map.len());
        for (pattern, subschema) in map {
            let pctx = ctx.with_path(pattern.as_str());
            patterns.push((
                match ecma::to_rust_regex(pattern).map(|pattern| Regex::new(&pattern)) {
                    Ok(Ok(r)) => r,
                    _ => {
                        return Err(ValidationError::format(
                            JsonPointer::default(),
                            ctx.clone().into_pointer(),
                            subschema,
                            "regex",
                        ))
                    }
                },
                compiler::compile(&pctx, pctx.as_resource_ref(subschema))?,
            ));
        }
        Ok(Box::new(PatternPropertiesValidator { patterns }))
    }
}

impl Validate for PatternPropertiesValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            self.patterns.iter().all(move |(re, node)| {
                item.iter()
                    .filter(move |(key, _)| re.is_match(key).unwrap_or(false))
                    .all(move |(_key, value)| node.is_valid(value))
            })
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
                .patterns
                .iter()
                .flat_map(move |(re, node)| {
                    item.iter()
                        .filter(move |(key, _)| re.is_match(key).unwrap_or(false))
                        .flat_map(move |(key, value)| {
                            let instance_path = instance_path.push(key.as_str());
                            node.validate(value, &instance_path)
                        })
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
        if let Value::Object(item) = instance {
            let mut matched_propnames = Vec::with_capacity(item.len());
            let mut sub_results = BasicOutput::default();
            for (pattern, node) in &self.patterns {
                for (key, value) in item {
                    if pattern.is_match(key).unwrap_or(false) {
                        let path = instance_path.push(key.as_str());
                        matched_propnames.push(key.clone());
                        sub_results += node.apply_rooted(value, &path);
                    }
                }
            }
            let mut result: PartialApplication = sub_results.into();
            result.annotate(Value::from(matched_propnames).into());
            result
        } else {
            PartialApplication::valid_empty()
        }
    }
}

pub(crate) struct SingleValuePatternPropertiesValidator {
    pattern: Regex,
    node: SchemaNode,
}

impl SingleValuePatternPropertiesValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        ctx: &compiler::Context,
        pattern: &'a str,
        schema: &'a Value,
    ) -> CompilationResult<'a> {
        let kctx = ctx.with_path("patternProperties");
        let pctx = kctx.with_path(pattern);
        Ok(Box::new(SingleValuePatternPropertiesValidator {
            pattern: {
                match ecma::to_rust_regex(pattern).map(|pattern| Regex::new(&pattern)) {
                    Ok(Ok(r)) => r,
                    _ => {
                        return Err(ValidationError::format(
                            JsonPointer::default(),
                            kctx.clone().into_pointer(),
                            schema,
                            "regex",
                        ))
                    }
                }
            },
            node: compiler::compile(&pctx, pctx.as_resource_ref(schema))?,
        }))
    }
}

impl Validate for SingleValuePatternPropertiesValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            item.iter()
                .filter(move |(key, _)| self.pattern.is_match(key).unwrap_or(false))
                .all(move |(_key, value)| self.node.is_valid(value))
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
            let errors: Vec<_> = item
                .iter()
                .filter(move |(key, _)| self.pattern.is_match(key).unwrap_or(false))
                .flat_map(move |(key, value)| {
                    let instance_path = instance_path.push(key.as_str());
                    self.node.validate(value, &instance_path)
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
        if let Value::Object(item) = instance {
            let mut matched_propnames = Vec::with_capacity(item.len());
            let mut outputs = BasicOutput::default();
            for (key, value) in item {
                if self.pattern.is_match(key).unwrap_or(false) {
                    let path = instance_path.push(key.as_str());
                    matched_propnames.push(key.clone());
                    outputs += self.node.apply_rooted(value, &path);
                }
            }
            let mut result: PartialApplication = outputs.into();
            result.annotate(Value::from(matched_propnames).into());
            result
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
    match parent.get("additionalProperties") {
        // This type of `additionalProperties` validator handles `patternProperties` logic
        Some(Value::Bool(false)) | Some(Value::Object(_)) => None,
        _ => {
            if let Value::Object(map) = schema {
                if map.len() == 1 {
                    let (key, value) = map.iter().next().expect("Map is not empty");
                    Some(SingleValuePatternPropertiesValidator::compile(
                        ctx, key, value,
                    ))
                } else {
                    Some(PatternPropertiesValidator::compile(ctx, map))
                }
            } else {
                Some(Err(ValidationError::single_type_error(
                    JsonPointer::default(),
                    ctx.clone().into_pointer(),
                    schema,
                    PrimitiveType::Object,
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"patternProperties": {"^f": {"type": "string"}}}), &json!({"f": 42}), "/patternProperties/^f/type")]
    #[test_case(&json!({"patternProperties": {"^f": {"type": "string"}, "^x": {"type": "string"}}}), &json!({"f": 42}), "/patternProperties/^f/type")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
