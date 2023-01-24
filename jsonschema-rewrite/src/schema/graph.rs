use crate::{
    schema::{
        edges,
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

pub(crate) type KeywordNode<'s> = (usize, &'s Value, &'static str);
pub(crate) type KeywordMap<'s> = BTreeMap<usize, Vec<KeywordNode<'s>>>;
pub(crate) type EdgeMap = BTreeMap<usize, Vec<edges::SingleEdge>>;

/// The main goal of this phase is to collect all nodes from the input schema and its remote
/// dependencies into a single graph where each node is a reference to a JSON value from these
/// schemas. Each edge is represented as a pair indexes into the node vector and a label.
pub(crate) fn build<'s>(
    schema: &'s Value,
    root_resolver: &'s Resolver,
    resolvers: &'s HashMap<&str, Resolver>,
) -> Result<(usize, Vec<Keyword>, Vec<edges::MultiEdge>)> {
    let (keyword_map, edge_map) = collect(schema, root_resolver, resolvers)?;
    let mut keywords = vec![];
    let mut edges = vec![];

    macro_rules! push_edge {
        ($target:expr, $next:expr) => {{
            // TODO: It will not work for $ref - it will point to some other keywords
            for edge in &edge_map[$target] {
                let end = $next + keyword_map[&edge.target].len();
                edges.push(edges::multi(edge.label.clone(), $next..end));
                $next = end;
            }
        }};
    }

    macro_rules! next_edges {
        ($target:expr) => {
            edges.len()..edges.len() + edge_map[$target].len()
        };
    }

    let root_offset = keyword_map.get(&0).map_or(0, Vec::len);
    for node_keywords in keyword_map.values() {
        let mut next = keywords.len() + node_keywords.len();
        for (target, value, keyword) in node_keywords {
            match *keyword {
                "allOf" => {
                    keywords.push(AllOf::build(next_edges!(target)));
                    push_edge!(target, next);
                }
                "items" => {}
                "maximum" => keywords.push(Maximum::build(value.as_u64().unwrap())),
                "maxLength" => keywords.push(MaxLength::build(value.as_u64().unwrap())),
                "minProperties" => keywords.push(MinProperties::build(value.as_u64().unwrap())),
                "properties" => {
                    keywords.push(Properties::build(next_edges!(target)));
                    push_edge!(target, next);
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

#[derive(Debug, Eq, PartialEq)]
enum ValueEntry {
    Occupied,
    Vacant,
}
use ValueEntry::{Occupied, Vacant};

/// Storage for intermediate collection data.
pub(crate) struct Collector<'s> {
    resolvers: &'s HashMap<&'s str, Resolver<'s>>,
    /// Nodes of the input schema.
    nodes: Vec<&'s Value>,
    keywords: KeywordMap<'s>,
    edges: EdgeMap,
    /// Nodes already seen during collection.
    seen: HashMap<*const Value, usize>,
}

impl<'s> Collector<'s> {
    /// Create a new collector.
    pub(crate) fn new(resolvers: &'s HashMap<&str, Resolver>) -> Self {
        Self {
            resolvers,
            nodes: vec![],
            keywords: BTreeMap::default(),
            edges: BTreeMap::default(),
            seen: HashMap::default(),
        }
    }

    /// Push a value to the tree.
    /// If value already exists there - return its index.
    fn push(&mut self, value: &'s Value) -> (ValueEntry, usize) {
        match self.seen.entry(value) {
            Entry::Occupied(entry) => (Occupied, *entry.get()),
            Entry::Vacant(entry) => {
                let node_id = self.nodes.len();
                self.nodes.push(value);
                entry.insert(node_id);
                (Vacant, node_id)
            }
        }
    }

    fn add_value(
        &mut self,
        source: usize,
        value: &'s Value,
        label: impl Into<edges::EdgeLabel>,
    ) -> (ValueEntry, usize) {
        let (entry, target) = self.push(value);
        self.edges
            .entry(source)
            .or_insert_with(Vec::new)
            .push(edges::single(label, source, target));
        (entry, target)
    }
    fn add_keyword(
        &mut self,
        source: usize,
        value: &'s Value,
        keyword: &'static str,
    ) -> (ValueEntry, usize) {
        let (entry, target) = self.push(value);
        self.keywords
            .entry(source)
            .or_insert_with(Vec::new)
            .push((target, value, keyword));
        (entry, target)
    }

    pub(crate) fn collect(
        mut self,
        node: &'s Value,
        resolver: &'s Resolver,
    ) -> Result<(KeywordMap<'s>, EdgeMap)> {
        let mut scope = CollectionScope::new(resolver);
        let (_, node_id) = self.push(node);
        self.collect_schema(node, node_id, &mut scope)?;
        Ok((self.keywords, self.edges))
    }

    fn collect_schema(
        &mut self,
        schema: &'s Value,
        parent_id: usize,
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
                        if let (Vacant, source) = self.add_keyword(parent_id, value, "allOf") {
                            if let Value::Array(items) = value {
                                for (id, schema) in items.iter().enumerate() {
                                    if let (Vacant, id) = self.add_value(source, schema, id) {
                                        self.collect_schema(schema, id, scope)?;
                                    }
                                }
                            }
                        }
                    }
                    "items" => {
                        if let (Vacant, id) = self.add_keyword(parent_id, value, "items") {
                            self.collect_schema(value, id, scope)?;
                        }
                    }
                    "properties" => {
                        if let (Vacant, source) = self.add_keyword(parent_id, value, "properties") {
                            if let Value::Object(object) = value {
                                for (key, schema) in object {
                                    if let (Vacant, id) = self.add_value(source, schema, key) {
                                        self.collect_schema(schema, id, scope)?;
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
        source: usize,
        scope: &mut CollectionScope<'s>,
    ) -> Result<()> {
        // Resolve reference & traverse it.
        let (target, resolved) = match Reference::try_from(reference)? {
            Reference::Absolute(location) => {
                if let Some(resolver) = self.resolvers.get(location.as_str()) {
                    let (folders, resolved) = resolver.resolve(reference)?;
                    let (_, target) = self.push(resolved);
                    let mut scope = CollectionScope::with_folders(resolver, folders);
                    self.collect_schema(resolved, target, &mut scope)?;
                    (target, resolved)
                } else {
                    let (_, resolved) = scope.resolver.resolve(reference)?;
                    (self.push(resolved).1, resolved)
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
                // Push the resolved value onto the stack to explore them further
                match self.push(resolved) {
                    (Vacant, target) => {
                        let mut scope = CollectionScope::with_folders(resolver, folders);
                        self.collect_schema(resolved, target, &mut scope)?;
                        (target, resolved)
                    }
                    (Occupied, target) => (target, resolved),
                }
            }
        };
        self.keywords
            .entry(source)
            .or_insert_with(Vec::new)
            .push((target, resolved, "$ref"));
        Ok(())
    }
}
