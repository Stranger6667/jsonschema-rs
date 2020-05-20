use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{no_error, ErrorIterator},
    keywords::{format_validators, CompilationResult, Validators},
    validator::Validate,
};
use serde_json::{Map, Value};

pub struct IfThenValidator {
    schema: Validators,
    then_schema: Validators,
}

impl IfThenValidator {
    #[inline]
    pub(crate) fn compile(
        schema: &Value,
        then_schema: &Value,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(IfThenValidator {
            schema: compile_validators(schema, context)?,
            then_schema: compile_validators(then_schema, context)?,
        }))
    }
}

impl Validate for IfThenValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self
            .schema
            .iter()
            .all(|validator| validator.is_valid(schema, instance))
        {
            let errors: Vec<_> = self
                .then_schema
                .iter()
                .flat_map(move |validator| validator.validate(schema, instance))
                .collect();
            return Box::new(errors.into_iter());
        }
        no_error()
    }

    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if self
            .schema
            .iter()
            .all(|validator| validator.is_valid(schema, instance))
        {
            return self
                .then_schema
                .iter()
                .all(move |validator| validator.is_valid(schema, instance));
        }
        true
    }

    fn name(&self) -> String {
        format!(
            "if: {}, then: {}",
            format_validators(&self.schema),
            format_validators(&self.then_schema)
        )
    }
}

pub struct IfElseValidator {
    schema: Validators,
    else_schema: Validators,
}

impl<'a> IfElseValidator {
    #[inline]
    pub(crate) fn compile(
        schema: &'a Value,
        else_schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(IfElseValidator {
            schema: compile_validators(schema, context)?,
            else_schema: compile_validators(else_schema, context)?,
        }))
    }
}

impl Validate for IfElseValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self
            .schema
            .iter()
            .any(|validator| !validator.is_valid(schema, instance))
        {
            let errors: Vec<_> = self
                .else_schema
                .iter()
                .flat_map(move |validator| validator.validate(schema, instance))
                .collect();
            return Box::new(errors.into_iter());
        }
        no_error()
    }

    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if self
            .schema
            .iter()
            .any(|validator| !validator.is_valid(schema, instance))
        {
            return self
                .else_schema
                .iter()
                .all(move |validator| validator.is_valid(schema, instance));
        }
        true
    }

    fn name(&self) -> String {
        format!(
            "if: {}, else: {}",
            format_validators(&self.schema),
            format_validators(&self.else_schema)
        )
    }
}

pub struct IfThenElseValidator {
    schema: Validators,
    then_schema: Validators,
    else_schema: Validators,
}

impl IfThenElseValidator {
    #[inline]
    pub(crate) fn compile(
        schema: &Value,
        then_schema: &Value,
        else_schema: &Value,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(IfThenElseValidator {
            schema: compile_validators(schema, context)?,
            then_schema: compile_validators(then_schema, context)?,
            else_schema: compile_validators(else_schema, context)?,
        }))
    }
}

impl Validate for IfThenElseValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self
            .schema
            .iter()
            .all(|validator| validator.is_valid(schema, instance))
        {
            let errors: Vec<_> = self
                .then_schema
                .iter()
                .flat_map(move |validator| validator.validate(schema, instance))
                .collect();
            Box::new(errors.into_iter())
        } else {
            let errors: Vec<_> = self
                .else_schema
                .iter()
                .flat_map(move |validator| validator.validate(schema, instance))
                .collect();
            Box::new(errors.into_iter())
        }
    }

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

    fn name(&self) -> String {
        format!(
            "if: {}, then: {}, else: {}",
            format_validators(&self.schema),
            format_validators(&self.then_schema),
            format_validators(&self.else_schema)
        )
    }
}

#[inline]
pub fn compile(
    parent: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
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
