use serde_json::Value;

use crate::{Error, Resolver, ResourceRef, Segments};

use super::subresources::{self, SubresourceIterator};

pub(crate) fn subresources_of(contents: &Value) -> SubresourceIterator<'_> {
    match contents.as_object() {
        Some(schema) => Box::new(schema.iter().flat_map(|(key, value)| {
            // Rust compiler does not always optimize `match` into a jump table
            // This length & first byte matches force it to do so.
            match key.len() {
                3 if key == "not" => Box::new(std::iter::once(value)) as SubresourceIterator<'_>,
                5 => match key.as_bytes()[0] {
                    b'a' if key == "allOf" => {
                        Box::new(value.as_array().into_iter().flatten()) as SubresourceIterator<'_>
                    }
                    b'a' if key == "anyOf" => Box::new(value.as_array().into_iter().flatten()),
                    b'o' if key == "oneOf" => Box::new(value.as_array().into_iter().flatten()),
                    b'i' if key == "items" => match value {
                        Value::Array(arr) => Box::new(arr.iter()) as SubresourceIterator<'_>,
                        _ => Box::new(std::iter::once(value)),
                    },
                    _ => Box::new(std::iter::empty()),
                },
                10 if key == "properties" => {
                    Box::new(value.as_object().into_iter().flat_map(|o| o.values()))
                }
                11 if key == "definitions" => {
                    Box::new(value.as_object().into_iter().flat_map(|o| o.values()))
                }
                12 if key == "dependencies" => Box::new(
                    value
                        .as_object()
                        .into_iter()
                        .flat_map(|o| o.values())
                        .filter(|v| v.is_object()),
                ),
                15 if key == "additionalItems" && value.is_object() => {
                    Box::new(std::iter::once(value))
                }
                17 if key == "patternProperties" => {
                    Box::new(value.as_object().into_iter().flat_map(|o| o.values()))
                }
                20 if key == "additionalProperties" && value.is_object() => {
                    Box::new(std::iter::once(value))
                }
                _ => Box::new(std::iter::empty()),
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
    const IN_VALUE: &[&str] = &["additionalItems", "additionalProperties", "not"];
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
