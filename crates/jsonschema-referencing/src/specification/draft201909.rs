use serde_json::Value;

use crate::{segments::Segment, Error, Resolver, ResourceRef, Segments};

use super::subresources::{SubresourceIterator, SubresourceIteratorImpl};

pub(crate) fn subresources_of(contents: &Value) -> SubresourceIterator<'_> {
    match contents.as_object() {
        Some(schema) => Box::new(schema.iter().flat_map(|(key, value)| {
            match key.as_str() {
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
                | "unevaluatedProperties" => SubresourceIteratorImpl::once(value),
                "allOf" | "anyOf" | "oneOf" => value
                    .as_array()
                    .map_or(SubresourceIteratorImpl::Empty, |arr| {
                        SubresourceIteratorImpl::Array(arr.iter())
                    }),
                "$defs" | "definitions" | "dependentSchemas" | "patternProperties"
                | "properties" => value
                    .as_object()
                    .map_or(SubresourceIteratorImpl::Empty, |obj| {
                        SubresourceIteratorImpl::Object(obj.values())
                    }),
                "items" => match value {
                    Value::Array(arr) => SubresourceIteratorImpl::Array(arr.iter()),
                    _ => SubresourceIteratorImpl::once(value),
                },
                _ => SubresourceIteratorImpl::Empty,
            }
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
