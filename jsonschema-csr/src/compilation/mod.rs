/// Compressed sparse row format for JSON Schema.
///
/// Fast and cache efficient validation requires fast iteration over the schema, therefore a
/// representation like `serde_json::Value` should be converted to a more compact one.
///
/// JSON Schema is a directed graph, where cycles could be represented via the `$ref` keyword.
use serde_json::Value;
mod nodes;
pub mod resolver;

use crate::vocabularies::{
    applicator::properties, ref_, validation::maximum, CompositeKeyword, Keyword, LeafKeyword,
};
pub(crate) use nodes::IntermediateNode;
pub(crate) use resolver::LocalResolver;

#[derive(Debug)]
pub struct JsonSchema {
    schema: Box<[Keyword]>,
}

impl JsonSchema {
    pub fn compile(schema: &serde_json::Value) -> Self {
        let mut nodes = Vec::with_capacity(32);
        let resolver = LocalResolver::new(schema);
        compile(schema, &resolver, &mut nodes);
        Self {
            schema: resolve_references(nodes).into_boxed_slice(),
        }
    }
}

pub(crate) fn compile<'schema>(
    value: &'schema Value,
    resolver: &'schema LocalResolver,
    global: &mut Vec<IntermediateNode<'schema>>,
) {
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
    let mut local = vec![];
    compile_one(value, resolver, global, &mut local);
    let start = global.len();
    global.extend(local.into_iter());
    global.push(IntermediateNode::Root {
        children: start..global.len(),
        value,
    });
}

pub(crate) fn compile_one<'schema>(
    schema: &'schema Value,
    resolver: &'schema LocalResolver,
    global: &mut Vec<IntermediateNode<'schema>>,
    local: &mut Vec<IntermediateNode<'schema>>,
) {
    match schema {
        Value::Object(object) => {
            if let Some(reference) = object.get("$ref") {
                if let Value::String(reference) = reference {
                    let resolved = resolver.resolve(reference).unwrap();
                    local.push(IntermediateNode::Reference(resolved))
                } else {
                    todo!()
                }
            } else {
                for (keyword, value) in object.iter() {
                    match keyword.as_str() {
                        "maximum" => local.push(maximum::compile::intermediate(value)),
                        "properties" => {
                            local.push(properties::compile::intermediate(value, global, resolver))
                        }
                        _ => {}
                    }
                }
            }
        }
        _ => todo!(),
    }
}

fn resolve_references(nodes: Vec<IntermediateNode>) -> Vec<Keyword> {
    let mut keywords = Vec::with_capacity(nodes.len());
    for node in &nodes {
        match node {
            IntermediateNode::Root { .. } => continue,
            IntermediateNode::Composite {
                keyword,
                children,
                value,
            } => match keyword {
                CompositeKeyword::ItemsArray => {}
                CompositeKeyword::Properties => {
                    keywords.push(properties::compile::to_final(value, children.clone()));
                    // stack.push(children.clone());
                }
            },
            IntermediateNode::Leaf { keyword, value } => match keyword {
                LeafKeyword::Maximum => keywords.push(
                    maximum::Maximum {
                        limit: value.as_u64().unwrap(),
                    }
                    .into(),
                ),
                LeafKeyword::Ref => {}
            },
            IntermediateNode::Reference(value) => {
                for (_, node) in nodes.iter().enumerate() {
                    match node {
                        IntermediateNode::Root { children, .. }
                        | IntermediateNode::Composite { children, .. } => {
                            if std::ptr::eq(node.as_inner() as *const _, *value as *const _) {
                                keywords.push(
                                    ref_::Ref {
                                        reference: "".to_string(),
                                        range: children.clone(),
                                    }
                                    .into(),
                                );
                                break;
                            }
                        }
                        IntermediateNode::Leaf { .. } => {}
                        IntermediateNode::Reference(_) => continue,
                    }
                }
            }
        }
    }
    keywords
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
        });
        let compiled = JsonSchema::compile(&schema);
    }
}
