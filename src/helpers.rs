use crate::schemas::{id_of, Draft};
use serde_json::Value;

pub(crate) fn equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => left.as_f64() == right.as_f64(),
        (_, _) => left == right,
    }
}

fn parse_index(s: &str) -> Option<usize> {
    if s.starts_with('+') || (s.starts_with('0') && s.len() != 1) {
        return None;
    }
    s.parse().ok()
}
pub fn pointer<'a>(
    draft: Draft,
    document: &'a Value,
    pointer: &str,
) -> Option<(Vec<&'a str>, &'a Value)> {
    if pointer == "" {
        return Some((vec![], document));
    }
    if !pointer.starts_with('/') {
        return None;
    }
    let tokens = pointer
        .split('/')
        .skip(1)
        .map(|x| x.replace("~1", "/").replace("~0", "~"));
    let mut target = document;
    let mut folders = vec![];

    for token in tokens {
        let target_opt = match *target {
            Value::Object(ref map) => {
                if let Some(id) = id_of(draft, target) {
                    folders.push(id);
                }
                map.get(&token)
            }
            Value::Array(ref list) => parse_index(&token).and_then(|x| list.get(x)),
            _ => return None,
        };
        if let Some(t) = target_opt {
            target = t;
        } else {
            return None;
        }
    }
    Some((folders, target))
}
