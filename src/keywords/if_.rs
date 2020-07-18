use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{no_error, ErrorIterator},
    keywords::{format_validators, CompilationResult, Validators},
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct IfThenValidator {
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

macro_rules! if_then_impl_is_valid {
    ($method_suffix:tt, $instance_type: ty) => {
        paste::item! {
            #[inline]
            fn [<is_valid_ $method_suffix>](
                &self,
                schema: &JSONSchema,
                instance: &Value,
                instance_value: $instance_type,
            ) -> bool {
                if self
                    .schema
                    .iter()
                    .all(|validator| validator.[<is_valid_ $method_suffix>](schema, instance, instance_value))
                {
                    self.then_schema
                        .iter()
                        .all(move |validator| validator.[<is_valid_ $method_suffix>](schema, instance, instance_value))
                } else {
                    true
                }
            }
        }
    };
}
macro_rules! if_then_impl_validate {
    ($method_suffix:tt, $instance_type: ty) => {
        paste::item! {
            #[inline]
            fn [<validate_ $method_suffix>]<'a>(
                &self,
                schema: &'a JSONSchema,
                instance: &'a Value,
                instance_value: $instance_type,
            ) -> ErrorIterator<'a> {
                if self
                    .schema
                    .iter()
                    .all(|validator| validator.[<is_valid_ $method_suffix>](schema, instance, instance_value))
                {
                    Box::new(
                        self
                            .then_schema
                            .iter()
                            .flat_map(move |validator| validator.[<validate_ $method_suffix>](schema, instance, instance_value))
                            .collect::<Vec<_>>()
                            .into_iter()
                        )
                } else {
                    no_error()
                }
            }
        }
    };
}

impl Validate for IfThenValidator {
    if_then_impl_is_valid!(array, &[Value]);
    if_then_impl_is_valid!(boolean, bool);
    if_then_impl_is_valid!(null, ());
    if_then_impl_is_valid!(number, f64);
    if_then_impl_is_valid!(object, &Map<String, Value>);
    if_then_impl_is_valid!(signed_integer, i64);
    if_then_impl_is_valid!(string, &str);
    if_then_impl_is_valid!(unsigned_integer, u64);

    if_then_impl_validate!(array, &'a [Value]);
    if_then_impl_validate!(boolean, bool);
    if_then_impl_validate!(null, ());
    if_then_impl_validate!(number, f64);
    if_then_impl_validate!(object, &'a Map<String, Value>);
    if_then_impl_validate!(signed_integer, i64);
    if_then_impl_validate!(string, &'a str);
    if_then_impl_validate!(unsigned_integer, u64);
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
    ) -> CompilationResult {
        Ok(Box::new(IfElseValidator {
            schema: compile_validators(schema, context)?,
            else_schema: compile_validators(else_schema, context)?,
        }))
    }
}

macro_rules! if_else_impl_is_valid {
    ($method_suffix:tt, $instance_type: ty) => {
        paste::item! {
            #[inline]
            fn [<is_valid_ $method_suffix>](
                &self,
                schema: &JSONSchema,
                instance: &Value,
                instance_value: $instance_type,
            ) -> bool {
                if self
                    .schema
                    .iter()
                    .all(|validator| validator.[<is_valid_ $method_suffix>](schema, instance, instance_value))
                {
                    true
                } else {
                    self.else_schema
                        .iter()
                        .all(move |validator| validator.[<is_valid_ $method_suffix>](schema, instance, instance_value))
                }
            }
        }
    };
}
macro_rules! if_else_impl_validate {
    ($method_suffix:tt, $instance_type: ty) => {
        paste::item! {
            #[inline]
            fn [<validate_ $method_suffix>]<'a>(
                &self,
                schema: &'a JSONSchema,
                instance: &'a Value,
                instance_value: $instance_type,
            ) -> ErrorIterator<'a> {
                if self
                    .schema
                    .iter()
                    .all(|validator| validator.[<is_valid_ $method_suffix>](schema, instance, instance_value))
                {
                    no_error()
                } else {
                    Box::new(
                        self
                            .else_schema
                            .iter()
                            .flat_map(move |validator| validator.[<validate_ $method_suffix>](schema, instance, instance_value))
                            .collect::<Vec<_>>()
                            .into_iter()
                        )
                }
            }
        }
    };
}

impl Validate for IfElseValidator {
    if_else_impl_is_valid!(array, &[Value]);
    if_else_impl_is_valid!(boolean, bool);
    if_else_impl_is_valid!(null, ());
    if_else_impl_is_valid!(number, f64);
    if_else_impl_is_valid!(object, &Map<String, Value>);
    if_else_impl_is_valid!(signed_integer, i64);
    if_else_impl_is_valid!(string, &str);
    if_else_impl_is_valid!(unsigned_integer, u64);

    if_else_impl_validate!(array, &'a [Value]);
    if_else_impl_validate!(boolean, bool);
    if_else_impl_validate!(null, ());
    if_else_impl_validate!(number, f64);
    if_else_impl_validate!(object, &'a Map<String, Value>);
    if_else_impl_validate!(signed_integer, i64);
    if_else_impl_validate!(string, &'a str);
    if_else_impl_validate!(unsigned_integer, u64);
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

macro_rules! if_then_else_impl_is_valid {
    ($method_suffix:tt, $instance_type: ty) => {
        paste::item! {
            #[inline]
            fn [<is_valid_ $method_suffix>](
                &self,
                schema: &JSONSchema,
                instance: &Value,
                instance_value: $instance_type,
            ) -> bool {
                if self
                    .schema
                    .iter()
                    .all(|validator| validator.[<is_valid_ $method_suffix>](schema, instance, instance_value))
                {
                    self.then_schema
                        .iter()
                        .all(move |validator| validator.[<is_valid_ $method_suffix>](schema, instance, instance_value))
                } else {
                    self.else_schema
                        .iter()
                        .all(move |validator| validator.[<is_valid_ $method_suffix>](schema, instance, instance_value))
                }
            }
        }
    };
}
macro_rules! if_then_else_impl_validate {
    ($method_suffix:tt, $instance_type: ty) => {
        paste::item! {
            #[inline]
            fn [<validate_ $method_suffix>]<'a>(
                &self,
                schema: &'a JSONSchema,
                instance: &'a Value,
                instance_value: $instance_type,
            ) -> ErrorIterator<'a> {
                if self
                    .schema
                    .iter()
                    .all(|validator| validator.[<is_valid_ $method_suffix>](schema, instance, instance_value))
                {
                    Box::new(
                        self
                            .then_schema
                            .iter()
                            .flat_map(move |validator| validator.[<validate_ $method_suffix>](schema, instance, instance_value))
                            .collect::<Vec<_>>()
                            .into_iter()
                    )
                } else {
                    Box::new(
                        self
                            .else_schema
                            .iter()
                            .flat_map(move |validator| validator.[<validate_ $method_suffix>](schema, instance, instance_value))
                            .collect::<Vec<_>>()
                            .into_iter()
                        )
                }
            }
        }
    };
}

impl Validate for IfThenElseValidator {
    if_then_else_impl_is_valid!(array, &[Value]);
    if_then_else_impl_is_valid!(boolean, bool);
    if_then_else_impl_is_valid!(null, ());
    if_then_else_impl_is_valid!(number, f64);
    if_then_else_impl_is_valid!(object, &Map<String, Value>);
    if_then_else_impl_is_valid!(signed_integer, i64);
    if_then_else_impl_is_valid!(string, &str);
    if_then_else_impl_is_valid!(unsigned_integer, u64);

    if_then_else_impl_validate!(array, &'a [Value]);
    if_then_else_impl_validate!(boolean, bool);
    if_then_else_impl_validate!(null, ());
    if_then_else_impl_validate!(number, f64);
    if_then_else_impl_validate!(object, &'a Map<String, Value>);
    if_then_else_impl_validate!(signed_integer, i64);
    if_then_else_impl_validate!(string, &'a str);
    if_then_else_impl_validate!(unsigned_integer, u64);
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
pub(crate) fn compile(
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
