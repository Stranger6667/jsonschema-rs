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
