use crate::{
    compiler,
    error::{no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    node::SchemaNode,
    output::BasicOutput,
    paths::{JsonPointer, JsonPointerNode},
    primitive_type::PrimitiveType,
    properties::*,
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
    schema_path: JsonPointer,
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
        ctx: &compiler::Context,
        parent: &'a Map<String, Value>,
        schema: &'a Value,
    ) -> Result<Self, ValidationError<'a>> {
        let unevaluated =
            UnevaluatedSubvalidator::from_value(schema, &ctx.with_path("unevaluatedProperties"))?;

        let additional = parent
            .get("additionalProperties")
            .map(|additional_properties| {
                UnevaluatedSubvalidator::from_value(
                    additional_properties,
                    &ctx.with_path("additionalProperties"),
                )
            })
            .transpose()?;

        let properties = parent
            .get("properties")
            .map(|properties| PropertySubvalidator::from_value(ctx, properties))
            .transpose()?;
        let patterns = parent
            .get("patternProperties")
            .map(|pattern_properties| PatternSubvalidator::from_value(ctx, pattern_properties))
            .transpose()?;

        let conditional = parent
            .get("if")
            .map(|condition| {
                let success = parent.get("then");
                let failure = parent.get("else");

                ConditionalSubvalidator::from_values(ctx, schema, condition, success, failure)
                    .map(Box::new)
            })
            .transpose()?;

        let dependent = parent
            .get("dependentSchemas")
            .map(|dependent_schemas| {
                DependentSchemaSubvalidator::from_value(ctx, schema, dependent_schemas)
            })
            .transpose()?;

        let reference = parent
            .get("$ref")
            .map(|reference| ReferenceSubvalidator::from_value(ctx, schema, reference))
            .transpose()?
            .flatten();

        let mut subschema_validators = vec![];
        if let Some(Value::Array(subschemas)) = parent.get("allOf") {
            let validator = SubschemaSubvalidator::from_values(
                schema,
                subschemas,
                SubschemaBehavior::All,
                ctx,
            )?;
            subschema_validators.push(validator);
        }

        if let Some(Value::Array(subschemas)) = parent.get("anyOf") {
            let validator = SubschemaSubvalidator::from_values(
                schema,
                subschemas,
                SubschemaBehavior::Any,
                ctx,
            )?;
            subschema_validators.push(validator);
        }

        if let Some(Value::Array(subschemas)) = parent.get("oneOf") {
            let validator = SubschemaSubvalidator::from_values(
                schema,
                subschemas,
                SubschemaBehavior::One,
                ctx,
            )?;
            subschema_validators.push(validator);
        }

        let subschemas = if subschema_validators.is_empty() {
            None
        } else {
            Some(subschema_validators)
        };

        Ok(Self {
            schema_path: JsonPointer::from(&ctx.path),
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
                self.subschemas.as_ref().and_then(|subschemas| {
                    subschemas.iter().find_map(|subschema| {
                        subschema.apply_property(
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

/// A subvalidator for properties.
#[derive(Debug)]
struct PropertySubvalidator {
    prop_map: SmallValidatorsMap,
}

impl PropertySubvalidator {
    fn from_value<'a>(
        ctx: &compiler::Context,
        properties: &'a Value,
    ) -> Result<Self, ValidationError<'a>> {
        properties
            .as_object()
            .ok_or_else(ValidationError::null_schema)
            .and_then(|props| SmallValidatorsMap::from_map(ctx, props))
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
        ctx: &compiler::Context,
        properties: &'a Value,
    ) -> Result<Self, ValidationError<'a>> {
        properties
            .as_object()
            .ok_or_else(ValidationError::null_schema)
            .and_then(|props| compile_patterns(ctx, props))
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

        had_match.then_some(true)
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

        had_match.then_some(output)
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
        ctx: &compiler::Context,
    ) -> Result<Self, ValidationError<'a>> {
        let mut subvalidators = vec![];
        let keyword_context = ctx.with_path(behavior.as_str());

        for (i, value) in values.iter().enumerate() {
            if let Value::Object(subschema) = value {
                let ctx = keyword_context.with_path(i);

                let node = compiler::compile(&ctx, ctx.as_resource_ref(value))?;
                let subvalidator = UnevaluatedPropertiesValidator::compile(
                    &ctx,
                    subschema,
                    get_transitive_unevaluated_props_schema(subschema, parent),
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
                if all_subschemas_valid {
                    // We only need to find the first valid evaluation because we know if that
                    // all subschemas were valid against the instance that there can't actually
                    // be any subschemas where the property was evaluated but invalid.
                    results
                        .iter()
                        .find_map(|(property_result, _)| *property_result)
                } else {
                    None
                }
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
                    instance_valid.then_some(property_result).flatten()
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
                        .then_some(property_errors)
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
                        .then_some(property_output)
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
        ctx: &compiler::Context,
    ) -> Result<Self, ValidationError<'a>> {
        let behavior = match value {
            Value::Bool(false) => UnevaluatedBehavior::Deny,
            Value::Bool(true) => UnevaluatedBehavior::Allow,
            _ => UnevaluatedBehavior::IfValid(compiler::compile(ctx, ctx.as_resource_ref(value))?),
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
        ctx: &compiler::Context,
        parent: &'a Value,
        schema: &'a Value,
        success: Option<&'a Value>,
        failure: Option<&'a Value>,
    ) -> Result<Self, ValidationError<'a>> {
        let if_context = ctx.with_path("if");
        compiler::compile(&if_context, if_context.as_resource_ref(schema)).and_then(|condition| {
            let node = schema
                .as_object()
                .map(|node_schema| {
                    UnevaluatedPropertiesValidator::compile(
                        &if_context,
                        node_schema,
                        get_transitive_unevaluated_props_schema(node_schema, parent),
                    )
                })
                .transpose()?;
            let success = success
                .and_then(|value| value.as_object())
                .map(|success_schema| {
                    UnevaluatedPropertiesValidator::compile(
                        &ctx.with_path("then"),
                        success_schema,
                        get_transitive_unevaluated_props_schema(success_schema, parent),
                    )
                })
                .transpose()?;
            let failure = failure
                .and_then(|value| value.as_object())
                .map(|failure_schema| {
                    UnevaluatedPropertiesValidator::compile(
                        &ctx.with_path("else"),
                        failure_schema,
                        get_transitive_unevaluated_props_schema(failure_schema, parent),
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
        ctx: &compiler::Context,
        parent: &'a Value,
        value: &'a Value,
    ) -> Result<Self, ValidationError<'a>> {
        let ctx = ctx.with_path("dependentSchemas");
        let schemas = value
            .as_object()
            .ok_or_else(|| unexpected_type(&ctx, value, PrimitiveType::Object))?;
        let mut nodes = AHashMap::new();

        for (dependent_property_name, dependent_schema) in schemas {
            let dependent_schema = dependent_schema
                .as_object()
                .ok_or_else(ValidationError::null_schema)?;

            let ctx = ctx.with_path(dependent_property_name.as_str());
            let node = UnevaluatedPropertiesValidator::compile(
                &ctx,
                dependent_schema,
                get_transitive_unevaluated_props_schema(dependent_schema, parent),
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
                    .then_some(node)
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
                    .then_some(node)
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
                    .then_some(node)
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
    fn from_value_impl<'a>(
        ctx: &compiler::Context,
        parent: &'a Value,
        contents: &Value,
    ) -> Result<Option<Self>, ValidationError<'a>> {
        contents
            .as_object()
            .map(|schema| {
                UnevaluatedPropertiesValidator::compile(
                    ctx,
                    schema,
                    get_transitive_unevaluated_props_schema(schema, parent),
                )
                .map(|validator| ReferenceSubvalidator {
                    node: Box::new(validator),
                })
                .map_err(|e| e.into_owned())
            })
            .transpose()
    }
    fn from_value<'a>(
        ctx: &compiler::Context,
        parent: &'a Value,
        value: &'a Value,
    ) -> Result<Option<Self>, ValidationError<'a>> {
        let kctx = ctx.with_path("$ref");
        let reference = value
            .as_str()
            .ok_or_else(|| unexpected_type(&kctx, value, PrimitiveType::String))?;

        let is_recursive = parent
            .get("$recursiveAnchor")
            .and_then(Value::as_bool)
            .unwrap_or_default();
        if let Some((_, _, resource)) = ctx.lookup_maybe_recursive(reference, is_recursive)? {
            Self::from_value_impl(ctx, parent, resource.contents())
        } else {
            let resolved = ctx.lookup(reference)?;
            Self::from_value_impl(ctx, parent, resolved.contents())
        }
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
    ctx: &compiler::Context,
    parent: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    // Nothing to validate if `unevaluatedProperties` is set to `true`, which is the default:
    if let Value::Bool(true) = schema {
        return None;
    }

    match UnevaluatedPropertiesValidator::compile(ctx, parent, schema) {
        Ok(validator) => Some(Ok(Box::new(validator))),
        Err(e) => Some(Err(e)),
    }
}

fn boxed_errors<'a>(errors: Vec<ValidationError<'a>>) -> ErrorIterator<'a> {
    let boxed_errors: ErrorIterator<'a> = Box::new(errors.into_iter());
    boxed_errors
}

fn unexpected_type<'a>(
    ctx: &compiler::Context,
    instance: &'a Value,
    expected_type: PrimitiveType,
) -> ValidationError<'a> {
    ValidationError::single_type_error(
        JsonPointer::default(),
        ctx.clone().into_pointer(),
        instance,
        expected_type,
    )
}

#[cfg(test)]
mod tests {
    use crate::{tests_util, Draft};
    use serde_json::json;

    #[test]
    fn one_of() {
        tests_util::is_valid_with_draft(
            Draft::Draft202012,
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
            Draft::Draft202012,
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
            Draft::Draft202012,
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
            Draft::Draft202012,
            &schema,
            &json!({ "foo": "wee", "another": "thing" }),
        );

        tests_util::is_not_valid_with_draft(
            Draft::Draft202012,
            &schema,
            &json!({ "foo": "wee", "another": false }),
        );
    }

    #[test]
    fn test_unevaluated_properties_with_allof_oneof() {
        let schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "allOf": [{}],
            "oneOf": [
                {
                    "properties": {
                        "blah": true
                    }
                }
            ],
            "unevaluatedProperties": false
        });

        let valid = json!({
            "blah": 1
        });

        let validator = crate::validator_for(&schema).expect("Schema should compile");

        assert!(validator.validate(&valid).is_ok(), "Validation should pass");
        assert!(validator.is_valid(&valid), "Instance should be valid");

        let invalid = json!({
            "blah": 1,
            "extra": "property"
        });

        assert!(
            !validator.is_valid(&invalid),
            "Instance with extra property should be invalid"
        );
        assert!(
            validator.validate(&invalid).is_err(),
            "Validation should fail for instance with extra property"
        );
    }

    #[test]
    fn test_unevaluated_properties_with_recursion() {
        // See GH-420
        let schema = json!({
          "allOf": [
            {
              "$ref": "#/$defs/1_1"
            }
          ],
          "unevaluatedProperties": false,
          "$defs": {
            "1_1": {
              "type": "object",
              "properties": {
                "b": {
                  "allOf": [
                    {
                      "$ref": "#/$defs/1_2"
                    }
                  ],
                  "unevaluatedProperties": false
                }
              },
              "required": [
                "b"
              ]
            },
            "1_2": {
              "type": "object",
              "properties": {
                "f": {
                  "allOf": [
                    {
                      "$ref": "#/$defs/1_1"
                    }
                  ],
                  "unevaluatedProperties": false
                }
              },
              "required": [
                "f"
              ]
            }
          }
        });

        let validator = crate::validator_for(&schema).expect("Schema should compile");

        let instance = json!({"b": {"f": null}});
        assert!(!validator.is_valid(&instance));
        assert!(validator.validate(&instance).is_err());
    }
}
