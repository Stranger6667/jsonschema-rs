use crate::vocabularies::{Keyword, Validate};
use serde_json::Value;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct Properties {
    properties: Box<[Box<str>]>,
    start: usize,
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

pub(crate) mod compile {
    use crate::{
        compilation::{compile_one, IntermediateNode, LocalResolver},
        vocabularies::KeywordKind,
    };
    use serde_json::Value;

    pub(crate) fn intermediate<'schema>(
        value: &'schema Value,
        global: &mut Vec<IntermediateNode<'schema>>,
        resolver: &'schema LocalResolver,
    ) -> IntermediateNode<'schema> {
        match value {
            Value::Object(map) => {
                let start = global.len();
                let mut local = Vec::with_capacity(map.len());
                for (key, subschema) in map {
                    compile_one(subschema, resolver, global, &mut local)
                }
                global.extend(local.into_iter());
                IntermediateNode::Parent {
                    keyword: KeywordKind::Properties,
                    children: start..global.len(),
                    value,
                }
            }
            _ => todo!(),
        }
    }
}
