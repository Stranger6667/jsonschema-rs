use serde_json::Value;

fn are_f64_equal(left: f64, right: f64) -> bool {
    (left - right).abs() < 3. * f64::EPSILON
}

pub(crate) fn equal(left: &Value, right: &Value) -> bool {
    if left == right {
        true
    } else if let Some(left_num) = left.as_f64() {
        if let Some(right_num) = right.as_f64() {
            are_f64_equal(left_num, right_num)
        } else {
            false
        }
    } else {
        false
    }
}
