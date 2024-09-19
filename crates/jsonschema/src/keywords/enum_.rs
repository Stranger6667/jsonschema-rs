use crate::{
    compiler,
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{helpers, CompilationResult},
    paths::{JsonPointer, JsonPointerNode},
    primitive_type::{PrimitiveType, PrimitiveTypesBitMap},
    validator::Validate,
};
use serde_json::{Map, Value};

#[derive(Debug)]
pub(crate) struct EnumValidator {
    options: Value,
    // Types that occur in items
    types: PrimitiveTypesBitMap,
    items: Vec<Value>,
    schema_path: JsonPointer,
}

impl EnumValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        items: &'a [Value],
        schema_path: JsonPointer,
    ) -> CompilationResult<'a> {
        let mut types = PrimitiveTypesBitMap::new();
        for item in items {
            types |= PrimitiveType::from(item);
        }
        Ok(Box::new(EnumValidator {
            options: schema.clone(),
            items: items.to_vec(),
            types,
            schema_path,
        }))
    }
}

impl Validate for EnumValidator {
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if self.is_valid(instance) {
            no_error()
        } else {
            error(ValidationError::enumeration(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
                &self.options,
            ))
        }
    }

    fn is_valid(&self, instance: &Value) -> bool {
        // If the input value type is not in the types present among the enum options, then there
        // is no reason to compare it against all items - we know that
        // there are no items with such type at all
        if self.types.contains_type(PrimitiveType::from(instance)) {
            self.items.iter().any(|item| helpers::equal(instance, item))
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub(crate) struct SingleValueEnumValidator {
    value: Value,
    options: Value,
    schema_path: JsonPointer,
}

impl SingleValueEnumValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        value: &'a Value,
        schema_path: JsonPointer,
    ) -> CompilationResult<'a> {
        Ok(Box::new(SingleValueEnumValidator {
            options: schema.clone(),
            value: value.clone(),
            schema_path,
        }))
    }
}

impl Validate for SingleValueEnumValidator {
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if self.is_valid(instance) {
            no_error()
        } else {
            error(ValidationError::enumeration(
                self.schema_path.clone(),
                instance_path.into(),
                instance,
                &self.options,
            ))
        }
    }

    fn is_valid(&self, instance: &Value) -> bool {
        helpers::equal(&self.value, instance)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    _: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    if let Value::Array(items) = schema {
        let schema_path = ctx.as_pointer_with("enum");
        if items.len() == 1 {
            let value = items.iter().next().expect("Vec is not empty");
            Some(SingleValueEnumValidator::compile(
                schema,
                value,
                schema_path,
            ))
        } else {
            Some(EnumValidator::compile(schema, items, schema_path))
        }
    } else {
        Some(Err(ValidationError::single_type_error(
            JsonPointer::default(),
            ctx.clone().into_pointer(),
            schema,
            PrimitiveType::Array,
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({"enum": [1]}), &json!(2), "/enum")]
    #[test_case(&json!({"enum": [1, 3]}), &json!(2), "/enum")]
    fn schema_path(schema: &Value, instance: &Value, expected: &str) {
        tests_util::assert_schema_path(schema, instance, expected)
    }
}
