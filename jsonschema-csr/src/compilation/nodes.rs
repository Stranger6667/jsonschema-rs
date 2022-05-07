use crate::vocabularies::KeywordKind;
use serde_json::Value;
use std::ops::Range;

/// An intermediate representation of a JSON Schema node.
#[derive(Debug)]
pub(crate) enum IntermediateNode<'schema> {
    Root {
        children: Range<usize>,
        value: &'schema Value,
    },
    Parent {
        keyword: KeywordKind,
        children: Range<usize>,
        value: &'schema Value,
    },
    Leaf {
        keyword: KeywordKind,
        value: &'schema Value,
    },
    Reference(&'schema Value),
}

impl<'schema> IntermediateNode<'schema> {
    pub(crate) fn as_inner(&self) -> &'schema Value {
        match self {
            IntermediateNode::Leaf { value, .. } => value,
            IntermediateNode::Reference(value) => value,
            IntermediateNode::Parent { value, .. } => value,
            IntermediateNode::Root { value, .. } => value,
        }
    }
}
