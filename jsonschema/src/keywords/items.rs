use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{no_error, ErrorIterator},
    keywords::{format_validators, format_vec_of_validators, CompilationResult, Validators},
    paths::InstancePath,
    validator::Validate,
};
use serde_json::{Map, Value};

pub(crate) struct ItemsArrayValidator {
    items: Vec<Validators>,
}
impl ItemsArrayValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schemas: &'a [Value],
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let keyword_context = context.with_path("items");
        let mut items = Vec::with_capacity(schemas.len());
        for (idx, item) in schemas.iter().enumerate() {
            let item_context = keyword_context.with_path(idx);
            let validators = compile_validators(item, &item_context)?;
            items.push(validators)
        }
        Ok(Box::new(ItemsArrayValidator { items }))
    }
}
impl Validate for ItemsArrayValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            items
                .iter()
                .zip(self.items.iter())
                .all(move |(item, validators)| {
                    validators
                        .iter()
                        .all(move |validator| validator.is_valid(schema, item))
                })
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
        if let Value::Array(items) = instance {
            let errors: Vec<_> = items
                .iter()
                .zip(self.items.iter())
                .enumerate()
                .flat_map(move |(idx, (item, validators))| {
                    validators.iter().flat_map(move |validator| {
                        validator.validate(schema, item, &instance_path.push(idx))
                    })
                })
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl ToString for ItemsArrayValidator {
    fn to_string(&self) -> String {
        format!("items: [{}]", format_vec_of_validators(&self.items))
    }
}

pub(crate) struct ItemsObjectValidator {
    validators: Validators,
}
impl ItemsObjectValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        context: &CompilationContext,
    ) -> CompilationResult<'a> {
        let keyword_context = context.with_path("items");
        let validators = compile_validators(schema, &keyword_context)?;
        Ok(Box::new(ItemsObjectValidator { validators }))
    }
}
impl Validate for ItemsObjectValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Array(items) = instance {
            self.validators.iter().all(move |validator| {
                items
                    .iter()
                    .all(move |item| validator.is_valid(schema, item))
            })
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
        if let Value::Array(items) = instance {
            let errors: Vec<_> = self
                .validators
                .iter()
                .flat_map(move |validator| {
                    items.iter().enumerate().flat_map(move |(idx, item)| {
                        validator.validate(schema, item, &instance_path.push(idx))
                    })
                })
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl ToString for ItemsObjectValidator {
    fn to_string(&self) -> String {
        format!("items: {}", format_validators(&self.validators))
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    match schema {
        Value::Array(items) => Some(ItemsArrayValidator::compile(items, context)),
        Value::Object(_) => Some(ItemsObjectValidator::compile(schema, context)),
        Value::Bool(value) => {
            if *value {
                None
            } else {
                Some(ItemsObjectValidator::compile(schema, context))
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"items": false}), &json!([1]), "/items")]
    #[test_case(&json!({"items": {"type": "string"}}), &json!([1]), "/items/type")]
    #[test_case(&json!({"items": [{"type": "string"}]}), &json!([1]), "/items/0/type")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
