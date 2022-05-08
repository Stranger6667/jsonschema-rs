use crate::vocabularies::{CompositeKeyword, LeafKeyword};
use serde_json::Value;
use std::ops::Range;

/// An intermediate representation of a JSON Schema node.
#[derive(Debug)]
pub(crate) enum IntermediateNode<'schema> {
    Root {
        children: Range<usize>,
        value: &'schema Value,
    },
    Composite {
        keyword: CompositeKeyword,
        children: Range<usize>,
        value: &'schema Value,
    },
    Leaf {
        keyword: LeafKeyword,
        value: &'schema Value,
    },
    Reference(&'schema Value),
}

impl<'schema> IntermediateNode<'schema> {
    pub(crate) fn as_inner(&self) -> &'schema Value {
        match self {
            IntermediateNode::Leaf { value, .. } => value,
            IntermediateNode::Reference(value) => value,
            IntermediateNode::Composite { value, .. } => value,
            IntermediateNode::Root { value, .. } => value,
        }
    }
}
