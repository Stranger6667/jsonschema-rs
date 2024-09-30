use serde_json::Value;

use crate::{Error, Resolver, ResourceRef, Segments};

use super::subresources::{self, SubresourceIterator};

//pub(crate) fn subresources_of<'a>(contents: &'a Value) -> Box<dyn Iterator<Item = &'a Value> + 'a> {
//    const IN_VALUE: &[&str] = &["not"];
//    const IN_SUBARRAY: &[&str] = &["allOf", "anyOf", "oneOf"];
//    const IN_SUBVALUES: &[&str] = &["definitions", "patternProperties", "properties"];
//
//    match contents.as_object() {
//        Some(schema) => {
//            let subresources = subresources::lookup!(schema, IN_VALUE, IN_SUBVALUES, IN_SUBARRAY);
//            let items = subresources::lookup_in_items!(schema);
//            let dependencies = subresources::lookup_in_dependencies!(schema);
//            let additional = ["additionalItems", "additionalProperties"]
//                .iter()
//                .filter_map(|&key| schema.get(key))
//                .filter(|value| value.is_object());
//
//            Box::new(
//                subresources
//                    .chain(items)
//                    .chain(dependencies)
//                    .chain(additional),
//            )
//        }
//        None => Box::new(std::iter::empty()),
//    }
//}

pub(crate) fn subresources_of<'a>(contents: &'a Value) -> Box<dyn Iterator<Item = &'a Value> + 'a> {
    Box::new(contents.as_object().into_iter().flat_map(|schema| {
        schema
            .iter()
            .flat_map(move |(key, value)| match key.as_bytes().split_first() {
                Some((b'a', rest)) => match rest.split_first() {
                    Some((b'd', rest)) => match rest.split_first() {
                        Some((b'd', rest)) => {
                            if rest == b"itionalItems" || rest == b"itionalProperties" {
                                value.as_object().map_or(SubresourceIterator::Empty, |obj| {
                                    SubresourceIterator::Object(obj.values())
                                })
                            } else {
                                SubresourceIterator::Empty
                            }
                        }
                        _ => SubresourceIterator::Empty,
                    },
                    Some((b'l', rest)) => {
                        if rest == b"lOf" {
                            value.as_array().map_or(SubresourceIterator::Empty, |arr| {
                                SubresourceIterator::Array(arr.iter())
                            })
                        } else {
                            SubresourceIterator::Empty
                        }
                    }
                    Some((b'n', rest)) => {
                        if rest == b"yOf" {
                            value.as_array().map_or(SubresourceIterator::Empty, |arr| {
                                SubresourceIterator::Array(arr.iter())
                            })
                        } else {
                            SubresourceIterator::Empty
                        }
                    }
                    _ => SubresourceIterator::Empty,
                },
                Some((b'd', rest)) => match rest.split_first() {
                    Some((b'e', rest)) => match rest.split_first() {
                        Some((b'f', rest)) => {
                            if rest == b"initions" {
                                value.as_object().map_or(SubresourceIterator::Empty, |obj| {
                                    SubresourceIterator::Object(obj.values())
                                })
                            } else {
                                SubresourceIterator::Empty
                            }
                        }
                        Some((b'p', rest)) => {
                            if rest == b"endencies" {
                                value
                                    .as_object()
                                    .map_or(SubresourceIterator::Empty, |deps| {
                                        SubresourceIterator::FilteredObject(
                                            deps.values().filter(|v| v.is_object()),
                                        )
                                    })
                            } else {
                                SubresourceIterator::Empty
                            }
                        }
                        _ => SubresourceIterator::Empty,
                    },
                    _ => SubresourceIterator::Empty,
                },
                Some((b'i', rest)) => {
                    if rest == b"tems" {
                        match value {
                            Value::Array(arr) => SubresourceIterator::Array(arr.iter()),
                            _ => SubresourceIterator::once(value),
                        }
                    } else {
                        SubresourceIterator::Empty
                    }
                }
                Some((b'n', rest)) => {
                    if rest == b"ot" {
                        SubresourceIterator::once(value)
                    } else {
                        SubresourceIterator::Empty
                    }
                }
                Some((b'o', rest)) => {
                    if rest == b"neOf" {
                        value.as_array().map_or(SubresourceIterator::Empty, |arr| {
                            SubresourceIterator::Array(arr.iter())
                        })
                    } else {
                        SubresourceIterator::Empty
                    }
                }
                Some((b'p', rest)) => match rest.split_first() {
                    Some((b'a', rest)) => {
                        if rest == b"tternProperties" {
                            value.as_object().map_or(SubresourceIterator::Empty, |obj| {
                                SubresourceIterator::Object(obj.values())
                            })
                        } else {
                            SubresourceIterator::Empty
                        }
                    }
                    Some((b'r', rest)) => {
                        if rest == b"operties" {
                            value.as_object().map_or(SubresourceIterator::Empty, |obj| {
                                SubresourceIterator::Object(obj.values())
                            })
                        } else {
                            SubresourceIterator::Empty
                        }
                    }
                    _ => SubresourceIterator::Empty,
                },
                _ => SubresourceIterator::Empty,
            })
    }))
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
