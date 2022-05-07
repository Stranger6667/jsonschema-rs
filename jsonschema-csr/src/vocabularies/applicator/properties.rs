use crate::compilation;
// use crate::compilation::context::CompilationContext;
use crate::compilation::Node;
use crate::resolver::LocalResolver;
use crate::vocabularies::{Keyword, Validate};
use serde_json::Value;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct Properties {
    properties: Box<[Box<str>]>,
    start: usize,
}

impl Properties {
    // #[inline]
    // pub(crate) fn compile<'schema>(
    //     schema: &'schema Value,
    //     global: &mut Vec<Node<'schema>>,
    //     resolver: &'schema LocalResolver<'schema>,
    // ) -> Node<'schema> {
    //     match schema {
    //         Value::Object(map) => {
    //             let mut properties = Vec::with_capacity(map.len());
    //             let mut local = Vec::with_capacity(map.len());
    //             for (key, subschema) in map {
    //                 properties.push(key.clone().into_boxed_str());
    //                 compilation::build_one(subschema, resolver, global, &mut local)
    //             }
    //             let start = global.len();
    //             // global.extend(local.into_iter());
    //             Node::Value(schema)
    //         }
    //         _ => todo!(),
    //     }
    // }
}

macro_rules! next {
    ($iter:expr) => {{
        if let Some(value) = $iter.next() {
            value
        } else {
            return true;
        }
    }};
}

impl Validate for Properties {
    fn is_valid(&self, keywords: &[Keyword], instance: &Value) -> bool {
        if let Value::Object(items) = instance {
            // TODO. Separate keyword for single property
            // TODO. It depends on serde feature - won't work for index map
            let mut items = items.iter();
            let (mut key, mut value) = next!(items);
            let mut properties = self
                .properties
                .iter()
                .zip(&keywords[self.start..self.properties.len()]);
            let (mut property, mut keyword) = next!(properties);
            loop {
                match key.as_str().cmp(&**property) {
                    Ordering::Less => (key, value) = next!(items),
                    Ordering::Equal => {
                        if !keyword.is_valid(keywords, value) {
                            return false;
                        }
                        (key, value) = next!(items);
                        (property, keyword) = next!(properties);
                    }
                    Ordering::Greater => (property, keyword) = next!(properties),
                }
            }
        } else {
            true
        }
    }
}
//
// #[inline]
// pub(crate) fn compile<'schema>(
//     schema: &'schema Value,
//     global: &mut Vec<Node<'schema>>,
//     resolver: &'schema LocalResolver<'schema>,
//     // context: &mut CompilationContext,
// ) -> Node<'schema> {
//     Properties::compile(schema, global, resolver)
// }
