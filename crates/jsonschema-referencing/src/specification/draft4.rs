use serde_json::Value;

use crate::{Error, Resolver, ResourceRef, Segments};

use super::subresources;

pub(crate) fn subresources_of<'a>(contents: &'a Value) -> Box<dyn Iterator<Item = &'a Value> + 'a> {
    const IN_VALUE: &[&str] = &["not"];
    const IN_SUBARRAY: &[&str] = &["allOf", "anyOf", "oneOf"];
    const IN_SUBVALUES: &[&str] = &["definitions", "patternProperties", "properties"];

    match contents.as_object() {
        Some(schema) => {
            let subresources = subresources::lookup!(schema, IN_VALUE, IN_SUBVALUES, IN_SUBARRAY);
            let items = subresources::lookup_in_items!(schema);
            let dependencies = subresources::lookup_in_dependencies!(schema);
            let additional = ["additionalItems", "additionalProperties"]
                .iter()
                .filter_map(|&key| schema.get(key))
                .filter(|value| value.is_object());

            Box::new(
                subresources
                    .chain(items)
                    .chain(dependencies)
                    .chain(additional),
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
