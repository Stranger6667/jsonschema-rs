use serde_json::Value;

use crate::{segments::Segment, Error, Resolver, ResourceRef, Segments};

use super::subresources::SubresourceIterator;

pub(crate) fn subresources_of<'a>(contents: &'a Value) -> SubresourceIterator<'a> {
    match contents.as_object() {
        Some(schema) => Box::new(schema.iter().flat_map(|(key, value)| match key.as_str() {
            "additionalItems"
            | "additionalProperties"
            | "contains"
            | "contentSchema"
            | "else"
            | "if"
            | "not"
            | "propertyNames"
            | "then"
            | "unevaluatedItems"
            | "unevaluatedProperties" => {
                Box::new(std::iter::once(value)) as SubresourceIterator<'a>
            }
            "allOf" | "anyOf" | "oneOf" => Box::new(value.as_array().into_iter().flatten()),
            "$defs" | "definitions" | "dependentSchemas" | "patternProperties" | "properties" => {
                Box::new(value.as_object().into_iter().flat_map(|o| o.values()))
            }
            "items" => match value {
                Value::Array(arr) => Box::new(arr.iter()) as SubresourceIterator<'a>,
                _ => Box::new(std::iter::once(value)),
            },
            _ => Box::new(std::iter::empty()),
        })),
        None => Box::new(std::iter::empty()),
    }
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
