use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{no_error, ErrorIterator},
    keywords::{format_validators, CompilationResult, Validators},
    paths::InstancePath,
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct IfThenValidator {
    schema: Validators,
    then_schema: Validators,
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
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if self
            .schema
            .iter()
            .all(|validator| validator.is_valid(schema, instance))
        {
            self.then_schema
                .iter()
                .all(move |validator| validator.is_valid(schema, instance))
        } else {
            true
        }
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self
            .schema
            .iter()
            .all(|validator| validator.is_valid(schema, instance))
        {
            let errors: Vec<_> = self
                .then_schema
                .iter()
                .flat_map(move |validator| validator.validate(schema, instance, instance_path))
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl ToString for IfThenValidator {
    fn to_string(&self) -> String {
        format!(
            "if: {}, then: {}",
            format_validators(&self.schema),
            format_validators(&self.then_schema)
        )
    }
}

pub(crate) struct IfElseValidator {
    schema: Validators,
    else_schema: Validators,
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
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if self
            .schema
            .iter()
            .any(|validator| !validator.is_valid(schema, instance))
        {
            self.else_schema
                .iter()
                .all(move |validator| validator.is_valid(schema, instance))
        } else {
            true
        }
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self
            .schema
            .iter()
            .any(|validator| !validator.is_valid(schema, instance))
        {
            let errors: Vec<_> = self
                .else_schema
                .iter()
                .flat_map(move |validator| validator.validate(schema, instance, instance_path))
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl ToString for IfElseValidator {
    fn to_string(&self) -> String {
        format!(
            "if: {}, else: {}",
            format_validators(&self.schema),
            format_validators(&self.else_schema)
        )
    }
}

pub(crate) struct IfThenElseValidator {
    schema: Validators,
    then_schema: Validators,
    else_schema: Validators,
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
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if self
            .schema
            .iter()
            .all(|validator| validator.is_valid(schema, instance))
        {
            self.then_schema
                .iter()
                .all(move |validator| validator.is_valid(schema, instance))
        } else {
            self.else_schema
                .iter()
                .all(move |validator| validator.is_valid(schema, instance))
        }
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self
            .schema
            .iter()
            .all(|validator| validator.is_valid(schema, instance))
        {
            let errors: Vec<_> = self
                .then_schema
                .iter()
                .flat_map(move |validator| validator.validate(schema, instance, instance_path))
                .collect();
            Box::new(errors.into_iter())
        } else {
            let errors: Vec<_> = self
                .else_schema
                .iter()
                .flat_map(move |validator| validator.validate(schema, instance, instance_path))
                .collect();
            Box::new(errors.into_iter())
        }
    }
}

impl ToString for IfThenElseValidator {
    fn to_string(&self) -> String {
        format!(
            "if: {}, then: {}, else: {}",
            format_validators(&self.schema),
            format_validators(&self.then_schema),
            format_validators(&self.else_schema)
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
