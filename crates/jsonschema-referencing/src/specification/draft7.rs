use serde_json::Value;

use crate::{Error, Resolver, ResourceRef, Segments};

use super::subresources::{self, LegacySubresourceIteratorImpl, SubresourceIterator};

pub(crate) fn subresources_of(contents: &Value) -> SubresourceIterator<'_> {
    match contents.as_object() {
        Some(schema) => Box::new(schema.iter().flat_map(|(key, value)| {
            match key.as_str() {
                "additionalItems"
                | "additionalProperties"
                | "contains"
                | "else"
                | "if"
                | "not"
                | "propertyNames"
                | "then" => LegacySubresourceIteratorImpl::once(value),
                "allOf" | "anyOf" | "oneOf" => value
                    .as_array()
                    .map_or(LegacySubresourceIteratorImpl::Empty, |arr| {
                        LegacySubresourceIteratorImpl::Array(arr.iter())
                    }),
                "definitions" | "patternProperties" | "properties" => value
                    .as_object()
                    .map_or(LegacySubresourceIteratorImpl::Empty, |obj| {
                        LegacySubresourceIteratorImpl::Object(obj.values())
                    }),
                "items" => match value {
                    Value::Array(arr) => LegacySubresourceIteratorImpl::Array(arr.iter()),
                    _ => LegacySubresourceIteratorImpl::once(value),
                },
                "dependencies" => {
                    value
                        .as_object()
                        .map_or(LegacySubresourceIteratorImpl::Empty, |deps| {
                            LegacySubresourceIteratorImpl::FilteredObject(
                                deps.values().filter(|v| v.is_object()),
                            )
                        })
                }
                _ => LegacySubresourceIteratorImpl::Empty,
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
        "else",
        "if",
        "not",
        "propertyNames",
        "then",
    ];
    const IN_CHILD: &[&str] = &[
        "allOf",
        "anyOf",
        "oneOf",
        "definitions",
        "patternProperties",
        "properties",
    ];
    subresources::maybe_in_subresource_with_items_and_dependencies(
        segments,
        resolver,
        subresource,
        IN_VALUE,
        IN_CHILD,
    )
}
