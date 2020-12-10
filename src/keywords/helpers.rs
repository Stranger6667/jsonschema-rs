use num_cmp::NumCmp;
use serde_json::{Map, Value};

macro_rules! num_cmp {
    ($left:expr, $right:expr) => {
        if let Some(b) = $right.as_u64() {
            NumCmp::num_eq($left, b)
        } else if let Some(b) = $right.as_i64() {
            NumCmp::num_eq($left, b)
        } else {
            NumCmp::num_eq($left, $right.as_f64().expect("Always valid"))
        }
    };
}

#[inline]
pub(crate) fn equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => {
            if let Some(a) = left.as_u64() {
                num_cmp!(a, right)
            } else if let Some(a) = left.as_i64() {
                num_cmp!(a, right)
            } else {
                let a = left.as_f64().expect("Always valid");
                num_cmp!(a, right)
            }
        }
        (Value::Array(left), Value::Array(right)) => equal_arrays(left, right),
        (Value::Object(left), Value::Object(right)) => equal_objects(left, right),
        (_, _) => left == right,
    }
}

#[inline]
pub(crate) fn equal_arrays(left: &[Value], right: &[Value]) -> bool {
    left.len() == right.len() && left.iter().zip(right.iter()).all(|(a, b)| equal(a, b))
}

#[inline]
pub(crate) fn equal_objects(left: &Map<String, Value>, right: &Map<String, Value>) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|((ka, va), (kb, vb))| ka == kb && equal(va, vb))
}
