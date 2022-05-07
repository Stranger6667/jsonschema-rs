/// Compressed sparse row format for JSON Schema.
///
/// Fast and cache efficient validation requires fast iteration over the schema, therefore a
/// representation like `serde_json::Value` should be converted to a more compact one.
///
/// JSON Schema is a directed graph, where cycles could be represented via the `$ref` keyword.
use serde_json::Value;
pub mod resolver;

use resolver::Resolver;

pub fn build(schema: &Value) {
    // The input graph may be incomplete:
    //   1. Some nodes are absent because `$ref` can point to remote locations;
    //   2. Edges coming from the `$ref` keywords are unknown as they require resolving;
    //
    // Build a vector of nodes where each node is a reference to a node from `schema` or to a
    // loaded remote schema. The `$ref` keyword nodes are resolved during this process:
    //   - Local references are resolved in-place, so the resulting node is the actual target node
    //   - Remote references are loaded into a separate container, and processed as any other
    //   nodes.
    //
    // Remote schemas could also have references that should be resolved, therefore
    // this step is applied recursively to all resolved schemas.
    let resolver = Resolver::new(schema);
    let mut output = vec![schema];
    let mut stack = vec![schema];
    while let Some(node) = stack.pop() {
        match node {
            Value::Object(object) => {
                for (key, value) in object {
                    stack.push(value);
                    output.push(value);
                }
            }
            Value::Array(array) => {}
            _ => {}
        }
    }
    for r in &output {
        println!("REF: {:p}", *r as *const Value);
    }
    println!("{:?}", output);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn it_works() {
        let schema = json!({
            "properties": {
                "foo": {"$ref": "#"}
            }
        });
        build(&schema);
    }
}
