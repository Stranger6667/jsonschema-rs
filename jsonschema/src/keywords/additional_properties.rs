//! # Description
//! This module contains various validators for the `additionalProperties` keyword.
//!
//! The goal here is to compute intersections with another keywords affecting properties validation:
//!   - `properties`
//!   - `patternProperties`
//!
//! Each valid combination of these keywords has a validator here.
use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::{format_validators, CompilationResult, Validators},
    paths::InstancePath,
    validator::Validate,
};
use ahash::AHashMap;
use fancy_regex::Regex;
use serde_json::{Map, Value};

pub(crate) type PatternedValidators = Vec<(Regex, Validators)>;

/// Provide mapping API to get validators associated with a property from the underlying storage.
pub(crate) trait PropertiesValidatorsMap: Send + Sync {
    fn get_validator(&self, property: &str) -> Option<&Validators>;
    fn get_key_validator(&self, property: &str) -> Option<(&String, &Validators)>;
}

// Iterating over a small vector and comparing strings is faster than a map lookup
const MAP_SIZE_THRESHOLD: usize = 40;
pub(crate) type SmallValidatorsMap = Vec<(String, Validators)>;
pub(crate) type BigValidatorsMap = AHashMap<String, Validators>;

impl PropertiesValidatorsMap for SmallValidatorsMap {
    #[inline]
    fn get_validator(&self, property: &str) -> Option<&Validators> {
        for (prop, validators) in self {
            if prop == property {
                return Some(validators);
            }
        }
        None
    }
    #[inline]
    fn get_key_validator(&self, property: &str) -> Option<(&String, &Validators)> {
        for (prop, validators) in self {
            if prop == property {
                return Some((prop, validators));
            }
        }
        None
    }
}

impl PropertiesValidatorsMap for BigValidatorsMap {
    #[inline]
    fn get_validator(&self, property: &str) -> Option<&Validators> {
        self.get(property)
    }

    #[inline]
    fn get_key_validator(&self, property: &str) -> Option<(&String, &Validators)> {
        self.get_key_value(property)
    }
}

macro_rules! dynamic_map {
    ($validator:tt, $properties:ident, $( $arg:tt ),* $(,)*) => {{
        if let Value::Object(map) = $properties {
            if map.len() < MAP_SIZE_THRESHOLD {
                Some($validator::<SmallValidatorsMap>::compile(
                    map, $($arg, )*
                ))
            } else {
                Some($validator::<BigValidatorsMap>::compile(
                    map, $($arg, )*
                ))
            }
        } else {
            Some(Err(CompilationError::SchemaError))
        }
    }};
}
macro_rules! is_valid {
    ($validators:expr, $schema:ident, $value:ident) => {{
        $validators
            .iter()
            .all(|validator| validator.is_valid($schema, $value))
    }};
}

macro_rules! is_valid_pattern_schema {
    ($validators:expr, $schema:ident, $value:ident) => {{
        if is_valid!($validators, $schema, $value) {
            // Matched & valid - check the next pattern
            continue;
        } else {
            // Invalid - there is no reason to check other patterns
            return false;
        }
    }};
}

macro_rules! is_valid_patterns {
    ($schema:ident, $patterns:expr, $property:ident, $value:ident) => {{
        // One property may match multiple patterns, therefore we need to check them all
        let mut has_match = false;
        for (re, validators) in $patterns {
            // If there is a match, then the value should match the sub-schema
            if re.is_match($property).unwrap_or(false) {
                has_match = true;
                is_valid_pattern_schema!(validators, $schema, $value)
            }
        }
        if !has_match {
            // No pattern matched - INVALID property
            return false;
        }
    }};
}

macro_rules! validate {
    ($validators:expr, $schema:ident, $value:ident, $instance_path:expr, $property_name:expr) => {{
        let instance_path = $instance_path.push($property_name.to_owned());
        $validators
            .iter()
            .flat_map(move |validator| validator.validate($schema, $value, &instance_path))
    }};
}

fn compile_small_map(
    map: &Map<String, Value>,
    context: &CompilationContext,
) -> Result<SmallValidatorsMap, CompilationError> {
    let mut properties = Vec::with_capacity(map.len());
    for (key, subschema) in map {
        properties.push((key.clone(), compile_validators(subschema, context)?));
    }
    Ok(properties)
}
fn compile_big_map(
    map: &Map<String, Value>,
    context: &CompilationContext,
) -> Result<BigValidatorsMap, CompilationError> {
    let mut properties = AHashMap::with_capacity(map.len());
    for (key, subschema) in map {
        properties.insert(key.clone(), compile_validators(subschema, context)?);
    }
    Ok(properties)
}

/// # Schema example
///
/// ```json
/// {
///     "additionalProperties": {"type": "integer"},
/// }
/// ```
///
/// # Valid value
///
/// ```json
/// {
///     "bar": 6
/// }
/// ```
pub(crate) struct AdditionalPropertiesValidator {
    validators: Validators,
}
impl AdditionalPropertiesValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        Ok(Box::new(AdditionalPropertiesValidator {
            validators: compile_validators(schema, context)?,
        }))
    }
}
impl Validate for AdditionalPropertiesValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            for value in item.values() {
                if !is_valid!(self.validators, schema, value) {
                    return false;
                }
            }
        }
        true
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            let errors: Vec<_> = item
                .iter()
                .flat_map(|(name, value)| {
                    validate!(self.validators, schema, value, instance_path, name)
                })
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl ToString for AdditionalPropertiesValidator {
    fn to_string(&self) -> String {
        format!(
            "additionalProperties: {}",
            format_validators(&self.validators)
        )
    }
}

/// # Schema example
///
/// ```json
/// {
///     "additionalProperties": false
/// }
/// ```
///
/// # Valid value
///
/// ```json
/// {}
/// ```
pub(crate) struct AdditionalPropertiesFalseValidator {}
impl AdditionalPropertiesFalseValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(AdditionalPropertiesFalseValidator {}))
    }
}
impl Validate for AdditionalPropertiesFalseValidator {
    fn is_valid(&self, _: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            item.iter().next().is_none()
        } else {
            true
        }
    }

    fn validate<'a>(
        &self,
        _: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            if let Some((_, value)) = item.iter().next() {
                return error(ValidationError::false_schema(instance_path.into(), value));
            }
        }
        no_error()
    }
}
impl ToString for AdditionalPropertiesFalseValidator {
    fn to_string(&self) -> String {
        "additionalProperties: false".to_string()
    }
}

/// # Schema example
///
/// ```json
/// {
///     "additionalProperties": false,
///     "properties": {
///         "foo": {"type": "string"}
///     },
/// }
/// ```
///
/// # Valid value
///
/// ```json
/// {
///     "foo": "bar",
/// }
/// ```
pub(crate) struct AdditionalPropertiesNotEmptyFalseValidator<M: PropertiesValidatorsMap> {
    properties: M,
}
impl AdditionalPropertiesNotEmptyFalseValidator<SmallValidatorsMap> {
    #[inline]
    pub(crate) fn compile(
        map: &Map<String, Value>,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(AdditionalPropertiesNotEmptyFalseValidator {
            properties: compile_small_map(map, context)?,
        }))
    }
}
impl AdditionalPropertiesNotEmptyFalseValidator<BigValidatorsMap> {
    #[inline]
    pub(crate) fn compile(
        map: &Map<String, Value>,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(AdditionalPropertiesNotEmptyFalseValidator {
            properties: compile_big_map(map, context)?,
        }))
    }
}
impl<M: PropertiesValidatorsMap> Validate for AdditionalPropertiesNotEmptyFalseValidator<M> {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            for (property, value) in item {
                if let Some(validators) = self.properties.get_validator(property) {
                    is_valid_pattern_schema!(validators, schema, value)
                } else {
                    // No extra properties are allowed
                    return false;
                }
            }
        }
        true
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            let mut errors = vec![];
            let mut unexpected = vec![];
            for (property, value) in item {
                if let Some((name, validators)) = self.properties.get_key_validator(property) {
                    // When a property is in `properties`, then it should be VALID
                    errors.extend(validate!(validators, schema, value, instance_path, name));
                } else {
                    // No extra properties are allowed
                    unexpected.push(property.to_owned());
                }
            }
            if !unexpected.is_empty() {
                errors.push(ValidationError::additional_properties(
                    instance_path.into(),
                    instance,
                    unexpected,
                ))
            }
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl<M: PropertiesValidatorsMap> ToString for AdditionalPropertiesNotEmptyFalseValidator<M> {
    fn to_string(&self) -> String {
        "additionalProperties: false".to_string()
    }
}

/// # Schema example
///
/// ```json
/// {
///     "additionalProperties": {"type": "integer"},
///     "properties": {
///         "foo": {"type": "string"}
///     }
/// }
/// ```
///
/// # Valid value
///
/// ```json
/// {
///     "foo": "bar",
///     "bar": 6
/// }
/// ```
pub(crate) struct AdditionalPropertiesNotEmptyValidator<M: PropertiesValidatorsMap> {
    validators: Validators,
    properties: M,
}
impl AdditionalPropertiesNotEmptyValidator<SmallValidatorsMap> {
    #[inline]
    pub(crate) fn compile(
        map: &Map<String, Value>,
        schema: &Value,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(AdditionalPropertiesNotEmptyValidator {
            properties: compile_small_map(map, context)?,
            validators: compile_validators(schema, context)?,
        }))
    }
}
impl AdditionalPropertiesNotEmptyValidator<BigValidatorsMap> {
    #[inline]
    pub(crate) fn compile(
        map: &Map<String, Value>,
        schema: &Value,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(AdditionalPropertiesNotEmptyValidator {
            properties: compile_big_map(map, context)?,
            validators: compile_validators(schema, context)?,
        }))
    }
}
impl<M: PropertiesValidatorsMap> Validate for AdditionalPropertiesNotEmptyValidator<M> {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(map) = instance {
            for (property, value) in map {
                if let Some(property_validators) = self.properties.get_validator(property) {
                    is_valid_pattern_schema!(property_validators, schema, value)
                } else {
                    for validator in &self.validators {
                        if !validator.is_valid(schema, value) {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::Object(map) = instance {
            let mut errors = vec![];
            for (property, value) in map {
                if let Some((name, property_validators)) =
                    self.properties.get_key_validator(property)
                {
                    errors.extend(validate!(
                        property_validators,
                        schema,
                        value,
                        instance_path,
                        name
                    ))
                } else {
                    errors.extend(validate!(
                        self.validators,
                        schema,
                        value,
                        instance_path,
                        property
                    ))
                }
            }
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl<M: PropertiesValidatorsMap> ToString for AdditionalPropertiesNotEmptyValidator<M> {
    fn to_string(&self) -> String {
        format!(
            "additionalProperties: {}",
            format_validators(&self.validators)
        )
    }
}

/// # Schema example
///
/// ```json
/// {
///     "additionalProperties": {"type": "integer"},
///     "patternProperties": {
///         "^x-": {"type": "integer", "minimum": 5},
///         "-x$": {"type": "integer", "maximum": 10}
///     }
/// }
/// ```
///
/// # Valid value
///
/// ```json
/// {
///     "x-foo": 6,
///     "foo-x": 7,
///     "bar": 8
/// }
/// ```
pub(crate) struct AdditionalPropertiesWithPatternsValidator {
    validators: Validators,
    patterns: PatternedValidators,
}
impl AdditionalPropertiesWithPatternsValidator {
    #[inline]
    pub(crate) fn compile(
        schema: &Value,
        patterns: PatternedValidators,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(AdditionalPropertiesWithPatternsValidator {
            validators: compile_validators(schema, context)?,
            patterns,
        }))
    }
}
impl Validate for AdditionalPropertiesWithPatternsValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            for (property, value) in item.iter() {
                let mut has_match = false;
                for (re, validators) in &self.patterns {
                    if re.is_match(property).unwrap_or(false) {
                        has_match = true;
                        is_valid_pattern_schema!(validators, schema, value)
                    }
                }
                if !has_match && !is_valid!(self.validators, schema, value) {
                    return false;
                }
            }
        }
        true
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            let mut errors = vec![];
            for (property, value) in item.iter() {
                let mut has_match = false;
                errors.extend(
                    self.patterns
                        .iter()
                        .filter(|(re, _)| re.is_match(property).unwrap_or(false))
                        .flat_map(|(_, validators)| {
                            has_match = true;
                            validate!(validators, schema, value, instance_path, property)
                        }),
                );
                if !has_match {
                    errors.extend(validate!(
                        self.validators,
                        schema,
                        value,
                        instance_path,
                        property
                    ))
                }
            }
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl ToString for AdditionalPropertiesWithPatternsValidator {
    fn to_string(&self) -> String {
        format!(
            "additionalProperties: {}",
            format_validators(&self.validators)
        )
    }
}

/// # Schema example
///
/// ```json
/// {
///     "additionalProperties": false,
///     "patternProperties": {
///         "^x-": {"type": "integer", "minimum": 5},
///         "-x$": {"type": "integer", "maximum": 10}
///     }
/// }
/// ```
///
/// # Valid value
///
/// ```json
/// {
///     "x-bar": 6,
///     "spam-x": 7,
///     "x-baz-x": 8,
/// }
/// ```
pub(crate) struct AdditionalPropertiesWithPatternsFalseValidator {
    patterns: PatternedValidators,
}
impl AdditionalPropertiesWithPatternsFalseValidator {
    #[inline]
    pub(crate) fn compile(patterns: PatternedValidators) -> CompilationResult {
        Ok(Box::new(AdditionalPropertiesWithPatternsFalseValidator {
            patterns,
        }))
    }
}
impl Validate for AdditionalPropertiesWithPatternsFalseValidator {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            // No properties are allowed, except ones defined in `patternProperties`
            for (property, value) in item {
                is_valid_patterns!(schema, &self.patterns, property, value);
            }
        }
        true
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            let mut errors = vec![];
            let mut unexpected = vec![];
            for (property, value) in item {
                let mut has_match = false;
                errors.extend(
                    self.patterns
                        .iter()
                        .filter(|(re, _)| re.is_match(property).unwrap_or(false))
                        .flat_map(|(_, validators)| {
                            has_match = true;
                            validate!(validators, schema, value, instance_path, property)
                        }),
                );
                if !has_match {
                    unexpected.push(property.to_owned());
                }
            }
            if !unexpected.is_empty() {
                errors.push(ValidationError::additional_properties(
                    instance_path.into(),
                    instance,
                    unexpected,
                ))
            }
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl ToString for AdditionalPropertiesWithPatternsFalseValidator {
    fn to_string(&self) -> String {
        "additionalProperties: false".to_string()
    }
}

/// # Schema example
///
/// ```json
/// {
///     "additionalProperties": {"type": "integer"},
///     "properties": {
///         "foo": {"type": "string"}
///     },
///     "patternProperties": {
///         "^x-": {"type": "integer", "minimum": 5},
///         "-x$": {"type": "integer", "maximum": 10}
///     }
/// }
/// ```
///
/// # Valid value
///
/// ```json
/// {
///     "foo": "a",
///     "x-spam": 6,
///     "spam-x": 7,
///     "x-spam-x": 8,
///     "bar": 42
/// }
/// ```
pub(crate) struct AdditionalPropertiesWithPatternsNotEmptyValidator<M: PropertiesValidatorsMap> {
    validators: Validators,
    properties: M,
    patterns: PatternedValidators,
}
impl AdditionalPropertiesWithPatternsNotEmptyValidator<SmallValidatorsMap> {
    #[inline]
    pub(crate) fn compile(
        map: &Map<String, Value>,
        schema: &Value,
        patterns: PatternedValidators,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(
            AdditionalPropertiesWithPatternsNotEmptyValidator {
                validators: compile_validators(schema, context)?,
                properties: compile_small_map(map, context)?,
                patterns,
            },
        ))
    }
}
impl AdditionalPropertiesWithPatternsNotEmptyValidator<BigValidatorsMap> {
    #[inline]
    pub(crate) fn compile(
        map: &Map<String, Value>,
        schema: &Value,
        patterns: PatternedValidators,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(
            AdditionalPropertiesWithPatternsNotEmptyValidator {
                validators: compile_validators(schema, context)?,
                properties: compile_big_map(map, context)?,
                patterns,
            },
        ))
    }
}
impl<M: PropertiesValidatorsMap> Validate for AdditionalPropertiesWithPatternsNotEmptyValidator<M> {
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            for (property, value) in item.iter() {
                if let Some(validators) = self.properties.get_validator(property) {
                    if is_valid!(validators, schema, value) {
                        // Valid for `properties`, check `patternProperties`
                        for (re, validators) in &self.patterns {
                            // If there is a match, then the value should match the sub-schema
                            if re.is_match(property).unwrap_or(false) {
                                is_valid_pattern_schema!(validators, schema, value)
                            }
                        }
                    } else {
                        // INVALID, no reason to check the next one
                        return false;
                    }
                } else {
                    let mut has_match = false;
                    for (re, validators) in &self.patterns {
                        // If there is a match, then the value should match the sub-schema
                        if re.is_match(property).unwrap_or(false) {
                            has_match = true;
                            is_valid_pattern_schema!(validators, schema, value)
                        }
                    }
                    if !has_match && !is_valid!(self.validators, schema, value) {
                        return false;
                    }
                }
            }
            true
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
        if let Value::Object(item) = instance {
            let mut errors = vec![];
            for (property, value) in item.iter() {
                if let Some((name, validators)) = self.properties.get_key_validator(property) {
                    errors.extend(validate!(validators, schema, value, instance_path, name));
                    errors.extend(
                        self.patterns
                            .iter()
                            .filter(|(re, _)| re.is_match(property).unwrap_or(false))
                            .flat_map(|(_, validators)| {
                                validate!(validators, schema, value, instance_path, name)
                            }),
                    );
                } else {
                    let mut has_match = false;
                    errors.extend(
                        self.patterns
                            .iter()
                            .filter(|(re, _)| re.is_match(property).unwrap_or(false))
                            .flat_map(|(_, validators)| {
                                has_match = true;
                                validate!(validators, schema, value, instance_path, property)
                            }),
                    );
                    if !has_match {
                        errors.extend(validate!(
                            self.validators,
                            schema,
                            value,
                            instance_path,
                            property
                        ))
                    }
                }
            }
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}
impl<M: PropertiesValidatorsMap> ToString for AdditionalPropertiesWithPatternsNotEmptyValidator<M> {
    fn to_string(&self) -> String {
        format!(
            "additionalProperties: {}",
            format_validators(&self.validators)
        )
    }
}

/// # Schema example
///
/// ```json
/// {
///     "additionalProperties": false,
///     "properties": {
///         "foo": {"type": "string"}
///     },
///     "patternProperties": {
///         "^x-": {"type": "integer", "minimum": 5},
///         "-x$": {"type": "integer", "maximum": 10}
///     }
/// }
/// ```
///
/// # Valid value
///
/// ```json
/// {
///     "foo": "bar",
///     "x-bar": 6,
///     "spam-x": 7,
///     "x-baz-x": 8,
/// }
/// ```
pub(crate) struct AdditionalPropertiesWithPatternsNotEmptyFalseValidator<M: PropertiesValidatorsMap>
{
    properties: M,
    patterns: PatternedValidators,
}
impl AdditionalPropertiesWithPatternsNotEmptyFalseValidator<SmallValidatorsMap> {
    #[inline]
    pub(crate) fn compile(
        map: &Map<String, Value>,
        patterns: PatternedValidators,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(
            AdditionalPropertiesWithPatternsNotEmptyFalseValidator::<SmallValidatorsMap> {
                properties: compile_small_map(map, context)?,
                patterns,
            },
        ))
    }
}
impl AdditionalPropertiesWithPatternsNotEmptyFalseValidator<BigValidatorsMap> {
    #[inline]
    pub(crate) fn compile(
        map: &Map<String, Value>,
        patterns: PatternedValidators,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(
            AdditionalPropertiesWithPatternsNotEmptyFalseValidator {
                properties: compile_big_map(map, context)?,
                patterns,
            },
        ))
    }
}

impl<M: PropertiesValidatorsMap> Validate
    for AdditionalPropertiesWithPatternsNotEmptyFalseValidator<M>
{
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            // No properties are allowed, except ones defined in `properties` or `patternProperties`
            for (property, value) in item.iter() {
                if let Some(validators) = self.properties.get_validator(property) {
                    if is_valid!(validators, schema, value) {
                        // Valid for `properties`, check `patternProperties`
                        for (re, validators) in &self.patterns {
                            // If there is a match, then the value should match the sub-schema
                            if re.is_match(property).unwrap_or(false) {
                                is_valid_pattern_schema!(validators, schema, value)
                            }
                        }
                    } else {
                        // INVALID, no reason to check the next one
                        return false;
                    }
                } else {
                    is_valid_patterns!(schema, &self.patterns, property, value);
                }
            }
        }
        true
    }

    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            let mut errors = vec![];
            let mut unexpected = vec![];
            // No properties are allowed, except ones defined in `properties` or `patternProperties`
            for (property, value) in item.iter() {
                if let Some((name, validators)) = self.properties.get_key_validator(property) {
                    errors.extend(validate!(validators, schema, value, instance_path, name));
                    errors.extend(
                        self.patterns
                            .iter()
                            .filter(|(re, _)| re.is_match(property).unwrap_or(false))
                            .flat_map(|(_, validators)| {
                                validate!(validators, schema, value, instance_path, name)
                            }),
                    );
                } else {
                    let mut has_match = false;
                    errors.extend(
                        self.patterns
                            .iter()
                            .filter(|(re, _)| re.is_match(property).unwrap_or(false))
                            .flat_map(|(_, validators)| {
                                has_match = true;
                                validate!(validators, schema, value, instance_path, property)
                            }),
                    );
                    if !has_match {
                        unexpected.push(property.to_owned());
                    }
                }
            }
            if !unexpected.is_empty() {
                errors.push(ValidationError::additional_properties(
                    instance_path.into(),
                    instance,
                    unexpected,
                ))
            }
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }
}

impl<M: PropertiesValidatorsMap> ToString
    for AdditionalPropertiesWithPatternsNotEmptyFalseValidator<M>
{
    fn to_string(&self) -> String {
        "additionalProperties: false".to_string()
    }
}
#[inline]
pub(crate) fn compile(
    parent: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    let properties = parent.get("properties");
    if let Some(patterns) = parent.get("patternProperties") {
        if let Value::Object(obj) = patterns {
            // Compile all patterns & their validators to avoid doing work in the `patternProperties` validator
            if let Ok(compiled_patterns) = compile_patterns(obj, context) {
                match schema {
                    Value::Bool(true) => None, // "additionalProperties" are "true" by default
                    Value::Bool(false) => {
                        if let Some(properties) = properties {
                            dynamic_map!(
                                AdditionalPropertiesWithPatternsNotEmptyFalseValidator,
                                properties,
                                compiled_patterns,
                                context
                            )
                        } else {
                            Some(AdditionalPropertiesWithPatternsFalseValidator::compile(
                                compiled_patterns,
                            ))
                        }
                    }
                    _ => {
                        if let Some(properties) = properties {
                            dynamic_map!(
                                AdditionalPropertiesWithPatternsNotEmptyValidator,
                                properties,
                                schema,
                                compiled_patterns,
                                context
                            )
                        } else {
                            Some(AdditionalPropertiesWithPatternsValidator::compile(
                                schema,
                                compiled_patterns,
                                context,
                            ))
                        }
                    }
                }
            } else {
                Some(Err(CompilationError::SchemaError))
            }
        } else {
            Some(Err(CompilationError::SchemaError))
        }
    } else {
        match schema {
            Value::Bool(true) => None, // "additionalProperties" are "true" by default
            Value::Bool(false) => {
                if let Some(properties) = properties {
                    dynamic_map!(
                        AdditionalPropertiesNotEmptyFalseValidator,
                        properties,
                        context
                    )
                } else {
                    Some(AdditionalPropertiesFalseValidator::compile())
                }
            }
            _ => {
                if let Some(properties) = properties {
                    dynamic_map!(
                        AdditionalPropertiesNotEmptyValidator,
                        properties,
                        schema,
                        context
                    )
                } else {
                    Some(AdditionalPropertiesValidator::compile(schema, context))
                }
            }
        }
    }
}

/// Create a vector of pattern-validators pairs.
#[inline]
fn compile_patterns(
    obj: &Map<String, Value>,
    context: &CompilationContext,
) -> Result<PatternedValidators, CompilationError> {
    let mut compiled_patterns = Vec::with_capacity(obj.len());
    for (pattern, subschema) in obj {
        if let Ok(compiled_pattern) = Regex::new(pattern) {
            if let Ok(validators) = compile_validators(subschema, context) {
                compiled_patterns.push((compiled_pattern, validators));
            } else {
                return Err(CompilationError::SchemaError);
            }
        } else {
            return Err(CompilationError::SchemaError);
        }
    }
    Ok(compiled_patterns)
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::{json, Value};
    use test_case::test_case;

    fn schema_1() -> Value {
        // For `AdditionalPropertiesWithPatternsNotEmptyFalseValidator`
        json!({
            "additionalProperties": false,
            "properties": {
                "foo": {"type": "string"},
                "barbaz": {"type": "integer", "multipleOf": 3},
            },
            "patternProperties": {
                "^bar": {"type": "integer", "minimum": 5},
                "spam$": {"type": "integer", "maximum": 10},
            }
        })
    }

    // Another type
    #[test_case(&json!([1]))]
    // The right type
    #[test_case(&json!({}))]
    // Match `properties.foo`
    #[test_case(&json!({"foo": "a"}))]
    // Match `properties.barbaz` & `patternProperties.^bar`
    #[test_case(&json!({"barbaz": 6}))]
    // Match `patternProperties.^bar`
    #[test_case(&json!({"bar": 6}))]
    // Match `patternProperties.spam$`
    #[test_case(&json!({"spam": 7}))]
    // All `patternProperties` rules match on different values
    #[test_case(&json!({"bar": 6, "spam": 7}))]
    // All `patternProperties` rules match on the same value
    #[test_case(&json!({"barspam": 7}))]
    // All combined
    #[test_case(&json!({"barspam": 7, "bar": 6, "spam": 7, "foo": "a", "barbaz": 6}))]
    fn schema_1_valid(instance: &Value) {
        let schema = schema_1();
        tests_util::is_valid(&schema, instance)
    }

    // `properties.bar` - should be a string
    #[test_case(&json!({"foo": 3}), &["3 is not of type \"string\""])]
    // `additionalProperties` - extra keyword & not in `properties` / `patternProperties`
    #[test_case(&json!({"faz": 1}), &["Additional properties are not allowed (\'faz\' was unexpected)"])]
    #[test_case(&json!({"faz": 1, "haz": 1}), &["Additional properties are not allowed (\'faz\', \'haz\' were unexpected)"])]
    // `properties.foo` - should be a string & `patternProperties.^bar` - invalid
    #[test_case(&json!({"foo": 3, "bar": 4}), &["4 is less than the minimum of 5", "3 is not of type \"string\""])]
    // `properties.barbaz` - valid; `patternProperties.^bar` - invalid
    #[test_case(&json!({"barbaz": 3}), &["3 is less than the minimum of 5"])]
    // `patternProperties.^bar` (should be >=5)
    #[test_case(&json!({"bar": 4}), &["4 is less than the minimum of 5"])]
    // `patternProperties.spam$` (should be <=10)
    #[test_case(&json!({"spam": 11}), &["11 is greater than the maximum of 10"])]
    // `patternProperties` - both values are invalid
    #[test_case(&json!({"bar": 4, "spam": 11}), &["4 is less than the minimum of 5", "11 is greater than the maximum of 10"])]
    // `patternProperties` - `bar` is valid, `spam` is invalid
    #[test_case(&json!({"bar": 6, "spam": 11}), &["11 is greater than the maximum of 10"])]
    // `patternProperties` - `bar` is invalid, `spam` is valid
    #[test_case(&json!({"bar": 4, "spam": 8}), &["4 is less than the minimum of 5"])]
    // `patternProperties.^bar` - (should be >=5), but valid for `patternProperties.spam$`
    #[test_case(&json!({"barspam": 4}), &["4 is less than the minimum of 5"])]
    // `patternProperties.spam$` - (should be <=10), but valid for `patternProperties.^bar`
    #[test_case(&json!({"barspam": 11}), &["11 is greater than the maximum of 10"])]
    // All combined
    #[test_case(
      &json!({"bar": 4, "spam": 11, "foo": 3, "faz": 1}),
      &[
         "4 is less than the minimum of 5",
         "3 is not of type \"string\"",
         "11 is greater than the maximum of 10",
         "Additional properties are not allowed (\'faz\' was unexpected)"
      ]
    )]
    fn schema_1_invalid(instance: &Value, expected: &[&str]) {
        let schema = schema_1();
        tests_util::is_not_valid(&schema, instance);
        tests_util::expect_errors(&schema, instance, expected)
    }

    fn schema_2() -> Value {
        // For `AdditionalPropertiesWithPatternsFalseValidator`
        json!({
            "additionalProperties": false,
            "patternProperties": {
                "^bar": {"type": "integer", "minimum": 5},
                "spam$": {"type": "integer", "maximum": 10},
            }
        })
    }

    // Another type
    #[test_case(&json!([1]))]
    // The right type
    #[test_case(&json!({}))]
    // Match `patternProperties.^bar`
    #[test_case(&json!({"bar": 6}))]
    // Match `patternProperties.spam$`
    #[test_case(&json!({"spam": 7}))]
    // All `patternProperties` rules match on different values
    #[test_case(&json!({"bar": 6, "spam": 7}))]
    // All `patternProperties` rules match on the same value
    #[test_case(&json!({"barspam": 7}))]
    // All combined
    #[test_case(&json!({"barspam": 7, "bar": 6, "spam": 7}))]
    fn schema_2_valid(instance: &Value) {
        let schema = schema_2();
        tests_util::is_valid(&schema, instance)
    }

    // `additionalProperties` - extra keyword & not in `patternProperties`
    #[test_case(&json!({"faz": "a"}), &["Additional properties are not allowed (\'faz\' was unexpected)"])]
    // `patternProperties.^bar` (should be >=5)
    #[test_case(&json!({"bar": 4}), &["4 is less than the minimum of 5"])]
    // `patternProperties.spam$` (should be <=10)
    #[test_case(&json!({"spam": 11}), &["11 is greater than the maximum of 10"])]
    // `patternProperties` - both values are invalid
    #[test_case(&json!({"bar": 4, "spam": 11}), &["4 is less than the minimum of 5", "11 is greater than the maximum of 10"])]
    // `patternProperties` - `bar` is valid, `spam` is invalid
    #[test_case(&json!({"bar": 6, "spam": 11}), &["11 is greater than the maximum of 10"])]
    // `patternProperties` - `bar` is invalid, `spam` is valid
    #[test_case(&json!({"bar": 4, "spam": 8}), &["4 is less than the minimum of 5"])]
    // `patternProperties.^bar` - (should be >=5), but valid for `patternProperties.spam$`
    #[test_case(&json!({"barspam": 4}), &["4 is less than the minimum of 5"])]
    // `patternProperties.spam$` - (should be <=10), but valid for `patternProperties.^bar`
    #[test_case(&json!({"barspam": 11}), &["11 is greater than the maximum of 10"])]
    // All combined
    #[test_case(
      &json!({"bar": 4, "spam": 11, "faz": 1}),
      &[
         "4 is less than the minimum of 5",
         "11 is greater than the maximum of 10",
         "Additional properties are not allowed (\'faz\' was unexpected)"
      ]
    )]
    fn schema_2_invalid(instance: &Value, expected: &[&str]) {
        let schema = schema_2();
        tests_util::is_not_valid(&schema, instance);
        tests_util::expect_errors(&schema, instance, expected)
    }

    fn schema_3() -> Value {
        // For `AdditionalPropertiesNotEmptyFalseValidator`
        json!({
            "additionalProperties": false,
            "properties": {
                "foo": {"type": "string"}
            }
        })
    }

    // Another type
    #[test_case(&json!([1]))]
    // The right type
    #[test_case(&json!({}))]
    // Match `properties`
    #[test_case(&json!({"foo": "a"}))]
    fn schema_3_valid(instance: &Value) {
        let schema = schema_3();
        tests_util::is_valid(&schema, instance)
    }

    // `properties` - should be a string
    #[test_case(&json!({"foo": 3}), &["3 is not of type \"string\""])]
    // `additionalProperties` - extra keyword & not in `properties`
    #[test_case(&json!({"faz": "a"}), &["Additional properties are not allowed (\'faz\' was unexpected)"])]
    // All combined
    #[test_case(
      &json!(
        {"foo": 3, "faz": "a"}),
        &[
          "3 is not of type \"string\"",
          "Additional properties are not allowed (\'faz\' was unexpected)",
        ]
    )]
    fn schema_3_invalid(instance: &Value, expected: &[&str]) {
        let schema = schema_3();
        tests_util::is_not_valid(&schema, instance);
        tests_util::expect_errors(&schema, instance, expected)
    }

    fn schema_4() -> Value {
        // For `AdditionalPropertiesNotEmptyValidator`
        json!({
            "additionalProperties": {"type": "integer"},
            "properties": {
                "foo": {"type": "string"}
            }
        })
    }

    // Another type
    #[test_case(&json!([1]))]
    // The right type
    #[test_case(&json!({}))]
    // Match `properties`
    #[test_case(&json!({"foo": "a"}))]
    // Match `additionalProperties`
    #[test_case(&json!({"bar": 4}))]
    // All combined
    #[test_case(&json!({"foo": "a", "bar": 4}))]
    fn schema_4_valid(instance: &Value) {
        let schema = schema_4();
        tests_util::is_valid(&schema, instance)
    }

    // `properties` - should be a string
    #[test_case(&json!({"foo": 3}), &["3 is not of type \"string\""])]
    // `additionalProperties` - should be an integer
    #[test_case(&json!({"bar": "a"}), &["\"a\" is not of type \"integer\""])]
    // All combined
    #[test_case(
      &json!(
        {"foo": 3, "bar": "a"}),
        &[
          "\"a\" is not of type \"integer\"",
          "3 is not of type \"string\""
        ]
    )]
    fn schema_4_invalid(instance: &Value, expected: &[&str]) {
        let schema = schema_4();
        tests_util::is_not_valid(&schema, instance);
        tests_util::expect_errors(&schema, instance, expected)
    }

    fn schema_5() -> Value {
        // For `AdditionalPropertiesWithPatternsNotEmptyValidator`
        json!({
            "additionalProperties": {"type": "integer"},
            "properties": {
                "foo": {"type": "string"},
                "barbaz": {"type": "integer", "multipleOf": 3},
            },
            "patternProperties": {
                "^bar": {"type": "integer", "minimum": 5},
                "spam$": {"type": "integer", "maximum": 10},
            }
        })
    }

    // Another type
    #[test_case(&json!([1]))]
    // The right type
    #[test_case(&json!({}))]
    // Match `properties.foo`
    #[test_case(&json!({"foo": "a"}))]
    // Match `additionalProperties`
    #[test_case(&json!({"faz": 42}))]
    // Match `properties.barbaz` & `patternProperties.^bar`
    #[test_case(&json!({"barbaz": 6}))]
    // Match `patternProperties.^bar`
    #[test_case(&json!({"bar": 6}))]
    // Match `patternProperties.spam$`
    #[test_case(&json!({"spam": 7}))]
    // All `patternProperties` rules match on different values
    #[test_case(&json!({"bar": 6, "spam": 7}))]
    // All `patternProperties` rules match on the same value
    #[test_case(&json!({"barspam": 7}))]
    // All combined
    #[test_case(&json!({"barspam": 7, "bar": 6, "spam": 7, "foo": "a", "barbaz": 6, "faz": 42}))]
    fn schema_5_valid(instance: &Value) {
        let schema = schema_5();
        tests_util::is_valid(&schema, instance)
    }

    // `properties.bar` - should be a string
    #[test_case(&json!({"foo": 3}), &["3 is not of type \"string\""])]
    // `additionalProperties` - extra keyword that doesn't match `additionalProperties`
    #[test_case(&json!({"faz": "a"}), &["\"a\" is not of type \"integer\""])]
    #[test_case(&json!({"faz": "a", "haz": "a"}), &["\"a\" is not of type \"integer\"", "\"a\" is not of type \"integer\""])]
    // `properties.foo` - should be a string & `patternProperties.^bar` - invalid
    #[test_case(&json!({"foo": 3, "bar": 4}), &["4 is less than the minimum of 5", "3 is not of type \"string\""])]
    // `properties.barbaz` - valid; `patternProperties.^bar` - invalid
    #[test_case(&json!({"barbaz": 3}), &["3 is less than the minimum of 5"])]
    // `patternProperties.^bar` (should be >=5)
    #[test_case(&json!({"bar": 4}), &["4 is less than the minimum of 5"])]
    // `patternProperties.spam$` (should be <=10)
    #[test_case(&json!({"spam": 11}), &["11 is greater than the maximum of 10"])]
    // `patternProperties` - both values are invalid
    #[test_case(&json!({"bar": 4, "spam": 11}), &["4 is less than the minimum of 5", "11 is greater than the maximum of 10"])]
    // `patternProperties` - `bar` is valid, `spam` is invalid
    #[test_case(&json!({"bar": 6, "spam": 11}), &["11 is greater than the maximum of 10"])]
    // `patternProperties` - `bar` is invalid, `spam` is valid
    #[test_case(&json!({"bar": 4, "spam": 8}), &["4 is less than the minimum of 5"])]
    // `patternProperties.^bar` - (should be >=5), but valid for `patternProperties.spam$`
    #[test_case(&json!({"barspam": 4}), &["4 is less than the minimum of 5"])]
    // `patternProperties.spam$` - (should be <=10), but valid for `patternProperties.^bar`
    #[test_case(&json!({"barspam": 11}), &["11 is greater than the maximum of 10"])]
    // All combined + valid via `additionalProperties`
    #[test_case(
      &json!({"bar": 4, "spam": 11, "foo": 3, "faz": "a", "fam": 42}),
      &[
         "4 is less than the minimum of 5",
         "\"a\" is not of type \"integer\"",
         "3 is not of type \"string\"",
         "11 is greater than the maximum of 10",
      ]
    )]
    fn schema_5_invalid(instance: &Value, expected: &[&str]) {
        let schema = schema_5();
        tests_util::is_not_valid(&schema, instance);
        tests_util::expect_errors(&schema, instance, expected)
    }

    fn schema_6() -> Value {
        // For `AdditionalPropertiesWithPatternsValidator`
        json!({
            "additionalProperties": {"type": "integer"},
            "patternProperties": {
                "^bar": {"type": "integer", "minimum": 5},
                "spam$": {"type": "integer", "maximum": 10},
            }
        })
    }

    // Another type
    #[test_case(&json!([1]))]
    // The right type
    #[test_case(&json!({}))]
    // Match `additionalProperties`
    #[test_case(&json!({"faz": 42}))]
    // Match `patternProperties.^bar`
    #[test_case(&json!({"bar": 6}))]
    // Match `patternProperties.spam$`
    #[test_case(&json!({"spam": 7}))]
    // All `patternProperties` rules match on different values
    #[test_case(&json!({"bar": 6, "spam": 7}))]
    // All `patternProperties` rules match on the same value
    #[test_case(&json!({"barspam": 7}))]
    // All combined
    #[test_case(&json!({"barspam": 7, "bar": 6, "spam": 7, "faz": 42}))]
    fn schema_6_valid(instance: &Value) {
        let schema = schema_6();
        tests_util::is_valid(&schema, instance)
    }

    // `additionalProperties` - extra keyword that doesn't match `additionalProperties`
    #[test_case(&json!({"faz": "a"}), &["\"a\" is not of type \"integer\""])]
    #[test_case(&json!({"faz": "a", "haz": "a"}), &["\"a\" is not of type \"integer\"", "\"a\" is not of type \"integer\""])]
    // `additionalProperties` - should be an integer & `patternProperties.^bar` - invalid
    #[test_case(&json!({"foo": "a", "bar": 4}), &["4 is less than the minimum of 5", "\"a\" is not of type \"integer\""])]
    // `patternProperties.^bar` (should be >=5)
    #[test_case(&json!({"bar": 4}), &["4 is less than the minimum of 5"])]
    // `patternProperties.spam$` (should be <=10)
    #[test_case(&json!({"spam": 11}), &["11 is greater than the maximum of 10"])]
    // `patternProperties` - both values are invalid
    #[test_case(&json!({"bar": 4, "spam": 11}), &["4 is less than the minimum of 5", "11 is greater than the maximum of 10"])]
    // `patternProperties` - `bar` is valid, `spam` is invalid
    #[test_case(&json!({"bar": 6, "spam": 11}), &["11 is greater than the maximum of 10"])]
    // `patternProperties` - `bar` is invalid, `spam` is valid
    #[test_case(&json!({"bar": 4, "spam": 8}), &["4 is less than the minimum of 5"])]
    // `patternProperties.^bar` - (should be >=5), but valid for `patternProperties.spam$`
    #[test_case(&json!({"barspam": 4}), &["4 is less than the minimum of 5"])]
    // `patternProperties.spam$` - (should be <=10), but valid for `patternProperties.^bar`
    #[test_case(&json!({"barspam": 11}), &["11 is greater than the maximum of 10"])]
    // All combined + valid via `additionalProperties`
    #[test_case(
      &json!({"bar": 4, "spam": 11, "faz": "a", "fam": 42}),
      &[
         "4 is less than the minimum of 5",
         "\"a\" is not of type \"integer\"",
         "11 is greater than the maximum of 10",
      ]
    )]
    fn schema_6_invalid(instance: &Value, expected: &[&str]) {
        let schema = schema_6();
        tests_util::is_not_valid(&schema, instance);
        tests_util::expect_errors(&schema, instance, expected)
    }
}
