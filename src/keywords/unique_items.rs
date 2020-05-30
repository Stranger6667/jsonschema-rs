use crate::{
    compilation::{CompilationContext, JSONSchema},
    error::ValidationError,
    keywords::CompilationResult,
    validator::Validate,
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

impl Hash for HashedValue<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.0 {
            Value::Null => state.write_u32(3_221_225_473), // chosen randomly
            Value::Bool(ref item) => item.hash(state),
            Value::Number(ref item) => {
                if let Some(number) = item.as_u64() {
                    number.hash(state);
                } else if let Some(number) = item.as_i64() {
                    number.hash(state);
                } else if let Some(number) = item.as_f64() {
                    number.to_bits().hash(state)
                }
            }
            Value::String(ref item) => item.hash(state),
            Value::Array(ref items) => {
                for item in items {
                    HashedValue(item).hash(state);
                }
            }
            Value::Object(ref items) => {
                let mut hash = 0;
                for (key, value) in items {
                    // We have no way of building a new hasher of type `H`, so we
                    // hardcode using the default hasher of a hash map.
                    let mut item_hasher = DefaultHasher::default();
                    key.hash(&mut item_hasher);
                    HashedValue(value).hash(&mut item_hasher);
                    hash ^= item_hasher.finish();
                }
                state.write_u64(hash);
            }
        }
    }
}

pub fn is_unique(items: &[Value]) -> bool {
    let mut seen = HashSet::with_capacity(items.len());
    items.iter().map(HashedValue).all(move |x| seen.insert(x))
}

pub struct UniqueItemsValidator {}

impl UniqueItemsValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(UniqueItemsValidator {}))
    }
}

impl Validate for UniqueItemsValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::unique_items(instance)
    }

    fn name(&self) -> String {
        "uniqueItems: true".to_string()
    }

    #[inline]
    fn is_valid_array(&self, _: &JSONSchema, _: &Value, instance_value: &[Value]) -> bool {
        is_unique(instance_value)
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    _: &CompilationContext,
) -> Option<CompilationResult> {
    if let Value::Bool(value) = schema {
        if *value {
            Some(UniqueItemsValidator::compile())
        } else {
            None
        }
    } else {
        None
    }
}
