use crate::{
    compilation,
    vocabularies::{Keyword, Validate},
};
use serde_json::{Map, Value};
use std::ops::Range;

#[derive(Debug)]
pub struct ItemsArray {
    items: Range<usize>,
}

impl Validate for ItemsArray {
    fn is_valid(&self, keywords: &[Keyword], instance: &serde_json::Value) -> bool {
        if let Value::Array(items) = instance {
            items
                .iter()
                .zip(&keywords[self.items.clone()])
                .all(|(item, val)| val.is_valid(keywords, item))
        } else {
            true
        }
    }
}

pub(crate) fn compile(
    _: &Map<String, Value>,
    schema: &serde_json::Value,
    global: &mut Vec<Keyword>,
) -> Keyword {
    todo!()
    // match schema {
    //     Value::Array(schemas) => {
    //         let mut local = Vec::with_capacity(schemas.len());
    //         compilation::compile_many(schemas, global, &mut local, context);
    //         ItemsArray {
    //             items: compilation::append(global, local),
    //         }
    //         .into()
    //     }
    //     Value::Object(_) => {
    //         todo!()
    //     }
    //     _ => panic!(),
    // }
}
