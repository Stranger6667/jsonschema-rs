use crate::keywords::InstancePath;
use crate::{
    compilation::{context::CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::{helpers, CompilationResult},
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
}

impl EnumValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value) -> CompilationResult {
        if let Value::Array(items) = schema {
            let mut types = PrimitiveTypesBitMap::new();
            for item in items.iter() {
                types |= PrimitiveType::from(item);
            }
            Ok(Box::new(EnumValidator {
                options: schema.clone(),
                items: items.clone(),
                types,
            }))
        } else {
            Err(CompilationError::SchemaError)
        }
    }
}

impl Validate for EnumValidator {
    fn validate<'a, 'b>(
        &'b self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        curr_instance_path: InstancePath<'b>,
    ) -> ErrorIterator<'a> {
        if !self.is_valid(schema, instance) {
            error(ValidationError::enumeration(
                curr_instance_path.into(),
                instance,
                &self.options,
            ))
        } else {
            no_error()
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

#[inline]
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    Some(EnumValidator::compile(schema))
}
