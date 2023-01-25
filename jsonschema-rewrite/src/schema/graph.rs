use crate::{
    schema::{
        error::Result,
        resolving::{id_of_object, is_local, with_folders, Reference, Resolver},
    },
    value_type::ValueType,
    vocabularies::{
        applicator::{AllOf, Properties},
        validation::{MaxLength, Maximum, MinProperties, Type},
        Keyword,
    },
};
use serde_json::{Map, Value};
use std::collections::{hash_map::Entry, BTreeMap, HashMap};

use std::ops::Range;

/// A label on an edge between two JSON values.
/// It could be either a key name or an index.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum EdgeLabel {
    /// # Example
    ///
    /// `{"name": "Test"}` could be represented as:
    ///
    ///           name
    /// object ---------> "Test"
    ///
    /// The label for the edge between the top-level object and string "Test" is `name`.
    Key(Box<str>),
    /// # Example
    ///
    /// `["Test"]` could be represented as:
    ///
    ///          0
    /// array ------> "Test"
    ///
    /// The label for the edge between the top-level array and string "Test" is `0`.
    Index(usize),
}

impl EdgeLabel {
    pub(crate) fn as_key(&self) -> Option<&str> {
        if let EdgeLabel::Key(key) = self {
            Some(&**key)
        } else {
            None
        }
    }
}

impl From<usize> for EdgeLabel {
    fn from(value: usize) -> Self {
        EdgeLabel::Index(value)
    }
}

impl From<&String> for EdgeLabel {
    fn from(value: &String) -> Self {
        EdgeLabel::Key(value.to_owned().into_boxed_str())
    }
}

/// Id of a node in a graph.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub(crate) struct NodeId(usize);

/// An edge between two JSON values stored in a graph.
///
/// # Example
///
/// JSON:
///
/// ```json
/// {
///     "properties": {
///         "A": {
///             "type": "object"
///         },
///         "B": {
///             "type": "string"
///         }
///     }
/// }
/// ```
///
/// ("A", 0, 1) - an edge between `<properties>` and `<type: object>`
/// ("B", 0, 2) - an edge between `<properties>` and `<type: string>`
///
///          --------------- B ------------->
///         |  ------ A ---->               |
///         | |             |               |
/// `[ <properties>, <type: object>, <type: string>]`
///         0              1               2
///
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct SingleEdge {
    pub(crate) label: EdgeLabel,
    pub(crate) source: NodeId,
    pub(crate) target: NodeId,
}

/// A convenience shortcut to create `SingleEdge`.
pub(crate) fn single(label: impl Into<EdgeLabel>, source: NodeId, target: NodeId) -> SingleEdge {
    SingleEdge {
        label: label.into(),
        source,
        target,
    }
}

/// An edge between a single JSON value and a range of JSON values that are stored contiguously.
///
/// # Example
///
/// JSON:
///
/// ```json
/// {
///     "properties": {
///         "A": {
///             "type": "object",
///             "maxLength": 5
///         },
///         "B": {
///             "type": "string"
///         }
///     }
/// }
/// ```
///
/// ("A", 1..3) - an edge between `<properties>` and `<type: object>` + `<maxLength: 5>`
/// ("B", 3..4) - an edge between `<properties>` and `<type: string>`
///
///          ---------------------- B ---------------------->
///         |  ------------->------ A ------>               |
///         | |             |               |               |
/// `[ <properties>, <type: object>, <maxLength: 5>, <type: string>]`
///         0              1               2               3
///
#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct MultiEdge {
    pub(crate) label: EdgeLabel,
    pub(crate) keywords: Range<usize>,
}

impl MultiEdge {
    pub(crate) fn new(label: impl Into<EdgeLabel>, keywords: Range<usize>) -> MultiEdge {
        MultiEdge {
            label: label.into(),
            keywords,
        }
    }
}

/// A slot for a node in a tree.
pub(crate) struct NodeSlot {
    /// Unique node identifier.
    id: NodeId,
    state: SlotState,
}

#[derive(Debug, Eq, PartialEq)]
enum SlotState {
    /// Slot was not previously used.
    New,
    /// Slot is already occupied.
    Used,
}

impl NodeSlot {
    fn seen(id: NodeId) -> Self {
        Self {
            id,
            state: SlotState::Used,
        }
    }
    fn new(id: NodeId) -> Self {
        Self {
            id,
            state: SlotState::New,
        }
    }
    fn is_new(&self) -> bool {
        self.state == SlotState::New
    }
}

#[derive(Debug)]
pub(crate) struct KeywordNode<'s> {
    id: NodeId,
    value: &'s Value,
    name: &'static str,
}

impl<'s> KeywordNode<'s> {
    fn new(id: NodeId, value: &'s Value, name: &'static str) -> Self {
        Self { id, value, name }
    }
}

pub(crate) type KeywordMap<'s> = BTreeMap<NodeId, Vec<KeywordNode<'s>>>;
pub(crate) type EdgeMap = BTreeMap<NodeId, Vec<SingleEdge>>;
pub(crate) type SeenNodeMap = HashMap<*const Value, NodeId>;

/// The main goal of this phase is to collect all nodes from the input schema and its remote
/// dependencies into a single graph where each node is a reference to a JSON value from these
/// schemas.
pub(crate) fn build<'s>(
    schema: &'s Value,
    root_resolver: &'s Resolver,
    resolvers: &'s HashMap<&str, Resolver>,
) -> Result<(usize, Vec<Keyword>, Vec<MultiEdge>)> {
    let (keyword_map, edge_map) = collect(schema, root_resolver, resolvers)?;
    let mut keywords = vec![];
    let mut edges = vec![];
    dbg!(&keyword_map);
    dbg!(&edge_map);

    macro_rules! push_edge {
        ($node_id:expr, $next:expr) => {{
            // TODO: It will not work for $ref - it will point to some other keywords
            // Push all node's edges. For example, all key names from `properties`
            for edge in &edge_map[$node_id] {
                let end = $next + keyword_map[&edge.target].len();
                edges.push(MultiEdge::new(edge.label.clone(), $next..end));
                $next = end;
            }
        }};
    }

    macro_rules! next_edges {
        ($node_id:expr) => {
            edges.len()..edges.len() + edge_map[$node_id].len()
        };
    }

    let root_offset = keyword_map.get(&NodeId(0)).map_or(0, Vec::len);
    for node_keywords in keyword_map.values() {
        let mut next = keywords.len() + node_keywords.len();
        for KeywordNode { id, value, name } in node_keywords {
            match *name {
                "allOf" => {
                    keywords.push(AllOf::build(next_edges!(id)));
                    push_edge!(id, next);
                }
                "items" => {}
                "maximum" => keywords.push(Maximum::build(value.as_u64().unwrap())),
                "maxLength" => keywords.push(MaxLength::build(value.as_u64().unwrap())),
                "minProperties" => keywords.push(MinProperties::build(value.as_u64().unwrap())),
                "properties" => {
                    keywords.push(Properties::build(next_edges!(id)));
                    push_edge!(id, next);
                }
                "$ref" => {}
                "type" => {
                    let x = match value.as_str().unwrap() {
                        "array" => ValueType::Array,
                        "boolean" => ValueType::Boolean,
                        "integer" => ValueType::Integer,
                        "null" => ValueType::Null,
                        "number" => ValueType::Number,
                        "object" => ValueType::Object,
                        "string" => ValueType::String,
                        _ => panic!("invalid type"),
                    };
                    keywords.push(Type::build(x))
                }
                _ => {}
            }
        }
    }
    Ok((root_offset, keywords, edges))
}

pub(crate) fn collect<'s>(
    schema: &'s Value,
    root_resolver: &'s Resolver,
    resolvers: &'s HashMap<&str, Resolver>,
) -> Result<(KeywordMap<'s>, EdgeMap)> {
    Collector::new(resolvers).collect(schema, root_resolver)
}

struct CollectionScope<'s> {
    folders: Vec<&'s str>,
    resolver: &'s Resolver<'s>,
}

impl<'s> CollectionScope<'s> {
    pub(crate) fn new(resolver: &'s Resolver) -> Self {
        Self::with_folders(resolver, vec![])
    }
    pub(crate) fn with_folders(resolver: &'s Resolver, folders: Vec<&'s str>) -> Self {
        Self { folders, resolver }
    }
    pub(crate) fn track_folder(&mut self, object: &'s Map<String, Value>) {
        // Some objects may change `$ref` behavior via the `$id` keyword
        if let Some(id) = id_of_object(object) {
            self.folders.push(id);
        }
    }
}

/// Storage for intermediate collection data.
pub(crate) struct Collector<'s> {
    resolvers: &'s HashMap<&'s str, Resolver<'s>>,
    /// Nodes of the input schema flattened into a vector.
    nodes: Vec<&'s Value>,
    keywords: KeywordMap<'s>,
    edges: EdgeMap,
    /// Nodes already seen during collection.
    seen: SeenNodeMap,
}

impl<'s> Collector<'s> {
    /// Create a new collector.
    pub(crate) fn new(resolvers: &'s HashMap<&str, Resolver>) -> Self {
        Self {
            resolvers,
            nodes: vec![],
            keywords: KeywordMap::default(),
            edges: EdgeMap::default(),
            seen: SeenNodeMap::default(),
        }
    }

    /// Push a value to the tree & return its slot.
    fn push(&mut self, value: &'s Value) -> NodeSlot {
        match self.seen.entry(value) {
            Entry::Occupied(entry) => NodeSlot::seen(*entry.get()),
            Entry::Vacant(entry) => {
                let node_id = NodeId(self.nodes.len());
                self.nodes.push(value);
                entry.insert(node_id);
                NodeSlot::new(node_id)
            }
        }
    }

    fn add_value(
        &mut self,
        parent_id: NodeId,
        value: &'s Value,
        label: impl Into<EdgeLabel>,
    ) -> NodeSlot {
        let slot = self.push(value);
        self.edges
            .entry(parent_id)
            .or_insert_with(Vec::new)
            .push(single(label, parent_id, slot.id));
        slot
    }

    fn add_keyword(&mut self, parent_id: NodeId, value: &'s Value, name: &'static str) -> NodeSlot {
        let slot = self.push(value);
        self.keywords
            .entry(parent_id)
            .or_insert_with(Vec::new)
            .push(KeywordNode::new(slot.id, value, name));
        slot
    }

    pub(crate) fn collect(
        mut self,
        node: &'s Value,
        resolver: &'s Resolver,
    ) -> Result<(KeywordMap<'s>, EdgeMap)> {
        let mut scope = CollectionScope::new(resolver);
        let slot = self.push(node);
        self.collect_schema(node, slot.id, &mut scope)?;
        dbg!(&self.nodes);
        Ok((self.keywords, self.edges))
    }

    fn collect_schema(
        &mut self,
        schema: &'s Value,
        parent_id: NodeId,
        scope: &mut CollectionScope<'s>,
    ) -> Result<()> {
        if let Value::Object(object) = schema {
            scope.track_folder(object);
            for (key, value) in object {
                match key.as_str() {
                    "$ref" => {
                        if let Value::String(reference) = value {
                            self.collect_reference(reference, parent_id, scope)?;
                        }
                    }
                    "maximum" => {
                        self.add_keyword(parent_id, value, "maximum");
                    }
                    "maxLength" => {
                        self.add_keyword(parent_id, value, "maxLength");
                    }
                    "minProperties" => {
                        self.add_keyword(parent_id, value, "minProperties");
                    }
                    "type" => {
                        self.add_keyword(parent_id, value, "type");
                    }
                    "allOf" => {
                        if let Value::Array(items) = value {
                            let all_of = self.add_keyword(parent_id, value, "allOf");
                            if all_of.is_new() {
                                for (id, schema) in items.iter().enumerate() {
                                    let value = self.add_value(all_of.id, schema, id);
                                    if value.is_new() {
                                        self.collect_schema(schema, value.id, scope)?;
                                    }
                                }
                            }
                        }
                    }
                    "items" => {
                        let items = self.add_keyword(parent_id, value, "items");
                        if items.is_new() {
                            self.collect_schema(value, items.id, scope)?;
                        }
                    }
                    "properties" => {
                        if let Value::Object(object) = value {
                            let properties = self.add_keyword(parent_id, value, "properties");
                            if properties.is_new() {
                                for (key, schema) in object {
                                    let value = self.add_value(properties.id, schema, key);
                                    if value.is_new() {
                                        self.collect_schema(schema, value.id, scope)?;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                };
            }
        }
        Ok(())
    }

    fn collect_reference(
        &mut self,
        reference: &str,
        parent_id: NodeId,
        scope: &mut CollectionScope<'s>,
    ) -> Result<()> {
        // Resolve reference & traverse it.
        let (slot, value) = match Reference::try_from(reference)? {
            Reference::Absolute(location) => {
                if let Some(resolver) = self.resolvers.get(location.as_str()) {
                    let (folders, resolved) = resolver.resolve(reference)?;
                    let slot = self.push(resolved);
                    let mut scope = CollectionScope::with_folders(resolver, folders);
                    self.collect_schema(resolved, slot.id, &mut scope)?;
                    (slot, resolved)
                } else {
                    let (_, resolved) = scope.resolver.resolve(reference)?;
                    (self.push(resolved), resolved)
                }
            }
            Reference::Relative(location) => {
                let mut resolver = scope.resolver;
                if !is_local(location) {
                    let location = with_folders(resolver.scope(), location, &scope.folders)?;
                    if !resolver.contains(location.as_str()) {
                        resolver = self
                            .resolvers
                            .get(location.as_str())
                            .expect("Unknown reference");
                    }
                };
                let (folders, resolved) = resolver.resolve(location)?;
                let slot = self.push(resolved);
                if slot.is_new() {
                    // New value - traverse it
                    let mut scope = CollectionScope::with_folders(resolver, folders);
                    self.collect_schema(resolved, slot.id, &mut scope)?;
                }
                (slot, resolved)
            }
        };
        self.keywords
            .entry(parent_id)
            .or_insert_with(Vec::new)
            .push(KeywordNode::new(slot.id, value, "$ref"));
        Ok(())
    }
}
