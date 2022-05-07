/// Compressed sparse row format for JSON Schema.
///
/// Fast and cache efficient validation requires fast iteration over the schema, therefore a
/// representation like `serde_json::Value` should be converted to a more compact one.
///
/// JSON Schema is a directed graph, where cycles could be represented via the `$ref` keyword.
use serde_json::Value;
pub mod resolver;

use crate::vocabularies::Keyword;
use resolver::LocalResolver;

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
    //
    // Non-leaf nodes store their edges as a range that points to the same vector
    let mut nodes = vec![];
    let resolver = LocalResolver::new(schema);
    build_one(schema, &resolver, &mut nodes);
}

pub(crate) fn build_one<'schema>(
    schema: &'schema Value,
    resolver: &'schema LocalResolver,
    nodes: &mut Vec<Node<'schema>>,
) {
    let mut stack = vec![schema];
    while let Some(node) = stack.pop() {
        match node {
            Value::Object(object) => {
                for (key, value) in object {
                    if key == "$ref" {
                        // TODO.
                        //   - Local reference - use local resolver,
                        //   - remote - then, resolve and run the same procedure
                        let resolved = resolver.resolve(value.as_str().unwrap()).unwrap();
                        // Do not push onto the stack, because the reference is local, therefore
                        // it will be processed in any case
                        nodes.push(Node::Reference(resolved));
                    } else {
                        stack.push(value);
                        // local.push(Node::Keyword(value));
                    }
                }
            }
            Value::Array(array) => {}
            _ => {}
        }
    }
}

#[derive(Debug)]
pub(crate) enum Node<'schema> {
    Keyword(&'schema Value),
    Reference(&'schema Value),
}

impl<'schema> Node<'schema> {
    pub(crate) fn as_inner(&self) -> &'schema Value {
        match self {
            Node::Keyword(value) => value,
            Node::Reference(value) => value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn it_works() {
        let schema = json!({
            "properties": {
                "foo": {"$ref": "#"},
                "bar": {"maximum": 5}
            },
            "type": "object"
        });
        build(&schema);
    }
}
