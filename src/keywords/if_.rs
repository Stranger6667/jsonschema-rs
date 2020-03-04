use super::{CompilationResult, ValidationResult};
use super::{Validate, Validators};
use crate::context::CompilationContext;
use crate::validator::compile_validators;
use crate::JSONSchema;
use serde_json::{Map, Value};

pub struct IfThenValidator<'a> {
    schema: Validators<'a>,
    then_schema: Validators<'a>,
}

impl<'a> IfThenValidator<'a> {
    pub(crate) fn compile(
        schema: &'a Value,
        then_schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        Ok(Box::new(IfThenValidator {
            schema: compile_validators(schema, context)?,
            then_schema: compile_validators(then_schema, context)?,
        }))
    }
}

impl<'a> Validate<'a> for IfThenValidator<'a> {
    fn validate(&self, config: &JSONSchema, instance: &Value) -> ValidationResult {
        if self
            .schema
            .iter()
            .all(|validator| validator.is_valid(config, instance))
        {
            for validator in self.then_schema.iter() {
                validator.validate(config, instance)?
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<if-then: {:?} {:?}>", self.schema, self.then_schema)
    }
}

pub struct IfElseValidator<'a> {
    schema: Validators<'a>,
    else_schema: Validators<'a>,
}

impl<'a> IfElseValidator<'a> {
    pub(crate) fn compile(
        schema: &'a Value,
        else_schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        Ok(Box::new(IfElseValidator {
            schema: compile_validators(schema, context)?,
            else_schema: compile_validators(else_schema, context)?,
        }))
    }
}

impl<'a> Validate<'a> for IfElseValidator<'a> {
    fn validate(&self, config: &JSONSchema, instance: &Value) -> ValidationResult {
        if self
            .schema
            .iter()
            .any(|validator| !validator.is_valid(config, instance))
        {
            for validator in self.else_schema.iter() {
                validator.validate(config, instance)?
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!("<if-else: {:?} {:?}>", self.schema, self.else_schema)
    }
}

pub struct IfThenElseValidator<'a> {
    schema: Validators<'a>,
    then_schema: Validators<'a>,
    else_schema: Validators<'a>,
}

impl<'a> IfThenElseValidator<'a> {
    pub(crate) fn compile(
        schema: &'a Value,
        then_schema: &'a Value,
        else_schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        Ok(Box::new(IfThenElseValidator {
            schema: compile_validators(schema, context)?,
            then_schema: compile_validators(then_schema, context)?,
            else_schema: compile_validators(else_schema, context)?,
        }))
    }
}

impl<'a> Validate<'a> for IfThenElseValidator<'a> {
    fn validate(&self, config: &JSONSchema, instance: &Value) -> ValidationResult {
        if self
            .schema
            .iter()
            .all(|validator| validator.is_valid(config, instance))
        {
            for validator in self.then_schema.iter() {
                validator.validate(config, instance)?
            }
        } else {
            for validator in self.else_schema.iter() {
                validator.validate(config, instance)?
            }
        }
        Ok(())
    }
    fn name(&self) -> String {
        format!(
            "<if-then-else: {:?} {:?} {:?}>",
            self.schema, self.then_schema, self.else_schema
        )
    }
}

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
