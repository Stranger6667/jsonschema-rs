use std::sync::Arc;

use crate::{
    compilation::{compile_validators, context::CompilationContext},
    error::{no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    output::BasicOutput,
    paths::{InstancePath, JSONPointer},
    primitive_type::PrimitiveType,
    properties::*,
    schema_node::SchemaNode,
    validator::{PartialApplication, Validate},
};
use ahash::AHashMap;
use serde_json::{Map, Value};

/// A validator for unevaluated properties.
///
/// In contrast to `additionalProperties`, which can only be used for properties defined in a schema
/// of type `object`, `unevaluatedProperties` can "see through" advanced validation features like
/// subschema validation (`allOf`, `oneOf`, `anyOf`), conditional validation (`if`/`then`/`else`),
/// dependent schemas (`dependentSchemas`), and schema references (`$ref`), which allows applying
/// `additionalProperties`-like behavior to schemas which use the aforementioned advanced validation
/// keywords.
#[derive(Debug)]
struct UnevaluatedPropertiesValidator {
    schema_path: JSONPointer,
    unevaluated: UnevaluatedSubvalidator,
    properties: Option<PropertySubvalidator>,
    patterns: Option<PatternSubvalidator>,
    conditional: Option<Box<ConditionalSubvalidator>>,
    dependent: Option<DependentSchemaSubvalidator>,
    reference: Option<ReferenceSubvalidator>,
    subschemas: Option<Vec<SubschemaSubvalidator>>,
}

impl UnevaluatedPropertiesValidator {
    fn compile<'a>(
        parent: &'a Map<String, Value>,
        schema: &'a Value,
        context: &CompilationContext,
    ) -> Result<Self, ValidationError<'a>> {
        let unevaluated = UnevaluatedSubvalidator::from_value(parent, schema, context)?;

        let properties = parent
            .get("properties")
            .map(|properties| PropertySubvalidator::from_value(properties, context))
            .transpose()?;
        let patterns = parent
            .get("patternProperties")
            .map(|pattern_properties| PatternSubvalidator::from_value(pattern_properties, context))
            .transpose()?;

        let conditional = parent
            .get("if")
            .map(|condition| {
                let success = parent.get("then");
                let failure = parent.get("else");

                ConditionalSubvalidator::from_values(condition, success, failure, context)
                    .map(Box::new)
            })
            .transpose()?;

        let dependent = parent
            .get("dependentSchemas")
            .map(|dependent_schemas| {
                DependentSchemaSubvalidator::from_value(dependent_schemas, context)
            })
            .transpose()?;

        let reference = parent
            .get("$ref")
            .map(|reference| ReferenceSubvalidator::from_value(reference, context))
            .transpose()?
            .flatten();

        let mut subschema_validators = vec![];
        if let Some(Value::Array(subschemas)) = parent.get("allOf") {
            let validator = SubschemaSubvalidator::from_values(subschemas, context)?;
            subschema_validators.push(validator);
        }

        if let Some(Value::Array(subschemas)) = parent.get("anyOf") {
            let validator = SubschemaSubvalidator::from_values(subschemas, context)?;
            subschema_validators.push(validator);
        }

        if let Some(Value::Array(subschemas)) = parent.get("oneOf") {
            let validator = SubschemaSubvalidator::from_values(subschemas, context)?;
            subschema_validators.push(validator);
        }

        let subschemas = if subschema_validators.is_empty() {
            None
        } else {
            Some(subschema_validators)
        };

        Ok(Self {
            schema_path: JSONPointer::from(&context.schema_path),
            unevaluated,
            properties,
            patterns,
            conditional,
            dependent,
            reference,
            subschemas,
        })
    }

    fn is_valid_property(
        &self,
        instance: &Value,
        property_instance: &Value,
        property_name: &str,
    ) -> Option<bool> {
        self.properties
            .as_ref()
            .and_then(|prop_map| prop_map.is_valid_property(property_instance, property_name))
            .or_else(|| {
                self.patterns.as_ref().and_then(|patterns| {
                    patterns.is_valid_property(property_instance, property_name)
                })
            })
            .or_else(|| {
                self.conditional.as_ref().and_then(|conditional| {
                    conditional.is_valid_property(instance, property_instance, property_name)
                })
            })
            .or_else(|| {
                self.dependent.as_ref().and_then(|dependent| {
                    dependent.is_valid_property(instance, property_instance, property_name)
                })
            })
            .or_else(|| {
                self.reference.as_ref().and_then(|reference| {
                    reference.is_valid_property(instance, property_instance, property_name)
                })
            })
            .or_else(|| {
                self.subschemas.as_ref().and_then(|subschemas| {
                    subschemas.iter().find_map(|subschema| {
                        subschema.is_valid_property(instance, property_instance, property_name)
                    })
                })
            })
            .or_else(|| {
                self.unevaluated
                    .is_valid_property(property_instance, property_name)
            })
    }

    fn validate_property<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &InstancePath,
        property_path: &InstancePath,
        property_instance: &'instance Value,
        property_name: &str,
    ) -> Option<ErrorIterator<'instance>> {
        self.properties
            .as_ref()
            .and_then(|prop_map| {
                prop_map.validate_property(property_path, property_instance, property_name)
            })
            .or_else(|| {
                self.patterns.as_ref().and_then(|patterns| {
                    patterns.validate_property(property_path, property_instance, property_name)
                })
            })
            .or_else(|| {
                self.conditional.as_ref().and_then(|conditional| {
                    conditional.validate_property(
                        instance,
                        instance_path,
                        property_path,
                        property_instance,
                        property_name,
                    )
                })
            })
            .or_else(|| {
                self.dependent.as_ref().and_then(|dependent| {
                    dependent.validate_property(
                        instance,
                        instance_path,
                        property_path,
                        property_instance,
                        property_name,
                    )
                })
            })
            .or_else(|| {
                self.reference.as_ref().and_then(|reference| {
                    reference.validate_property(
                        instance,
                        instance_path,
                        property_path,
                        property_instance,
                        property_name,
                    )
                })
            })
            .or_else(|| {
                let result = self.subschemas.as_ref().and_then(|subschemas| {
                    subschemas.iter().find_map(|subschema| {
                        subschema.validate_property(
                            instance,
                            instance_path,
                            property_path,
                            property_instance,
                            property_name,
                        )
                    })
                });

                result
            })
            .or_else(|| {
                self.unevaluated
                    .validate_property(property_path, property_instance, property_name)
            })
    }

    fn apply_property<'a>(
        &'a self,
        instance: &Value,
        instance_path: &InstancePath,
        property_path: &InstancePath,
        property_instance: &Value,
        property_name: &str,
    ) -> Option<BasicOutput<'a>> {
        self.properties
            .as_ref()
            .and_then(|prop_map| {
                prop_map.apply_property(property_path, property_instance, property_name)
            })
            .or_else(|| {
                self.patterns.as_ref().and_then(|patterns| {
                    patterns.apply_property(property_path, property_instance, property_name)
                })
            })
            .or_else(|| {
                self.conditional.as_ref().and_then(|conditional| {
                    conditional.apply_property(
                        instance,
                        instance_path,
                        property_path,
                        property_instance,
                        property_name,
                    )
                })
            })
            .or_else(|| {
                self.dependent.as_ref().and_then(|dependent| {
                    dependent.apply_property(
                        instance,
                        instance_path,
                        property_path,
                        property_instance,
                        property_name,
                    )
                })
            })
            .or_else(|| {
                self.reference.as_ref().and_then(|reference| {
                    reference.apply_property(
                        instance,
                        instance_path,
                        property_path,
                        property_instance,
                        property_name,
                    )
                })
            })
            .or_else(|| {
                let result = self.subschemas.as_ref().and_then(|subschemas| {
                    subschemas.iter().find_map(|subschema| {
                        subschema.apply_property(
                            instance,
                            instance_path,
                            property_path,
                            property_instance,
                            property_name,
                        )
                    })
                });

                result
            })
            .or_else(|| {
                self.unevaluated
                    .apply_property(property_path, property_instance, property_name)
            })
    }
}

impl Validate for UnevaluatedPropertiesValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Object(props) = instance {
            props.iter().all(|(property_name, property_instance)| {
                self.is_valid_property(instance, property_instance, property_name)
                    .unwrap_or(false)
            })
        } else {
            true
        }
    }

    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'instance> {
        if let Value::Object(props) = instance {
            let mut errors = vec![];
            let mut unexpected = vec![];

            for (property_name, property_instance) in props {
                let property_path = instance_path.push(property_name.clone());
                let maybe_property_errors = self.validate_property(
                    instance,
                    instance_path,
                    &property_path,
                    property_instance,
                    property_name,
                );

                match maybe_property_errors {
                    Some(property_errors) => errors.extend(property_errors),
                    None => {
                        // If we can't validate, that means that "unevaluatedProperties" is
                        // "false", which means that this property was not expected.
                        unexpected.push(property_name.to_string());
                    }
                }
            }

            if !unexpected.is_empty() {
                errors.push(ValidationError::unevaluated_properties(
                    self.schema_path.clone(),
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

    fn apply<'a>(
        &'a self,
        instance: &Value,
        instance_path: &InstancePath,
    ) -> PartialApplication<'a> {
        if let Value::Object(props) = instance {
            let mut output = BasicOutput::default();
            let mut unexpected = vec![];

            for (property_name, property_instance) in props {
                let property_path = instance_path.push(property_name.clone());
                let maybe_property_output = self.apply_property(
                    instance,
                    instance_path,
                    &property_path,
                    property_instance,
                    property_name,
                );

                match maybe_property_output {
                    Some(property_output) => output += property_output,
                    None => {
                        // If we can't validate, that means that "unevaluatedProperties" is
                        // "false", which means that this property was not expected.
                        unexpected.push(property_name.to_string());
                    }
                }
            }

            let mut result: PartialApplication = output.into();
            if !unexpected.is_empty() {
                result.mark_errored(
                    ValidationError::unevaluated_properties(
                        self.schema_path.clone(),
                        instance_path.into(),
                        instance,
                        unexpected,
                    )
                    .into(),
                )
            }
            result
        } else {
            PartialApplication::valid_empty()
        }
    }
}

impl core::fmt::Display for UnevaluatedPropertiesValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        "unevaluatedProperties: {}".fmt(f)
    }
}

/// A subvalidator for properties.
#[derive(Debug)]
struct PropertySubvalidator {
    prop_map: SmallValidatorsMap,
}

impl PropertySubvalidator {
    fn from_value<'a>(
        properties: &'a Value,
        context: &CompilationContext,
    ) -> Result<Self, ValidationError<'a>> {
        properties
            .as_object()
            .ok_or_else(ValidationError::null_schema)
            .and_then(|props| SmallValidatorsMap::from_map(props, context))
            .map(|prop_map| Self { prop_map })
    }

    fn is_valid_property(&self, property_instance: &Value, property_name: &str) -> Option<bool> {
        self.prop_map
            .get_validator(property_name)
            .map(|node| node.is_valid(property_instance))
    }

    fn validate_property<'instance>(
        &self,
        property_path: &InstancePath,
        property_instance: &'instance Value,
        property_name: &str,
    ) -> Option<ErrorIterator<'instance>> {
        self.prop_map
            .get_key_validator(property_name)
            .map(|(_, node)| node.validate(property_instance, property_path))
    }

    fn apply_property<'a>(
        &'a self,
        property_path: &InstancePath,
        property_instance: &Value,
        property_name: &str,
    ) -> Option<BasicOutput<'a>> {
        self.prop_map
            .get_key_validator(property_name)
            .map(|(_, node)| node.apply_rooted(property_instance, property_path))
    }
}

/// A subvalidator for pattern properties.
#[derive(Debug)]
struct PatternSubvalidator {
    patterns: PatternedValidators,
}

impl PatternSubvalidator {
    fn from_value<'a>(
        properties: &'a Value,
        context: &CompilationContext,
    ) -> Result<Self, ValidationError<'a>> {
        properties
            .as_object()
            .ok_or_else(ValidationError::null_schema)
            .and_then(|props| compile_patterns(props, context))
            .map(|patterns| Self { patterns })
    }

    fn is_valid_property(&self, property_instance: &Value, property_name: &str) -> Option<bool> {
        let mut had_match = false;

        for (pattern, node) in &self.patterns {
            if pattern.is_match(property_name).unwrap_or(false) {
                had_match = true;

                if !node.is_valid(property_instance) {
                    return Some(false);
                }
            }
        }

        had_match.then(|| true)
    }

    fn validate_property<'instance>(
        &self,
        property_path: &InstancePath,
        property_instance: &'instance Value,
        property_name: &str,
    ) -> Option<ErrorIterator<'instance>> {
        let mut had_match = false;
        let mut errors = vec![];

        for (pattern, node) in &self.patterns {
            if pattern.is_match(property_name).unwrap_or(false) {
                had_match = true;

                errors.extend(node.validate(property_instance, property_path));
            }
        }

        let errors: ErrorIterator<'instance> = Box::new(errors.into_iter());
        had_match.then(|| errors)
    }

    fn apply_property<'a>(
        &'a self,
        property_path: &InstancePath,
        property_instance: &Value,
        property_name: &str,
    ) -> Option<BasicOutput<'a>> {
        let mut had_match = false;
        let mut output = BasicOutput::default();

        for (pattern, node) in &self.patterns {
            if pattern.is_match(property_name).unwrap_or(false) {
                had_match = true;

                let pattern_output = node.apply_rooted(property_instance, property_path);
                output += pattern_output;
            }
        }

        had_match.then(|| output)
    }
}

/// A subvalidator for subschema validation such as `allOf`, `oneOf`, and `anyOf`.
///
/// Unlike the validation logic for `allOf`/`oneOf`/`anyOf` themselves, this subvalidator searches
/// configured subvalidators in a first-match-wins process. For example, a property will be
/// considered evaluated against subschemas defined via `oneOf` so long as one subschema would evaluate
/// the property, even if, say, more than one subschema in `oneOf` is technically valid, which would
/// otherwise be a failure for validation of `oneOf` in and of itself.
#[derive(Debug)]
struct SubschemaSubvalidator {
    subvalidators: Vec<UnevaluatedPropertiesValidator>,
}

impl SubschemaSubvalidator {
    fn from_values<'a>(
        values: &'a [Value],
        context: &CompilationContext,
    ) -> Result<Self, ValidationError<'a>> {
        let mut subvalidators = vec![];
        for value in values {
            if let Value::Object(subschema) = value {
                let subvalidator = UnevaluatedPropertiesValidator::compile(
                    subschema,
                    get_unevaluated_props_schema(subschema),
                    context,
                )?;
                subvalidators.push(subvalidator);
            }
        }

        Ok(Self { subvalidators })
    }

    fn is_valid_property(
        &self,
        instance: &Value,
        property_instance: &Value,
        property_name: &str,
    ) -> Option<bool> {
        self.subvalidators.iter().find_map(|subvalidator| {
            subvalidator.is_valid_property(instance, property_instance, property_name)
        })
    }

    fn validate_property<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &InstancePath,
        property_path: &InstancePath,
        property_instance: &'instance Value,
        property_name: &str,
    ) -> Option<ErrorIterator<'instance>> {
        self.subvalidators.iter().find_map(|subvalidator| {
            subvalidator.validate_property(
                instance,
                instance_path,
                property_path,
                property_instance,
                property_name,
            )
        })
    }

    fn apply_property<'a>(
        &'a self,
        instance: &Value,
        instance_path: &InstancePath,
        property_path: &InstancePath,
        property_instance: &Value,
        property_name: &str,
    ) -> Option<BasicOutput<'a>> {
        self.subvalidators.iter().find_map(|subvalidator| {
            subvalidator.apply_property(
                instance,
                instance_path,
                property_path,
                property_instance,
                property_name,
            )
        })
    }
}

/// Unevaluated properties behavior.
#[derive(Debug)]
enum UnevaluatedBehavior {
    /// Unevaluated properties are allowed, regardless of instance value.
    Allow,

    /// Unevaluated properties are not allowed, regardless of instance value.
    Deny,

    /// Unevaluated properties are allowed, so long as the instance is valid against the given
    /// schema.
    IfValid(SchemaNode),
}

/// A subvalidator for unevaluated properties.
#[derive(Debug)]
struct UnevaluatedSubvalidator {
    behavior: UnevaluatedBehavior,
}

impl UnevaluatedSubvalidator {
    fn from_value<'a>(
        parent: &'a Map<String, Value>,
        value: &'a Value,
        context: &CompilationContext,
    ) -> Result<Self, ValidationError<'a>> {
        // We also examine the value of `additionalProperties` here, if present, because if it's
        // specified as `true`, it can potentially override the behavior of the validator depending
        // on the value of `unevaluatedProperties`.
        //
        // TODO: We probably need to think about this more because `unevaluatedProperties` affects
        // subschema validation, when really all we want to have this do (based on the JSON Schema
        // test suite cases) is disable the `unevaluatedProperties: false` bit _just_ for normal
        // properties on the top-level instance.
        let additional_properties = parent.get("additionalProperties");
        let behavior = match (value, additional_properties) {
            (Value::Bool(false), None) | (Value::Bool(false), Some(Value::Bool(false))) => {
                UnevaluatedBehavior::Deny
            }
            (Value::Bool(true), _) | (Value::Bool(false), Some(Value::Bool(true))) => {
                UnevaluatedBehavior::Allow
            }
            _ => UnevaluatedBehavior::IfValid(compile_validators(
                value,
                &context.with_path("unevaluatedProperties"),
            )?),
        };

        Ok(Self { behavior })
    }

    fn is_valid_property(&self, property_instance: &Value, _property_name: &str) -> Option<bool> {
        match &self.behavior {
            UnevaluatedBehavior::Allow => Some(true),
            UnevaluatedBehavior::Deny => None,
            UnevaluatedBehavior::IfValid(node) => Some(node.is_valid(property_instance)),
        }
    }

    fn validate_property<'instance>(
        &self,
        property_path: &InstancePath,
        property_instance: &'instance Value,
        _property_name: &str,
    ) -> Option<ErrorIterator<'instance>> {
        match &self.behavior {
            UnevaluatedBehavior::Allow => Some(no_error()),
            UnevaluatedBehavior::Deny => None,
            UnevaluatedBehavior::IfValid(node) => {
                Some(node.validate(property_instance, property_path))
            }
        }
    }

    fn apply_property<'a>(
        &'a self,
        property_path: &InstancePath,
        property_instance: &Value,
        _property_name: &str,
    ) -> Option<BasicOutput<'a>> {
        match &self.behavior {
            UnevaluatedBehavior::Allow => Some(BasicOutput::default()),
            UnevaluatedBehavior::Deny => None,
            UnevaluatedBehavior::IfValid(node) => {
                Some(node.apply_rooted(property_instance, property_path))
            }
        }
    }
}

/// A subvalidator for any conditional subschemas.
///
/// This subvalidator handles any subschemas specified via `if`, and handles both the `then` case
/// (`success`) and `else` case (`failure`).
#[derive(Debug)]
struct ConditionalSubvalidator {
    // Validator created from the `if` schema to actually validate the given instance and
    // determine whether or not to check the `then` or `else` schemas, if defined.
    condition: SchemaNode,

    // Validator for checking if the `if` schema evaluates a particular property.
    node: Option<UnevaluatedPropertiesValidator>,

    success: Option<UnevaluatedPropertiesValidator>,
    failure: Option<UnevaluatedPropertiesValidator>,
}

impl ConditionalSubvalidator {
    fn from_values<'a>(
        schema: &'a Value,
        success: Option<&'a Value>,
        failure: Option<&'a Value>,
        context: &CompilationContext,
    ) -> Result<Self, ValidationError<'a>> {
        compile_validators(schema, context).and_then(|condition| {
            let node = schema
                .as_object()
                .map(|parent| {
                    UnevaluatedPropertiesValidator::compile(
                        parent,
                        get_unevaluated_props_schema(parent),
                        context,
                    )
                })
                .transpose()?;
            let success = success
                .and_then(|value| value.as_object())
                .map(|parent| {
                    UnevaluatedPropertiesValidator::compile(
                        parent,
                        get_unevaluated_props_schema(parent),
                        context,
                    )
                })
                .transpose()?;
            let failure = failure
                .and_then(|value| value.as_object())
                .map(|parent| {
                    UnevaluatedPropertiesValidator::compile(
                        parent,
                        get_unevaluated_props_schema(parent),
                        context,
                    )
                })
                .transpose()?;

            Ok(Self {
                condition,
                node,
                success,
                failure,
            })
        })
    }

    fn is_valid_property(
        &self,
        instance: &Value,
        property_instance: &Value,
        property_name: &str,
    ) -> Option<bool> {
        self.node
            .as_ref()
            .and_then(|node| node.is_valid_property(instance, property_instance, property_name))
            .or_else(|| {
                if self.condition.is_valid(instance) {
                    self.success.as_ref().and_then(|success| {
                        success.is_valid_property(instance, property_instance, property_name)
                    })
                } else {
                    self.failure.as_ref().and_then(|failure| {
                        failure.is_valid_property(instance, property_instance, property_name)
                    })
                }
            })
    }

    fn validate_property<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &InstancePath,
        property_path: &InstancePath,
        property_instance: &'instance Value,
        property_name: &str,
    ) -> Option<ErrorIterator<'instance>> {
        self.node
            .as_ref()
            .and_then(|node| {
                node.validate_property(
                    instance,
                    instance_path,
                    property_path,
                    property_instance,
                    property_name,
                )
            })
            .or_else(|| {
                if self.condition.validate(instance, instance_path).count() == 0 {
                    self.success.as_ref().and_then(|success| {
                        success.validate_property(
                            instance,
                            instance_path,
                            property_path,
                            property_instance,
                            property_name,
                        )
                    })
                } else {
                    self.failure.as_ref().and_then(|failure| {
                        failure.validate_property(
                            instance,
                            instance_path,
                            property_path,
                            property_instance,
                            property_name,
                        )
                    })
                }
            })
    }

    fn apply_property<'a>(
        &'a self,
        instance: &Value,
        instance_path: &InstancePath,
        property_path: &InstancePath,
        property_instance: &Value,
        property_name: &str,
    ) -> Option<BasicOutput<'a>> {
        self.node
            .as_ref()
            .and_then(|node| {
                node.apply_property(
                    instance,
                    instance_path,
                    property_path,
                    property_instance,
                    property_name,
                )
            })
            .or_else(|| {
                let partial = self.condition.apply(instance, instance_path);
                if partial.is_valid() {
                    self.success.as_ref().and_then(|success| {
                        success.apply_property(
                            instance,
                            instance_path,
                            property_path,
                            property_instance,
                            property_name,
                        )
                    })
                } else {
                    self.failure.as_ref().and_then(|failure| {
                        failure.apply_property(
                            instance,
                            instance_path,
                            property_path,
                            property_instance,
                            property_name,
                        )
                    })
                }
            })
    }
}

/// A subvalidator for dependent schemas.
#[derive(Debug)]
struct DependentSchemaSubvalidator {
    nodes: AHashMap<String, UnevaluatedPropertiesValidator>,
}

impl DependentSchemaSubvalidator {
    fn from_value<'a>(
        value: &'a Value,
        context: &CompilationContext,
    ) -> Result<Self, ValidationError<'a>> {
        let schemas = value
            .as_object()
            .ok_or_else(|| unexpected_type(context, value, PrimitiveType::Object))?;
        let mut nodes = AHashMap::new();
        for (dependent_property_name, dependent_schema) in schemas {
            let parent = dependent_schema
                .as_object()
                .ok_or_else(ValidationError::null_schema)?;

            let node = UnevaluatedPropertiesValidator::compile(
                parent,
                get_unevaluated_props_schema(parent),
                context,
            )?;
            nodes.insert(dependent_property_name.to_string(), node);
        }

        Ok(Self { nodes })
    }

    fn is_valid_property(
        &self,
        instance: &Value,
        property_instance: &Value,
        property_name: &str,
    ) -> Option<bool> {
        self.nodes
            .iter()
            .find_map(|(dependent_property_name, node)| {
                value_has_object_key(instance, dependent_property_name)
                    .then(|| node)
                    .and_then(|node| {
                        node.is_valid_property(instance, property_instance, property_name)
                    })
            })
    }

    fn validate_property<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &InstancePath,
        property_path: &InstancePath,
        property_instance: &'instance Value,
        property_name: &str,
    ) -> Option<ErrorIterator<'instance>> {
        self.nodes
            .iter()
            .find_map(|(dependent_property_name, node)| {
                value_has_object_key(instance, dependent_property_name)
                    .then(|| node)
                    .and_then(|node| {
                        node.validate_property(
                            instance,
                            instance_path,
                            property_path,
                            property_instance,
                            property_name,
                        )
                    })
            })
    }

    fn apply_property<'a>(
        &'a self,
        instance: &Value,
        instance_path: &InstancePath,
        property_path: &InstancePath,
        property_instance: &Value,
        property_name: &str,
    ) -> Option<BasicOutput<'a>> {
        self.nodes
            .iter()
            .find_map(|(dependent_property_name, node)| {
                value_has_object_key(instance, dependent_property_name)
                    .then(|| node)
                    .and_then(|node| {
                        node.apply_property(
                            instance,
                            instance_path,
                            property_path,
                            property_instance,
                            property_name,
                        )
                    })
            })
    }
}

/// A subvalidator for a top-level schema reference. (`$ref`)
#[derive(Debug)]
struct ReferenceSubvalidator {
    node: Box<UnevaluatedPropertiesValidator>,
}

impl ReferenceSubvalidator {
    fn from_value<'a>(
        value: &'a Value,
        context: &CompilationContext,
    ) -> Result<Option<Self>, ValidationError<'a>> {
        let reference = value
            .as_str()
            .ok_or_else(|| unexpected_type(context, value, PrimitiveType::String))?;

        let reference_url = context.build_url(reference)?;
        let (scope, resolved) = context
            .resolver
            .resolve_fragment(context.config.draft(), &reference_url, reference)
            .map_err(|e| e.into_owned())?;

        let ref_context = CompilationContext::new(
            scope.into(),
            Arc::clone(&context.config),
            Arc::clone(&context.resolver),
        );

        resolved
            .as_object()
            .map(|parent| {
                UnevaluatedPropertiesValidator::compile(
                    parent,
                    get_unevaluated_props_schema(parent),
                    &ref_context,
                )
                .map(|validator| ReferenceSubvalidator {
                    node: Box::new(validator),
                })
                .map_err(|e| e.into_owned())
            })
            .transpose()
    }

    fn is_valid_property(
        &self,
        instance: &Value,
        property_instance: &Value,
        property_name: &str,
    ) -> Option<bool> {
        self.node
            .is_valid_property(instance, property_instance, property_name)
    }

    fn validate_property<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &InstancePath,
        property_path: &InstancePath,
        property_instance: &'instance Value,
        property_name: &str,
    ) -> Option<ErrorIterator<'instance>> {
        self.node.validate_property(
            instance,
            instance_path,
            property_path,
            property_instance,
            property_name,
        )
    }

    fn apply_property<'a>(
        &'a self,
        instance: &Value,
        instance_path: &InstancePath,
        property_path: &InstancePath,
        property_instance: &Value,
        property_name: &str,
    ) -> Option<BasicOutput<'a>> {
        self.node.apply_property(
            instance,
            instance_path,
            property_path,
            property_instance,
            property_name,
        )
    }
}

fn value_has_object_key(value: &Value, key: &str) -> bool {
    match value {
        Value::Object(map) => map.contains_key(key),
        _ => false,
    }
}

fn get_unevaluated_props_schema(parent: &Map<String, Value>) -> &Value {
    parent
        .get("unevaluatedProperties")
        .unwrap_or(&Value::Bool(false))
}

pub(crate) fn compile<'a>(
    parent: &'a Map<String, Value>,
    schema: &'a Value,
    context: &CompilationContext,
) -> Option<CompilationResult<'a>> {
    // Nothing to validate if `unevaluatedProperties` is set to `true`, which is the default:
    if let Value::Bool(true) = schema {
        return None;
    }

    match UnevaluatedPropertiesValidator::compile(parent, schema, context) {
        Ok(compiled) => Some(Ok(Box::new(compiled))),
        Err(e) => Some(Err(e)),
    }
}

fn unexpected_type<'a>(
    context: &CompilationContext,
    instance: &'a Value,
    expected_type: PrimitiveType,
) -> ValidationError<'a> {
    ValidationError::single_type_error(
        JSONPointer::default(),
        context.clone().into_pointer(),
        instance,
        expected_type,
    )
}
