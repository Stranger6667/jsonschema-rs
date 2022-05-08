use crate::vocabularies::{Keyword, Validate};
use num_cmp::NumCmp;
use serde_json::Value;

#[derive(Debug)]
pub struct Maximum {
    pub(crate) limit: u64,
}

impl Validate for Maximum {
    fn is_valid(&self, _: &[Keyword], instance: &Value) -> bool {
        if let Value::Number(item) = instance {
            if let Some(item) = item.as_u64() {
                !NumCmp::num_gt(item, self.limit)
            } else if let Some(item) = item.as_i64() {
                !NumCmp::num_gt(item, self.limit)
            } else {
                let item = item.as_f64().expect("Always valid");
                !NumCmp::num_gt(item, self.limit)
            }
        } else {
            true
        }
    }
}

pub(crate) mod compile {
    use crate::{compilation::IntermediateNode, vocabularies::LeafKeyword};
    use serde_json::Value;

    pub(crate) fn intermediate(value: &Value) -> IntermediateNode {
        IntermediateNode::Leaf {
            keyword: LeafKeyword::Maximum,
            value,
        }
    }
}
