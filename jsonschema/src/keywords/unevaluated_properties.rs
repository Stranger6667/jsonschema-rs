use std::sync::Arc;

use crate::{
    compilation::{compile_validators, context::CompilationContext},
    error::{no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    output::BasicOutput,
    paths::{JSONPointer, JsonPointerNode},
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
    additional: Option<UnevaluatedSubvalidator>,
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
        let unevaluated = UnevaluatedSubvalidator::from_value(
            schema,
            &context.with_path("unevaluatedProperties"),
        )?;

        let additional = parent
            .get("additionalProperties")
            .map(|additional_properties| {
                UnevaluatedSubvalidator::from_value(
                    additional_properties,
                    &context.with_path("additionalProperties"),
                )
            })
            .transpose()?;

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

                ConditionalSubvalidator::from_values(schema, condition, success, failure, context)
                    .map(Box::new)
            })
            .transpose()?;

        let dependent = parent
            .get("dependentSchemas")
            .map(|dependent_schemas| {
                DependentSchemaSubvalidator::from_value(schema, dependent_schemas, context)
            })
            .transpose()?;

        let reference = parent
            .get("$ref")
            .map(|reference| ReferenceSubvalidator::from_value(schema, reference, context))
            .transpose()?
            .flatten();

        let mut subschema_validators = vec![];
        if let Some(Value::Array(subschemas)) = parent.get("allOf") {
            let validator = SubschemaSubvalidator::from_values(
                schema,
                subschemas,
                SubschemaBehavior::All,
                context,
            )?;
            subschema_validators.push(validator);
        }

        if let Some(Value::Array(subschemas)) = parent.get("anyOf") {
            let validator = SubschemaSubvalidator::from_values(
                schema,
                subschemas,
                SubschemaBehavior::Any,
                context,
            )?;
            subschema_validators.push(validator);
        }

        if let Some(Value::Array(subschemas)) = parent.get("oneOf") {
            let validator = SubschemaSubvalidator::from_values(
                schema,
                subschemas,
                SubschemaBehavior::One,
                context,
            )?;
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
            additional,
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
                self.additional.as_ref().and_then(|additional| {
                    additional.is_valid_property(property_instance, property_name)
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
        instance_path: &JsonPointerNode,
        property_path: &JsonPointerNode,
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
                self.subschemas.as_ref().and_then(|subschemas| {
                    subschemas.iter().find_map(|subschema| {
                        subschema.validate_property(
                            instance,
                            instance_path,
                            property_path,
                            property_instance,
                            property_name,
                        )
                    })
                })
            })
            .or_else(|| {
                self.additional.as_ref().and_then(|additional| {
                    additional.validate_property(property_path, property_instance, property_name)
                })
            })
            .or_else(|| {
                self.unevaluated
                    .validate_property(property_path, property_instance, property_name)
            })
    }

    fn apply_property<'a>(
        &'a self,
        instance: &Value,
        instance_path: &JsonPointerNode,
        property_path: &JsonPointerNode,
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
                self.additional.as_ref().and_then(|additional| {
                    additional.apply_property(property_path, property_instance, property_name)
                })
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
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if let Value::Object(props) = instance {
            let mut errors = vec![];
            let mut unevaluated = vec![];

            for (property_name, property_instance) in props {
                let property_path = instance_path.push(property_name.as_str());
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
                        unevaluated.push(property_name.to_string());
                    }
                }
            }

            if !unevaluated.is_empty() {
                errors.push(ValidationError::unevaluated_properties(
                    self.schema_path.clone(),
                    instance_path.into(),
                    instance,
                    unevaluated,
                ));
            }

            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }

    fn apply<'a>(
        &'a self,
        instance: &Value,
        instance_path: &JsonPointerNode,
    ) -> PartialApplication<'a> {
        if let Value::Object(props) = instance {
            let mut output = BasicOutput::default();
            let mut unevaluated = vec![];

            for (property_name, property_instance) in props {
                let property_path = instance_path.push(property_name.as_str());
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
                        unevaluated.push(property_name.to_string());
                    }
                }
            }

            let mut result: PartialApplication = output.into();
            if !unevaluated.is_empty() {
                result.mark_errored(
                    ValidationError::unevaluated_properties(
                        self.schema_path.clone(),
                        instance_path.into(),
                        instance,
                        unevaluated,
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
        property_path: &JsonPointerNode,
        property_instance: &'instance Value,
        property_name: &str,
    ) -> Option<ErrorIterator<'instance>> {
        self.prop_map
            .get_key_validator(property_name)
            .map(|(_, node)| node.validate(property_instance, property_path))
    }

    fn apply_property<'a>(
        &'a self,
        property_path: &JsonPointerNode,
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
        property_path: &JsonPointerNode,
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

        had_match.then(|| boxed_errors(errors))
    }

    fn apply_property<'a>(
        &'a self,
        property_path: &JsonPointerNode,
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

/// Subschema validator behavior.
#[derive(Debug)]
enum SubschemaBehavior {
    /// Properties must be valid for all subschemas that would evaluate them.
    All,

    /// Properties must be valid for exactly one subschema that would evaluate them.
    One,

    /// Properties must be valid for at least one subschema that would evaluate them.
    Any,
}

impl SubschemaBehavior {
    const fn as_str(&self) -> &'static str {
        match self {
            SubschemaBehavior::All => "allOf",
            SubschemaBehavior::One => "oneOf",
            SubschemaBehavior::Any => "anyOf",
        }
    }
}

/// A subvalidator for subschema validation such as `allOf`, `oneOf`, and `anyOf`.
#[derive(Debug)]
struct SubschemaSubvalidator {
    behavior: SubschemaBehavior,
    subvalidators: Vec<(SchemaNode, UnevaluatedPropertiesValidator)>,
}

impl SubschemaSubvalidator {
    fn from_values<'a>(
        parent: &'a Value,
        values: &'a [Value],
        behavior: SubschemaBehavior,
        context: &CompilationContext,
    ) -> Result<Self, ValidationError<'a>> {
        let mut subvalidators = vec![];
        let keyword_context = context.with_path(behavior.as_str());

        for (i, value) in values.iter().enumerate() {
            if let Value::Object(subschema) = value {
                let subschema_context = keyword_context.with_path(i);

                let node = compile_validators(value, &subschema_context)?;
                let subvalidator = UnevaluatedPropertiesValidator::compile(
                    subschema,
                    get_transitive_unevaluated_props_schema(subschema, parent),
                    &subschema_context,
                )?;

                subvalidators.push((node, subvalidator));
            }
        }

        Ok(Self {
            behavior,
            subvalidators,
        })
    }

    fn is_valid_property(
        &self,
        instance: &Value,
        property_instance: &Value,
        property_name: &str,
    ) -> Option<bool> {
        let mapped = self.subvalidators.iter().map(|(node, subvalidator)| {
            (
                subvalidator.is_valid_property(instance, property_instance, property_name),
                node.is_valid(instance),
            )
        });

        match self.behavior {
            // The instance must be valid against _all_ subschemas, and the property must be
            // evaluated by at least one subschema.
            SubschemaBehavior::All => {
                let results = mapped.collect::<Vec<_>>();
                let all_subschemas_valid =
                    results.iter().all(|(_, instance_valid)| *instance_valid);
                all_subschemas_valid.then(|| {
                    // We only need to find the first valid evaluation because we know if that
                    // all subschemas were valid against the instance that there can't actually
                    // be any subschemas where the property was evaluated but invalid.
                    results
                        .iter()
                        .any(|(property_result, _)| matches!(property_result, Some(true)))
                })
            }

            // The instance must be valid against only _one_ subschema, and for that subschema, the
            // property must be evaluated by it.
            SubschemaBehavior::One => {
                let mut evaluated_property = None;
                for (property_result, instance_valid) in mapped {
                    if instance_valid {
                        if evaluated_property.is_some() {
                            // We already found a subschema that the instance was valid against, and
                            // had evaluated the property, which means this `oneOf` is not valid
                            // overall, and so the property is not considered evaluated.
                            return None;
                        }

                        evaluated_property = property_result;
                    }
                }

                evaluated_property
            }

            // The instance must be valid against _at least_ one subschema, and for that subschema,
            // the property must be evaluated by it.
            SubschemaBehavior::Any => mapped
                .filter_map(|(property_result, instance_valid)| {
                    instance_valid.then(|| property_result).flatten()
                })
                .find(|x| *x),
        }
    }

    fn validate_property<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
        property_path: &JsonPointerNode,
        property_instance: &'instance Value,
        property_name: &str,
    ) -> Option<ErrorIterator<'instance>> {
        let mapped = self.subvalidators.iter().map(|(node, subvalidator)| {
            let property_result = subvalidator
                .validate_property(
                    instance,
                    instance_path,
                    property_path,
                    property_instance,
                    property_name,
                )
                .map(|errs| errs.collect::<Vec<_>>());

            let instance_result = node.validate(instance, instance_path).collect::<Vec<_>>();

            (property_result, instance_result)
        });

        match self.behavior {
            // The instance must be valid against _all_ subschemas, and the property must be
            // evaluated by at least one subschema. We group the errors for the property itself
            // across all subschemas, though.
            SubschemaBehavior::All => {
                let results = mapped.collect::<Vec<_>>();
                let all_subschemas_valid = results
                    .iter()
                    .all(|(_, instance_errors)| instance_errors.is_empty());
                all_subschemas_valid
                    .then(|| {
                        results
                            .into_iter()
                            .filter_map(|(property_errors, _)| property_errors)
                            .reduce(|mut previous, current| {
                                previous.extend(current);
                                previous
                            })
                            .map(boxed_errors)
                    })
                    .flatten()
            }

            // The instance must be valid against only _one_ subschema, and for that subschema, the
            // property must be evaluated by it.
            SubschemaBehavior::One => {
                let mut evaluated_property_errors = None;
                for (property_errors, instance_errors) in mapped {
                    if instance_errors.is_empty() {
                        if evaluated_property_errors.is_some() {
                            // We already found a subschema that the instance was valid against, and
                            // had evaluated the property, which means this `oneOf` is not valid
                            // overall, and so the property is not considered evaluated.
                            return None;
                        }

                        evaluated_property_errors = property_errors.map(boxed_errors);
                    }
                }

                evaluated_property_errors
            }

            // The instance must be valid against _at least_ one subschema, and for that subschema,
            // the property must be evaluated by it.
            SubschemaBehavior::Any => mapped
                .filter_map(|(property_errors, instance_errors)| {
                    instance_errors
                        .is_empty()
                        .then(|| property_errors)
                        .flatten()
                })
                .filter(|x| x.is_empty())
                .map(boxed_errors)
                .next(),
        }
    }

    fn apply_property<'a>(
        &'a self,
        instance: &Value,
        instance_path: &JsonPointerNode,
        property_path: &JsonPointerNode,
        property_instance: &Value,
        property_name: &str,
    ) -> Option<BasicOutput<'a>> {
        let mapped = self.subvalidators.iter().map(|(node, subvalidator)| {
            let property_result = subvalidator.apply_property(
                instance,
                instance_path,
                property_path,
                property_instance,
                property_name,
            );

            let instance_result = node.apply(instance, instance_path);

            (property_result, instance_result)
        });

        match self.behavior {
            // The instance must be valid against _all_ subschemas, and the property must be
            // evaluated by at least one subschema. We group the errors for the property itself
            // across all subschemas, though.
            SubschemaBehavior::All => {
                let results = mapped.collect::<Vec<_>>();
                let all_subschemas_valid = results
                    .iter()
                    .all(|(_, instance_output)| instance_output.is_valid());
                all_subschemas_valid
                    .then(|| {
                        results
                            .into_iter()
                            .filter_map(|(property_output, _)| property_output)
                            .reduce(|mut previous, current| {
                                previous += current;
                                previous
                            })
                    })
                    .flatten()
            }

            // The instance must be valid against only _one_ subschema, and for that subschema, the
            // property must be evaluated by it.
            SubschemaBehavior::One => {
                let mut evaluated_property_output = None;
                for (property_output, instance_output) in mapped {
                    if instance_output.is_valid() {
                        if evaluated_property_output.is_some() {
                            // We already found a subschema that the instance was valid against, and
                            // had evaluated the property, which means this `oneOf` is not valid
                            // overall, and so the property is not considered evaluated.
                            return None;
                        }

                        evaluated_property_output = property_output;
                    }
                }

                evaluated_property_output
            }

            // The instance must be valid against _at least_ one subschema, and for that subschema,
            // the property must be evaluated by it.
            SubschemaBehavior::Any => mapped
                .filter_map(|(property_output, instance_output)| {
                    instance_output
                        .is_valid()
                        .then(|| property_output)
                        .flatten()
                })
                .find(|x| x.is_valid()),
        }
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
        value: &'a Value,
        context: &CompilationContext,
    ) -> Result<Self, ValidationError<'a>> {
        let behavior = match value {
            Value::Bool(false) => UnevaluatedBehavior::Deny,
            Value::Bool(true) => UnevaluatedBehavior::Allow,
            _ => UnevaluatedBehavior::IfValid(compile_validators(value, context)?),
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
        property_path: &JsonPointerNode,
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
        property_path: &JsonPointerNode,
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
        parent: &'a Value,
        schema: &'a Value,
        success: Option<&'a Value>,
        failure: Option<&'a Value>,
        context: &CompilationContext,
    ) -> Result<Self, ValidationError<'a>> {
        let if_context = context.with_path("if");
        compile_validators(schema, &if_context).and_then(|condition| {
            let node = schema
                .as_object()
                .map(|node_schema| {
                    UnevaluatedPropertiesValidator::compile(
                        node_schema,
                        get_transitive_unevaluated_props_schema(node_schema, parent),
                        &if_context,
                    )
                })
                .transpose()?;
            let success = success
                .and_then(|value| value.as_object())
                .map(|success_schema| {
                    UnevaluatedPropertiesValidator::compile(
                        success_schema,
                        get_transitive_unevaluated_props_schema(success_schema, parent),
                        &context.with_path("then"),
                    )
                })
                .transpose()?;
            let failure = failure
                .and_then(|value| value.as_object())
                .map(|failure_schema| {
                    UnevaluatedPropertiesValidator::compile(
                        failure_schema,
                        get_transitive_unevaluated_props_schema(failure_schema, parent),
                        &context.with_path("else"),
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
        instance_path: &JsonPointerNode,
        property_path: &JsonPointerNode,
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
        instance_path: &JsonPointerNode,
        property_path: &JsonPointerNode,
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
        parent: &'a Value,
        value: &'a Value,
        context: &CompilationContext,
    ) -> Result<Self, ValidationError<'a>> {
        let keyword_context = context.with_path("dependentSchemas");
        let schemas = value
            .as_object()
            .ok_or_else(|| unexpected_type(&keyword_context, value, PrimitiveType::Object))?;
        let mut nodes = AHashMap::new();

        for (dependent_property_name, dependent_schema) in schemas {
            let dependent_schema = dependent_schema
                .as_object()
                .ok_or_else(ValidationError::null_schema)?;

            let schema_context = keyword_context.with_path(dependent_property_name.as_str());
            let node = UnevaluatedPropertiesValidator::compile(
                dependent_schema,
                get_transitive_unevaluated_props_schema(dependent_schema, parent),
                &schema_context,
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
        instance_path: &JsonPointerNode,
        property_path: &JsonPointerNode,
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
        instance_path: &JsonPointerNode,
        property_path: &JsonPointerNode,
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
        parent: &'a Value,
        value: &'a Value,
        context: &CompilationContext,
    ) -> Result<Option<Self>, ValidationError<'a>> {
        let keyword_context = context.with_path("$ref");
        let reference = value
            .as_str()
            .ok_or_else(|| unexpected_type(&keyword_context, value, PrimitiveType::String))?;

        let reference_url = context.build_url(reference)?;
        let (scope, resolved) = keyword_context
            .resolver
            .resolve_fragment(keyword_context.config.draft(), &reference_url, reference)
            .map_err(|e| e.into_owned())?;

        let mut ref_context = CompilationContext::new(
            scope.into(),
            Arc::clone(&context.config),
            Arc::clone(&context.resolver),
        );
        ref_context.schema_path = keyword_context.schema_path.clone();

        resolved
            .as_object()
            .map(|ref_schema| {
                UnevaluatedPropertiesValidator::compile(
                    ref_schema,
                    get_transitive_unevaluated_props_schema(ref_schema, parent),
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
        instance_path: &JsonPointerNode,
        property_path: &JsonPointerNode,
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
        instance_path: &JsonPointerNode,
        property_path: &JsonPointerNode,
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

fn get_transitive_unevaluated_props_schema<'a>(
    leaf: &'a Map<String, Value>,
    parent: &'a Value,
) -> &'a Value {
    // We first try and if the leaf schema has `unevaluatedProperties` defined, and if so, we use
    // that. Otherwise, we fallback to the parent schema value, which is the value of
    // `unevaluatedProperties` as defined at the level of the schema right where `leaf_schema`
    // lives.
    leaf.get("unevaluatedProperties").unwrap_or(parent)
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

fn boxed_errors<'a>(errors: Vec<ValidationError<'a>>) -> ErrorIterator<'a> {
    let boxed_errors: ErrorIterator<'a> = Box::new(errors.into_iter());
    boxed_errors
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

#[cfg(test)]
mod tests {
    use crate::{tests_util, Draft};
    use serde_json::json;

    #[cfg(all(feature = "draft201909", not(feature = "draft202012")))]
    const fn get_draft_version() -> Draft {
        Draft::Draft201909
    }

    #[cfg(all(feature = "draft202012", not(feature = "draft201909")))]
    const fn get_draft_version() -> Draft {
        Draft::Draft202012
    }

    #[cfg(all(feature = "draft201909", feature = "draft202012"))]
    const fn get_draft_version() -> Draft {
        Draft::Draft202012
    }

    #[test]
    fn one_of() {
        tests_util::is_valid_with_draft(
            get_draft_version(),
            &json!({
                "oneOf": [
                    { "properties": { "foo": { "const": "bar" } } },
                    { "properties": { "foo": { "const": "quux" } } }
                ],
                "unevaluatedProperties": false
            }),
            &json!({ "foo": "quux" }),
        )
    }

    #[test]
    fn any_of() {
        tests_util::is_valid_with_draft(
            get_draft_version(),
            &json!({
                "anyOf": [
                    { "properties": { "foo": { "minLength": 10 } } },
                    { "properties": { "foo": { "type": "string" } } }
                ],
                "unevaluatedProperties": false
            }),
            &json!({ "foo": "rut roh" }),
        )
    }

    #[test]
    fn all_of() {
        tests_util::is_not_valid_with_draft(
            get_draft_version(),
            &json!({
                "allOf": [
                    { "properties": { "foo": { "type": "string" } } },
                    { "properties": { "foo": { "minLength": 10 } } }
                ],
                "unevaluatedProperties": false
            }),
            &json!({ "foo": "rut roh" }),
        )
    }

    #[test]
    fn all_of_with_additional_props_subschema() {
        let schema = json!({
            "allOf": [
                {
                    "type": "object",
                    "required": [
                        "foo"
                    ],
                    "properties": {
                        "foo": { "type": "string" }
                    }
                },
                {
                    "type": "object",
                    "additionalProperties": { "type": "string" }
                }
            ],
            "unevaluatedProperties": false
        });

        tests_util::is_valid_with_draft(
            get_draft_version(),
            &schema,
            &json!({ "foo": "wee", "another": "thing" }),
        );

        tests_util::is_not_valid_with_draft(
            get_draft_version(),
            &schema,
            &json!({ "foo": "wee", "another": false }),
        );
    }
}
