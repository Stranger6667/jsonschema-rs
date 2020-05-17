use super::{CompilationResult, Validate};
use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::{error, no_error, ErrorIterator, PrimitiveType, ValidationError},
};
use serde_json::{Map, Value};
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    hash::{Hash, Hasher},
};

// Based on implementation proposed by Sven Marnach:
// https://stackoverflow.com/questions/60882381/what-is-the-fastest-correct-way-to-detect-that-there-are-no-duplicates-in-a-json
#[derive(PartialEq)]
pub struct HashedValue<'a>(&'a Value);

impl Eq for HashedValue<'_> {}

impl<'a> Hash for HashedValue<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match PrimitiveType::from(self.0) {
            PrimitiveType::Array => {
                // Unwrap is safe as we know that it is an array
                self.0
                    .as_array()
                    .unwrap()
                    .iter()
                    .for_each(|item| HashedValue(item).hash(state))
            }
            PrimitiveType::Boolean => self.0.as_bool().hash(state),
            PrimitiveType::Integer => {
                if let Some(number) = self.0.as_u64() {
                    number.hash(state);
                } else if let Some(number) = self.0.as_i64() {
                    number.hash(state);
                }
            }
            PrimitiveType::Null => state.write_u32(3_221_225_473), // chosen randomly
            PrimitiveType::Number => {
                // Unwrap is safe as we know that it is a number
                self.0.as_f64().unwrap().to_bits().hash(state)
            }
            PrimitiveType::Object => {
                let mut hash = 0;
                // Unwrap is safe as we know that it is an object
                for (key, value) in self.0.as_object().unwrap().iter() {
                    // We have no way of building a new hasher of type `H`, so we
                    // hardcode using the default hasher of a hash map.
                    let mut item_hasher = DefaultHasher::default();
                    key.hash(&mut item_hasher);
                    HashedValue(value).hash(&mut item_hasher);
                    hash ^= item_hasher.finish();
                }
                state.write_u64(hash);
            }
            PrimitiveType::String => self.0.as_str().hash(state),
        }
    }
}

pub fn is_unique(items: &[Value]) -> bool {
    let mut seen = HashSet::with_capacity(items.len());
    items.iter().map(HashedValue).all(move |x| seen.insert(x))
}

pub struct UniqueItemsValidator {}

impl UniqueItemsValidator {
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(UniqueItemsValidator {}))
    }
}

impl Validate for UniqueItemsValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if self.is_valid(schema, instance) {
            no_error()
        } else {
            error(ValidationError::unique_items(instance))
        }
    }

    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Some(items) = instance.as_array() {
            if !is_unique(items) {
                return false;
            }
        }
        true
    }

    fn name(&self) -> String {
        "<unique items>".to_string()
    }
}
pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    if let Some(value) = schema.as_bool() {
        if value {
            Some(UniqueItemsValidator::compile())
        } else {
            None
        }
    } else {
        None
    }
}
