use ahash::AHashMap;
use fancy_regex::Regex;
use serde_json::{Map, Value};

use crate::{
    compilation::{compile_validators, context::CompilationContext},
    paths::JSONPointer,
    schema_node::SchemaNode,
    validator::Validate,
    ValidationError,
};

pub(crate) type PatternedValidators = Vec<(Regex, SchemaNode)>;

/// A value that can look up property validators by name.
pub(crate) trait PropertiesValidatorsMap: Send + Sync {
    fn from_map<'a>(
        map: &'a Map<String, Value>,
        context: &CompilationContext,
    ) -> Result<Self, ValidationError<'a>>
    where
        Self: Sized;

    fn get_validator(&self, property: &str) -> Option<&SchemaNode>;
    fn get_key_validator(&self, property: &str) -> Option<(&String, &SchemaNode)>;
}

// We're defining two different property validator map implementations, one for small map sizes and
// one for large map sizes, to optimize the performance depending on the number of properties
// present.
//
// Implementors should use `compile_dynamic_prop_map_validator!` for building their validator maps
// at runtime, as it wraps up all of the logic to choose the right map size and then build and
// compile the validator.
pub(crate) type SmallValidatorsMap = Vec<(String, SchemaNode)>;
pub(crate) type BigValidatorsMap = AHashMap<String, SchemaNode>;

impl PropertiesValidatorsMap for SmallValidatorsMap {
    fn from_map<'a>(
        map: &'a Map<String, Value>,
        context: &CompilationContext,
    ) -> Result<Self, ValidationError<'a>>
    where
        Self: Sized,
    {
        compile_small_map(map, context)
    }

    #[inline]
    fn get_validator(&self, property: &str) -> Option<&SchemaNode> {
        for (prop, node) in self {
            if prop == property {
                return Some(node);
            }
        }
        None
    }
    #[inline]
    fn get_key_validator(&self, property: &str) -> Option<(&String, &SchemaNode)> {
        for (prop, node) in self {
            if prop == property {
                return Some((prop, node));
            }
        }
        None
    }
}

impl PropertiesValidatorsMap for BigValidatorsMap {
    fn from_map<'a>(
        map: &'a Map<String, Value>,
        context: &CompilationContext,
    ) -> Result<Self, ValidationError<'a>>
    where
        Self: Sized,
    {
        compile_big_map(map, context)
    }

    #[inline]
    fn get_validator(&self, property: &str) -> Option<&SchemaNode> {
        self.get(property)
    }

    #[inline]
    fn get_key_validator(&self, property: &str) -> Option<(&String, &SchemaNode)> {
        self.get_key_value(property)
    }
}

pub(crate) fn compile_small_map<'a>(
    map: &'a Map<String, Value>,
    context: &CompilationContext,
) -> Result<SmallValidatorsMap, ValidationError<'a>> {
    let mut properties = Vec::with_capacity(map.len());
    let keyword_context = context.with_path("properties");
    for (key, subschema) in map {
        let property_context = keyword_context.with_path(key.as_str());
        properties.push((
            key.clone(),
            compile_validators(subschema, &property_context)?,
        ));
    }
    Ok(properties)
}

pub(crate) fn compile_big_map<'a>(
    map: &'a Map<String, Value>,
    context: &CompilationContext,
) -> Result<BigValidatorsMap, ValidationError<'a>> {
    let mut properties = AHashMap::with_capacity(map.len());
    let keyword_context = context.with_path("properties");
    for (key, subschema) in map {
        let property_context = keyword_context.with_path(key.as_str());
        properties.insert(
            key.clone(),
            compile_validators(subschema, &property_context)?,
        );
    }
    Ok(properties)
}

pub(crate) fn are_properties_valid<M, F>(prop_map: &M, props: &Map<String, Value>, check: F) -> bool
where
    M: PropertiesValidatorsMap,
    F: Fn(&Value) -> bool,
{
    props.iter().all(|(property, instance)| {
        if let Some(validator) = prop_map.get_validator(property) {
            validator.is_valid(instance)
        } else {
            check(instance)
        }
    })
}

/// Create a vector of pattern-validators pairs.
#[inline]
pub(crate) fn compile_patterns<'a>(
    obj: &'a Map<String, Value>,
    context: &CompilationContext,
) -> Result<PatternedValidators, ValidationError<'a>> {
    let keyword_context = context.with_path("patternProperties");
    let mut compiled_patterns = Vec::with_capacity(obj.len());
    for (pattern, subschema) in obj {
        let pattern_context = keyword_context.with_path(pattern.as_str());
        if let Ok(compiled_pattern) = Regex::new(pattern) {
            let node = compile_validators(subschema, &pattern_context)?;
            compiled_patterns.push((compiled_pattern, node));
        } else {
            return Err(ValidationError::format(
                JSONPointer::default(),
                keyword_context.clone().into_pointer(),
                subschema,
                "regex",
            ));
        }
    }
    Ok(compiled_patterns)
}

macro_rules! compile_dynamic_prop_map_validator {
    ($validator:tt, $properties:ident, $( $arg:expr ),* $(,)*) => {{
        if let Value::Object(map) = $properties {
            if map.len() < 40 {
                Some($validator::<SmallValidatorsMap>::compile(
                    map, $($arg, )*
                ))
            } else {
                Some($validator::<BigValidatorsMap>::compile(
                    map, $($arg, )*
                ))
            }
        } else {
            Some(Err(ValidationError::null_schema()))
        }
    }};
}

pub(crate) use compile_dynamic_prop_map_validator;
