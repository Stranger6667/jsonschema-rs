use serde_json::Value;

/// Unique identifier of a node in a graph.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub(crate) struct NodeId(usize);

impl NodeId {
    pub(crate) fn new(value: usize) -> Self {
        Self(value)
    }

    pub(crate) fn value(&self) -> usize {
        self.0
    }
    /// If this `NodeId` points to the root node.
    pub(crate) fn is_root(&self) -> bool {
        self.value() == 0
    }
}

/// A slot for a node in a tree.
pub(crate) struct NodeSlot {
    /// Unique node identifier.
    pub(crate) id: NodeId,
    /// Whether this slot was already used or not.
    state: SlotState,
}

#[derive(Debug, Eq, PartialEq)]
enum SlotState {
    /// Slot was not previously used.
    New,
    /// Slot is already used.
    Used,
}

impl NodeSlot {
    pub(crate) fn seen(id: NodeId) -> Self {
        Self {
            id,
            state: SlotState::Used,
        }
    }
    pub(crate) fn new(id: NodeId) -> Self {
        Self {
            id,
            state: SlotState::New,
        }
    }
    pub(crate) fn is_new(&self) -> bool {
        self.state == SlotState::New
    }
}

/// A JSON Schema specific kind of nodes.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub(crate) enum JsonSchemaNodeKind {
    /// No specific meaning. Used to simplify building an intermediate graph.
    Dummy,
    /// JSON Schema that contains keywords.
    Schema,
    /// Not a schema and does not contain keywords.
    NotSchema,
}

/// An intermediate node in a tree.
/// It marks nodes with JSON Schema-specific meaning.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Node<'s> {
    pub(crate) value: &'s Value,
    kind: JsonSchemaNodeKind,
}

impl<'s> Node<'s> {
    pub(crate) fn dummy() -> Self {
        Self {
            value: &Value::Null,
            kind: JsonSchemaNodeKind::Dummy,
        }
    }

    pub(crate) fn schema(value: &'s Value) -> Self {
        Self {
            value,
            kind: JsonSchemaNodeKind::Schema,
        }
    }

    pub(crate) fn toggle(&self, value: &'s Value) -> Self {
        Self {
            value,
            kind: {
                if self.kind == JsonSchemaNodeKind::Schema {
                    JsonSchemaNodeKind::NotSchema
                } else {
                    JsonSchemaNodeKind::Schema
                }
            },
        }
    }

    pub(crate) fn is_schema(&self) -> bool {
        self.kind == JsonSchemaNodeKind::Schema
    }
}
