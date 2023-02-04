use crate::vocabularies::Keyword;
use num_cmp::NumCmp;
use serde_json::Value;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Maximum {
    pub(crate) limit: u64,
}

impl Maximum {
    pub(crate) fn build(limit: u64) -> Keyword {
        Self { limit }.into()
    }
}

impl Maximum {
    pub(crate) fn is_valid(&self, instance: &Value) -> bool {
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
