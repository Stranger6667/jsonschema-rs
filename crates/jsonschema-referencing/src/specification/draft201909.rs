use serde_json::Value;

use crate::{segments::Segment, Error, Resolver, ResourceRef, Segments};

use super::subresources;

pub(crate) fn subresources_of<'a>(contents: &'a Value) -> Box<dyn Iterator<Item = &'a Value> + 'a> {
    const IN_VALUE: &[&str] = &[
        "additionalItems",
        "additionalProperties",
        "contains",
        "contentSchema",
        "else",
        "if",
        "not",
        "propertyNames",
        "then",
        "unevaluatedItems",
        "unevaluatedProperties",
    ];
    const IN_SUBARRAY: &[&str] = &["allOf", "anyOf", "oneOf"];
    const IN_SUBVALUES: &[&str] = &[
        "$defs",
        "definitions",
        "dependentSchemas",
        "patternProperties",
        "properties",
    ];

    let Some(schema) = contents.as_object() else {
        return Box::new(std::iter::empty());
    };

    let subresources = subresources::lookup!(schema, IN_VALUE, IN_SUBVALUES, IN_SUBARRAY);
    let items = subresources::lookup_in_items!(schema);
    Box::new(subresources.chain(items))
}

pub(crate) fn maybe_in_subresource<'r>(
    segments: &Segments,
    resolver: &Resolver<'r>,
    subresource: ResourceRef<'r>,
) -> Result<Resolver<'r>, Error> {
    const IN_VALUE: &[&str] = &[
        "additionalItems",
        "additionalProperties",
        "contains",
        "contentSchema",
        "else",
        "if",
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
        "$defs",
        "definitions",
        "dependentSchemas",
        "patternProperties",
        "properties",
    ];

    let mut iter = segments.iter();
    while let Some(segment) = iter.next() {
        if let Segment::Key(key) = segment {
            if *key == "items" && subresource.contents().is_object() {
                return resolver.in_subresource(subresource);
            }
            if !IN_VALUE.contains(&key.as_ref())
                && (!IN_CHILD.contains(&key.as_ref()) || iter.next().is_none())
            {
                return Ok(resolver.clone());
            }
        }
    }
    resolver.in_subresource(subresource)
}
