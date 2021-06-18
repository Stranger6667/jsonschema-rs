use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, ValidationError},
    keywords::{helpers, CompilationResult},
    paths::{InstancePath, JSONPointer},
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
    schema_path: JSONPointer,
}

impl EnumValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        items: &'a [Value],
        schema_path: JSONPointer,
    ) -> CompilationResult<'a> {
        let mut types = PrimitiveTypesBitMap::new();
        for item in items.iter() {
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
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
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

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
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

impl ToString for EnumValidator {
    fn to_string(&self) -> String {
        format!(
            "enum: [{}]",
            self.items
                .iter()
                .map(Value::to_string)
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Debug)]
pub(crate) struct SingleValueEnumValidator {
    value: Value,
    options: Value,
    schema_path: JSONPointer,
}

impl SingleValueEnumValidator {
    #[inline]
    pub(crate) fn compile<'a>(
        schema: &'a Value,
        value: &'a Value,
        schema_path: JSONPointer,
    ) -> CompilationResult<'a> {
        Ok(Box::new(SingleValueEnumValidator {
            options: schema.clone(),
            value: value.clone(),
            schema_path,
        }))
    }
}

impl Validate for SingleValueEnumValidator {
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
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

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        helpers::equal(&self.value, instance)
    }
}

impl ToString for SingleValueEnumValidator {
    fn to_string(&self) -> String {
        format!("enum: [{}]", self.value)
    }
}

#[inline]
pub(crate) fn compile<'a>(
    _: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    if let Value::Array(items) = schema {
        let schema_path = context.as_pointer_with("enum");
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
        Some(Err(ValidationError::schema(schema)))
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
