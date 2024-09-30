use serde_json::Value;

use crate::{Error, Resolver, ResourceRef, Segments};

use super::subresources;

pub(crate) fn subresources_of<'a>(contents: &'a Value) -> Box<dyn Iterator<Item = &'a Value> + 'a> {
    const IN_VALUE: &[&str] = &[
        "additionalItems",
        "additionalProperties",
        "contains",
        "not",
        "propertyNames",
    ];
    const IN_SUBARRAY: &[&str] = &["allOf", "anyOf", "oneOf"];
    const IN_SUBVALUES: &[&str] = &["definitions", "patternProperties", "properties"];
    subresources::subresources_of_with_dependencies(contents)
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
        "not",
        "propertyNames",
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
