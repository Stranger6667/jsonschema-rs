use serde_json::Value;

use crate::{segments::Segment, Error, Resolver, ResourceRef, Segments};

macro_rules! lookup {
    ($schema:expr, $in_value:expr, $in_subvalues:expr, $in_subarray:expr) => {
        $in_value
            .iter()
            .flat_map(|&keyword| $schema.get(keyword).into_iter())
            .chain($in_subvalues.iter().flat_map(|&keyword| {
                $schema
                    .get(keyword)
                    .and_then(Value::as_object)
                    .into_iter()
                    .flat_map(|o| o.values())
            }))
            .chain($in_subarray.iter().flat_map(|&keyword| {
                $schema
                    .get(keyword)
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
            }))
    };
}

macro_rules! lookup_in_items {
    ($schema:expr) => {
        $schema
            .get("items")
            .into_iter()
            .flat_map(|items| match items {
                Value::Array(arr) => arr.iter().collect::<Vec<_>>(),
                _ => vec![items],
            })
    };
}

macro_rules! lookup_in_dependencies {
    ($schema:expr) => {
        $schema
            .get("dependencies")
            .into_iter()
            .flat_map(|dependencies| {
                if let Value::Object(deps) = dependencies {
                    let mut values = deps.values();
                    if let Some(first) = values.next() {
                        if first.is_object() {
                            std::iter::once(first).chain(values).collect::<Vec<_>>()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            })
    };
}
pub(crate) use lookup;
pub(crate) use lookup_in_dependencies;
pub(crate) use lookup_in_items;

pub(crate) fn subresources_of<'a>(contents: &'a Value) -> Box<dyn Iterator<Item = &'a Value> + 'a> {
    const IN_VALUE: &[&str] = &[
        "additionalProperties",
        "contains",
        "contentSchema",
        "else",
        "if",
        "items",
        "not",
        "propertyNames",
        "then",
        "unevaluatedItems",
        "unevaluatedProperties",
    ];
    const IN_SUBARRAY: &[&str] = &["allOf", "anyOf", "oneOf", "prefixItems"];
    const IN_SUBVALUES: &[&str] = &[
        "$defs",
        "definitions",
        "dependentSchemas",
        "patternProperties",
        "properties",
    ];

    match contents.as_object() {
        Some(schema) => Box::new(lookup!(schema, IN_VALUE, IN_SUBVALUES, IN_SUBARRAY)),
        None => Box::new(std::iter::empty()),
    }
}

#[inline]
pub(crate) fn subresources_of_with_dependencies<'a>(
    contents: &'a Value,
    in_value: &'static [&str],
    in_subvalues: &'static [&str],
    in_subarray: &'static [&str],
) -> Box<dyn Iterator<Item = &'a Value> + 'a> {
    match contents.as_object() {
        Some(schema) => {
            let normal_subresources = lookup!(schema, in_value, in_subvalues, in_subarray);
            let items_subresources = lookup_in_items!(schema);
            let dependencies_subresources = lookup_in_dependencies!(schema);
            Box::new(
                normal_subresources
                    .chain(items_subresources)
                    .chain(dependencies_subresources),
            )
        }
        None => Box::new(std::iter::empty()),
    }
}

pub(crate) fn maybe_in_subresource<'r>(
    segments: &Segments,
    resolver: &Resolver<'r>,
    subresource: ResourceRef<'r>,
) -> Result<Resolver<'r>, Error> {
    const IN_VALUE: &[&str] = &[
        "additionalProperties",
        "contains",
        "contentSchema",
        "else",
        "if",
        "items",
        "not",
        "propertyNames",
        "then",
        "unevaluatedItems",
        "unevaluatedProperties",
    ];
    const IN_CHILD: &[&str] = &[
        "allOf",
        "anyOf",
        "oneOf",
        "prefixItems",
        "$defs",
        "definitions",
        "dependentSchemas",
        "patternProperties",
        "properties",
    ];

    let mut iter = segments.iter();
    while let Some(segment) = iter.next() {
        if let Segment::Key(key) = segment {
            if !IN_VALUE.contains(&key.as_ref())
                && (!IN_CHILD.contains(&key.as_ref()) || iter.next().is_none())
            {
                return Ok(resolver.clone());
            }
        }
    }
    resolver.in_subresource(subresource)
}

#[inline]
pub(crate) fn maybe_in_subresource_with_items_and_dependencies<'r>(
    segments: &Segments,
    resolver: &Resolver<'r>,
    subresource: ResourceRef<'r>,
    in_value: &[&str],
    in_child: &[&str],
) -> Result<Resolver<'r>, Error> {
    let mut iter = segments.iter();
    while let Some(segment) = iter.next() {
        if let Segment::Key(key) = segment {
            if (*key == "items" || *key == "dependencies") && subresource.contents().is_object() {
                return resolver.in_subresource(subresource);
            }
            if !in_value.contains(&key.as_ref())
                && (!in_child.contains(&key.as_ref()) || iter.next().is_none())
            {
                return Ok(resolver.clone());
            }
        }
    }
    resolver.in_subresource(subresource)
}

#[cfg(test)]
mod tests {
    use crate::Draft;

    use super::subresources_of;
    use ahash::HashSet;
    use serde_json::json;
    use test_case::test_case;

    #[test_case(&json!(true), &[] ; "boolean schema")]
    #[test_case(&json!(false), &[] ; "boolean schema false")]
    #[test_case(&json!({}), &[] ; "empty object")]
    #[test_case(&json!({"type": "string"}), &[] ; "no subresources")]
    #[test_case(
        &json!({"additionalProperties": {"type": "string"}}),
        &[json!({"type": "string"})] ;
        "in_value single"
    )]
    #[test_case(
        &json!({"if": {"type": "string"}, "then": {"minimum": 0}}),
        &[json!({"type": "string"}), json!({"minimum": 0})] ;
        "in_value multiple"
    )]
    #[test_case(
        &json!({"properties": {"foo": {"type": "string"}, "bar": {"type": "number"}}}),
        &[json!({"type": "string"}), json!({"type": "number"})] ;
        "in_subvalues"
    )]
    #[test_case(
        &json!({"allOf": [{"type": "string"}, {"minLength": 1}]}),
        &[json!({"type": "string"}), json!({"minLength": 1})] ;
        "in_subarray"
    )]
    #[test_case(
        &json!({
            "type": "object",
            "properties": {
                "foo": {"type": "string"},
                "bar": {"type": "number"}
            },
            "additionalProperties": {"type": "boolean"},
            "allOf": [
                {"required": ["foo"]},
                {"required": ["bar"]}
            ]
        }),
        &[
            json!({"type": "string"}),
            json!({"type": "number"}),
            json!({"type": "boolean"}),
            json!({"required": ["foo"]}),
            json!({"required": ["bar"]})
        ] ;
        "complex schema"
    )]
    #[test_case(
        &json!({
            "$defs": {
                "positiveInteger": {
                    "type": "integer",
                    "exclusiveMinimum": 0
                }
            },
            "properties": {
                "count": { "$ref": "#/$defs/positiveInteger" }
            }
        }),
        &[
            json!({"type": "integer", "exclusiveMinimum": 0}),
            json!({"$ref": "#/$defs/positiveInteger"})
        ] ;
        "with $defs"
    )]
    fn test_subresources_of(schema: &serde_json::Value, expected: &[serde_json::Value]) {
        let subresources: HashSet<&serde_json::Value> = subresources_of(schema).collect();
        let expected_set: HashSet<&serde_json::Value> = expected.iter().collect();

        assert_eq!(
            subresources.len(),
            expected.len(),
            "Number of subresources doesn't match"
        );
        assert_eq!(
            subresources, expected_set,
            "Subresources don't match expected values"
        );
    }

    #[test]
    fn test_all_keywords() {
        let schema = json!({
            "additionalProperties": {"type": "string"},
            "contains": {"minimum": 0},
            "contentSchema": {"format": "email"},
            "else": {"maximum": 100},
            "if": {"type": "number"},
            "items": {"type": "array"},
            "not": {"type": "null"},
            "propertyNames": {"minLength": 1},
            "then": {"multipleOf": 2},
            "unevaluatedItems": {"type": "boolean"},
            "unevaluatedProperties": {"type": "integer"},
            "allOf": [{"type": "object"}, {"required": ["foo"]}],
            "anyOf": [{"minimum": 0}, {"maximum": 100}],
            "oneOf": [{"type": "string"}, {"type": "number"}],
            "prefixItems": [{"type": "string"}, {"type": "number"}],
            "$defs": {
                "positiveInteger": {"type": "integer", "minimum": 1}
            },
            "definitions": {
                "negativeInteger": {"type": "integer", "maximum": -1}
            },
            "dependentSchemas": {
                "foo": {"required": ["bar"]}
            },
            "patternProperties": {
                "^S_": {"type": "string"},
                "^I_": {"type": "integer"}
            },
            "properties": {
                "prop1": {"type": "string"},
                "prop2": {"type": "number"}
            }
        });

        let subresources: Vec<&serde_json::Value> = subresources_of(&schema).collect();
        assert_eq!(subresources.len(), 26);

        assert!(subresources.contains(&&json!({"type": "string"})));
        assert!(subresources.contains(&&json!({"minimum": 0})));
        assert!(subresources.contains(&&json!({"format": "email"})));
        assert!(subresources.contains(&&json!({"maximum": 100})));
        assert!(subresources.contains(&&json!({"type": "number"})));
        assert!(subresources.contains(&&json!({"type": "array"})));
        assert!(subresources.contains(&&json!({"type": "null"})));
        assert!(subresources.contains(&&json!({"minLength": 1})));
        assert!(subresources.contains(&&json!({"multipleOf": 2})));
        assert!(subresources.contains(&&json!({"type": "boolean"})));
        assert!(subresources.contains(&&json!({"type": "integer"})));
        assert!(subresources.contains(&&json!({"type": "object"})));
        assert!(subresources.contains(&&json!({"required": ["foo"]})));
        assert!(subresources.contains(&&json!({"minimum": 0})));
        assert!(subresources.contains(&&json!({"maximum": 100})));
        assert!(subresources.contains(&&json!({"type": "string"})));
        assert!(subresources.contains(&&json!({"type": "number"})));
        assert!(subresources.contains(&&json!({"type": "integer", "minimum": 1})));
        assert!(subresources.contains(&&json!({"type": "integer", "maximum": -1})));
        assert!(subresources.contains(&&json!({"required": ["bar"]})));
        assert!(subresources.contains(&&json!({"type": "string"})));
        assert!(subresources.contains(&&json!({"type": "integer"})));
    }

    #[test_case(Draft::Draft4)]
    #[test_case(Draft::Draft6)]
    #[test_case(Draft::Draft7)]
    #[test_case(Draft::Draft201909)]
    #[test_case(Draft::Draft202012)]
    fn test_subresources_of_bool_schema(draft: Draft) {
        let bool_schema = json!(true);

        assert!(
            draft
                .subresources_of(&bool_schema)
                .collect::<Vec<_>>()
                .is_empty(),
            "Draft {draft:?} should return empty subresources for boolean schema",
        );
    }
}
