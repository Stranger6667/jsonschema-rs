use crate::{
    compilation::{compile_validators, context::CompilationContext},
    error::{no_error, ErrorIterator},
    keywords::CompilationResult,
    paths::JsonPointerNode,
    schema_node::SchemaNode,
    validator::{format_validators, PartialApplication, Validate},
};
use serde_json::{Map, Value};

pub(crate) struct IfThenValidator {
    schema: SchemaNode,
    then_schema: SchemaNode,
}

impl IfThenValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        then_schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        Ok(Box::new(IfThenValidator {
            schema: {
                let if_context = context.with_path("if");
                compile_validators(schema, &if_context)?
            },
            then_schema: {
                let then_context = context.with_path("then");
                compile_validators(then_schema, &then_context)?
            },
        }))
    }
}

impl Validate for IfThenValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if self.schema.is_valid(instance) {
            self.then_schema.is_valid(instance)
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
        if self.schema.is_valid(instance) {
            let errors: Vec<_> = self.then_schema.validate(instance, instance_path).collect();
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
        let mut if_result = self.schema.apply_rooted(instance, instance_path);
        if if_result.is_valid() {
            let then_result = self.then_schema.apply_rooted(instance, instance_path);
            if_result += then_result;
            if_result.into()
        } else {
            PartialApplication::valid_empty()
        }
    }
}

impl core::fmt::Display for IfThenValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "if: {}, then: {}",
            format_validators(self.schema.validators()),
            format_validators(self.then_schema.validators())
        )
    }
}

pub(crate) struct IfElseValidator {
    schema: SchemaNode,
    else_schema: SchemaNode,
}

impl IfElseValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        else_schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        Ok(Box::new(IfElseValidator {
            schema: {
                let if_context = context.with_path("if");
                compile_validators(schema, &if_context)?
            },
            else_schema: {
                let else_context = context.with_path("else");
                compile_validators(else_schema, &else_context)?
            },
        }))
    }
}

impl Validate for IfElseValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if self.schema.is_valid(instance) {
            true
        } else {
            self.else_schema.is_valid(instance)
        }
    }

    #[allow(clippy::needless_collect)]
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if self.schema.is_valid(instance) {
            no_error()
        } else {
            let errors: Vec<_> = self.else_schema.validate(instance, instance_path).collect();
            Box::new(errors.into_iter())
        }
    }

    fn apply<'a>(
        &'a self,
        instance: &Value,
        instance_path: &JsonPointerNode,
    ) -> PartialApplication<'a> {
        let if_result = self.schema.apply_rooted(instance, instance_path);
        if if_result.is_valid() {
            if_result.into()
        } else {
            self.else_schema
                .apply_rooted(instance, instance_path)
                .into()
        }
    }
}

impl core::fmt::Display for IfElseValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "if: {}, else: {}",
            format_validators(self.schema.validators()),
            format_validators(self.else_schema.validators())
        )
    }
}

pub(crate) struct IfThenElseValidator {
    schema: SchemaNode,
    then_schema: SchemaNode,
    else_schema: SchemaNode,
}

impl IfThenElseValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        then_schema: &'a Value,
        else_schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        Ok(Box::new(IfThenElseValidator {
            schema: {
                let if_context = context.with_path("if");
                compile_validators(schema, &if_context)?
            },
            then_schema: {
                let then_context = context.with_path("then");
                compile_validators(then_schema, &then_context)?
            },
            else_schema: {
                let else_context = context.with_path("else");
                compile_validators(else_schema, &else_context)?
            },
        }))
    }
}

impl Validate for IfThenElseValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if self.schema.is_valid(instance) {
            self.then_schema.is_valid(instance)
        } else {
            self.else_schema.is_valid(instance)
        }
    }

    #[allow(clippy::needless_collect)]
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if self.schema.is_valid(instance) {
            let errors: Vec<_> = self.then_schema.validate(instance, instance_path).collect();
            Box::new(errors.into_iter())
        } else {
            let errors: Vec<_> = self.else_schema.validate(instance, instance_path).collect();
            Box::new(errors.into_iter())
        }
    }

    fn apply<'a>(
        &'a self,
        instance: &Value,
        instance_path: &JsonPointerNode,
    ) -> PartialApplication<'a> {
        let mut if_result = self.schema.apply_rooted(instance, instance_path);
        if if_result.is_valid() {
            if_result += self.then_schema.apply_rooted(instance, instance_path);
            if_result.into()
        } else {
            self.else_schema
                .apply_rooted(instance, instance_path)
                .into()
        }
    }
}

impl core::fmt::Display for IfThenElseValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "if: {}, then: {}, else: {}",
            format_validators(self.schema.validators()),
            format_validators(self.then_schema.validators()),
            format_validators(self.else_schema.validators())
        )
    }
}

#[inline]
pub(crate) fn compile<'a>(
    parent: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    let then = parent.get("then");
    let else_ = parent.get("else");
    match (then, else_) {
        (Some(then_schema), Some(else_schema)) => Some(IfThenElseValidator::compile(
            schema,
            then_schema,
            else_schema,
            context,
        )),
        (None, Some(else_schema)) => Some(IfElseValidator::compile(schema, else_schema, context)),
        (Some(then_schema), None) => Some(IfThenValidator::compile(schema, then_schema, context)),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"if": {"minimum": 0}, "else": {"multipleOf": 2}}), &json!(-1), "/else/multipleOf")]
    #[test_case(&json!({"if": {"minimum": 0}, "then": {"multipleOf": 2}}), &json!(3), "/then/multipleOf")]
    #[test_case(&json!({"if": {"minimum": 0}, "then": {"multipleOf": 2}, "else": {"multipleOf": 2}}), &json!(-1), "/else/multipleOf")]
    #[test_case(&json!({"if": {"minimum": 0}, "then": {"multipleOf": 2}, "else": {"multipleOf": 2}}), &json!(3), "/then/multipleOf")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
