//! Compressed sparse row format for JSON Schema.
//!
//! Fast and cache efficient validation requires fast iteration over the schema, therefore a
//! representation like `serde_json::Value` should be converted to a more compact one.
//!
//! JSON Schema is a directed graph, where cycles could be represented via the `$ref` keyword.
//! It contains two types of edges:
//!   - Concrete edges are regular Rust references that connect two JSON values;
//!   - Virtual edges are using the `$ref` keyword to do this.
//!
//! TODO. add more theory about how `serde_json::Value` is represented and how CSR is represented
use crate::{
    compilation::resolver::{id_of_object, is_default_scope, scope_of, Resolver},
    vocabularies::Keyword,
};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use url::Url;

pub(crate) mod edges;
mod error;
pub mod resolver;
use error::Result;

use crate::{
    compilation::{
        edges::EdgeLabel,
        resolver::{parse_reference, Reference},
    },
    vocabularies::{applicator, validation},
};
use edges::{CompressedEdge, RawEdge};

// TODO. Optimization ideas:
//   - Values ordering. "Cheaper" keywords might work better if they are executed first.
//     Example: `anyOf` where some items are very cheap and common to pass validation
//   - Interleave values & edges in the same struct. Might improve cache locality.
//   - Remove nodes that are not references by anything & not needed for validation.
//     Might reduce the graph size.
//   - Store root's keywords explicitly upfront to avoid calculation for small schemas

#[derive(Debug)]
pub struct JsonSchema {
    pub(crate) keywords: Box<[Keyword]>,
    head_end: usize,
    offsets: Box<[usize]>,
    pub(crate) edges: Box<[CompressedEdge]>,
}

impl JsonSchema {
    pub fn new(schema: &Value) -> Result<Self> {
        // Resolver for the root schema - needed to resolve location-independent references during
        // the initial resolving step
        let root_resolver = Resolver::new(schema, scope_of(schema)?);
        // Fetch all external schemas reachable from the root
        let external = fetch_external(schema, &root_resolver)?;
        // Build resolvers for external schemas
        let resolvers = build_resolvers(&external);
        // Collect all values and resolve references
        let (values, mut edges) = collect(schema, &resolvers)?;
        // Build a `Keyword` graph together with dropping not needed nodes and edges
        let (keywords, edges) = materialize(values, &mut edges);
        // And finally, compress the edges into the CSR format
        let (offsets, edges) = compress(edges);
        Ok(JsonSchema {
            keywords: keywords.into_boxed_slice(),
            head_end: 1,
            offsets: offsets.into_boxed_slice(),
            edges: edges.into_boxed_slice(),
        })
    }

    #[inline]
    pub(crate) fn edges_of(&self, node_idx: usize) -> impl Iterator<Item = &CompressedEdge> {
        let start = self.offsets[node_idx];
        let end = self.offsets[node_idx + 1];
        self.edges[start..end].iter()
    }

    pub fn is_valid(&self, instance: &Value) -> bool {
        self.keywords[..self.head_end]
            // TODO. each keyword should know its position, so it can find its edges
            .iter()
            .all(|keyword| keyword.is_valid(self, instance))
    }
}

/// Fetch all external schemas reachable from the root.
fn fetch_external(schema: &Value, resolver: &Resolver) -> Result<HashMap<Url, Value>> {
    let mut store = HashMap::new();
    fetch_routine(schema, &mut store, resolver)?;
    Ok(store)
}

const REF: &str = "$ref";

/// Recursive routine for traversing a schema and fetching external references.
fn fetch_routine(
    schema: &Value,
    store: &mut HashMap<Url, Value>,
    resolver: &Resolver,
) -> Result<()> {
    // Current schema id - if occurs in a reference, then there is no need to resolve it
    let scope = scope_of(schema)?;
    let mut stack = vec![(vec![], schema)];
    while let Some((mut folders, value)) = stack.pop() {
        match value {
            // Only objects may have references to external schemas
            Value::Object(object) => {
                if let Some(id) = id_of_object(object) {
                    folders.push(id);
                }
                for (key, value) in object {
                    if key == REF
                        && value
                            .as_str()
                            .map_or(false, |value| !is_local_reference(value))
                    {
                        if let Some(reference) = value.as_str() {
                            if resolver.contains(reference) {
                                continue;
                            }
                        }
                        fetch_external_reference(value, &folders, store, &scope, resolver)?;
                    } else {
                        // Explore any other key
                        stack.push((folders.clone(), value))
                    }
                }
            }
            // Explore arrays further
            Value::Array(items) => {
                for item in items {
                    stack.push((folders.clone(), item))
                }
            }
            // Primitive types do not contain any references, skip
            _ => continue,
        }
    }
    Ok(())
}

/// If reference is pointing to an external resource, then fetch & store it.
fn fetch_external_reference(
    value: &Value,
    folders: &[&str],
    store: &mut HashMap<Url, Value>,
    scope: &Url,
    resolver: &Resolver,
) -> Result<()> {
    if let Some(location) = without_fragment(value) {
        // Resolve only references that are:
        //   - pointing to another resource
        //   - are not already resolved
        fetch_and_store(store, scope, location, resolver)?;
    } else if !is_default_scope(scope) {
        if let Some(reference) = value.as_str() {
            let location = with_folders(scope, reference, folders)?;
            fetch_and_store(store, scope, location, resolver)?;
        }
    }
    Ok(())
}

fn with_folders(scope: &Url, reference: &str, folders: &[&str]) -> Result<Url> {
    let mut location = scope.clone();
    if folders.len() > 1 {
        for folder in folders.iter().skip(1) {
            location = location.join(folder)?;
        }
    }
    Ok(location.join(reference)?)
}

fn fetch_and_store(
    store: &mut HashMap<Url, Value>,
    scope: &Url,
    location: Url,
    resolver: &Resolver,
) -> Result<()> {
    if location != *scope && !store.contains_key(&location) && !resolver.contains(location.as_str())
    {
        let response = reqwest::blocking::get(location.as_str())?;
        let document = response.json::<Value>()?;
        // Make a recursive call to the callee routine
        fetch_routine(&document, store, resolver)?;
        store.insert(location, document);
    }
    Ok(())
}

/// Extract a fragment-less URL from the reference value.
fn without_fragment(value: &Value) -> Option<Url> {
    if let Some(Ok(mut location)) = value.as_str().map(Url::parse) {
        location.set_fragment(None);
        Some(location)
    } else {
        None
    }
}

fn build_resolvers(external: &HashMap<Url, Value>) -> HashMap<&str, Resolver> {
    let mut resolvers = HashMap::with_capacity(external.len());
    for (scope, document) in external {
        resolvers.insert(scope.as_str(), Resolver::new(document, scope.clone()));
    }
    resolvers
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ValueReference<'schema> {
    /// Reference to a concrete JSON value.
    Concrete(&'schema Value),
    /// Resolved `$ref` to a JSON value.
    Virtual(&'schema Value),
}

#[derive(Debug)]
struct SeenValues(HashSet<*const Value>);

impl SeenValues {
    pub(crate) fn new() -> Self {
        Self(HashSet::new())
    }
    #[inline]
    pub(crate) fn insert(&mut self, value: &Value) -> bool {
        self.0.insert(value as *const _)
    }
}

#[derive(Debug)]
struct Values<'a>(Vec<ValueReference<'a>>);

impl<'a> Values<'a> {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }
    pub(crate) fn add_concrete(&mut self, value: &'a Value) {
        self.0.push(ValueReference::Concrete(value))
    }
    pub(crate) fn add_virtual(&mut self, value: &'a Value) {
        self.0.push(ValueReference::Virtual(value))
    }
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
    pub(crate) fn into_inner(self) -> Vec<ValueReference<'a>> {
        self.0
    }
}

fn is_local_reference(reference: &str) -> bool {
    reference.starts_with('#')
}

macro_rules! push {
    ($stack: expr, $idx: expr, $value: expr, $label: expr, $seen: expr, $resolver: expr, $folders: expr) => {
        // Push the new value only if it wasn't seen yet.
        // Insert it immediately to avoid pushing references to the same target from multiple places
        if $seen.insert($value) {
            $stack.push((
                Some(($idx, crate::compilation::edges::label($label))),
                $folders.clone(),
                $resolver,
                $value,
            ));
        }
    };
}

/// The main goal of this phase is to collect all nodes from the input schema and its remote
/// dependencies into a single graph where each vertex is a reference to a JSON value from these
/// schemas. Each edge is represented as a pair indexes into the vertex vector.
///
/// This representation format is efficient to construct the schema graph, but not for input
/// validation. Input requires arbitrary traversal order from the root node, because it depends on
/// the input value - certain schema branches are needed only for certain types or values.
/// For example, `properties` sub-schemas are needed only if the input contains matching keys.
fn collect<'a>(
    schema: &'a Value,
    resolvers: &'a HashMap<&str, Resolver>,
) -> Result<(Vec<ValueReference<'a>>, Vec<RawEdge>)> {
    let mut values = Values::new();
    let mut edges = vec![];
    let resolver = Resolver::new(schema, scope_of(schema)?);
    let mut stack = vec![(None, vec![], &resolver, schema)];
    let mut seen = SeenValues::new();
    while let Some((parent, mut folders, mut resolver, node)) = stack.pop() {
        let node_idx = values.len();
        // Mark this value as seen to prevent re-traversing it if any reference leads to it
        seen.insert(node);
        values.add_concrete(node);
        match node {
            Value::Object(object) => {
                // Track folder changes within sub-schemas
                if let Some(id) = id_of_object(object) {
                    folders.push(id);
                }
                for (key, value) in object {
                    if key == REF {
                        if let Value::String(ref_string) = value {
                            match parse_reference(ref_string)? {
                                Reference::Absolute(location) => {
                                    let resolved = if let Some(resolver) =
                                        resolvers.get(location.as_str())
                                    {
                                        let (folders, resolved) = resolver.resolve(ref_string)?;
                                        push!(
                                            stack, node_idx, resolved, REF, seen, resolver, folders
                                        );
                                        resolved
                                    } else {
                                        let (_, resolved) = resolver.resolve(ref_string)?;
                                        resolved
                                    };
                                    values.add_virtual(resolved);
                                    push!(stack, node_idx, value, REF, seen, resolver, folders);
                                }
                                Reference::Relative(location) => {
                                    if !is_local_reference(location) {
                                        let location =
                                            with_folders(resolver.scope(), location, &folders)?;
                                        if !resolver.contains(location.as_str()) {
                                            resolver = resolvers
                                                .get(location.as_str())
                                                .expect("Unknown reference");
                                        }
                                    };
                                    let (folders, resolved) = resolver.resolve(location)?;
                                    values.add_virtual(resolved);
                                    // Push the resolved value & the reference itself onto the stack
                                    // to explore them further
                                    for value in [resolved, value] {
                                        push!(stack, node_idx, value, REF, seen, resolver, folders);
                                    }
                                }
                            }
                        } else {
                            // The `$ref` value is not a string - explore it further
                            push!(stack, node_idx, value, REF, seen, resolver, folders);
                        };
                    } else {
                        push!(stack, node_idx, value, key, seen, resolver, folders);
                    }
                }
            }
            Value::Array(items) => {
                for (idx, item) in items.iter().enumerate() {
                    push!(stack, node_idx, item, idx, seen, resolver, folders);
                }
            }
            _ => {}
        };
        if let Some((parent_idx, label)) = parent {
            edges.push(RawEdge::new(parent_idx, node_idx, label));
        }
    }
    Ok((values.into_inner(), edges))
}

/// Build a graph of `Keyword` from an intermediate graph that contain all possible values.
///
/// There are a few optimizations happen here:
///  - Inlining virtual references
///  - Skipping not needed nodes
fn materialize(
    values: Vec<ValueReference>,
    edges: &mut Vec<RawEdge>,
) -> (Vec<Keyword>, Vec<RawEdge>) {
    // TODO: edges should also be new - input ones refer to the original graph
    // TODO: Some edges are useless - they don't represent any useful keyword
    let new_edges = vec![];
    let mut keywords = Vec::with_capacity(values.len());
    for edge in edges {
        // TODO. check the keyword first before checking the value - in some cases, values are not needed if keyword is skipped
        match &values[edge.target] {
            ValueReference::Concrete(value) => match &edge.label {
                EdgeLabel::Key(key) => match key.as_ref() {
                    // TODO. it should be at the right index - otherwise the graph is broken
                    "maximum" => keywords.push(validation::Maximum::build(value.as_u64().unwrap())),
                    "properties" => keywords.push(applicator::Properties::build()),
                    _ => {}
                },
                EdgeLabel::Index(_) => {}
            },
            // Each node that has an edge to `$ref` could be replaced by a new edge that leads
            // directly to the $ref target. Example:
            //
            //  {
            //     "integer": {
            //         "type": "integer"
            //     },
            //     "refToInteger": {
            //         "$ref": "#/integer"
            //     }
            // }
            //
            // This schema could be represented as:
            // [
            //     Concrete(root),                -- 0
            //     Virtual({"type": "integer"})   -- 1. refToInteger
            //     Concrete({"type": "integer"})  -- 2. integer
            //     Concrete("integer")            -- 3. type
            // ]
            //
            // Edges: 0-1 (refToInteger), 0-2 (integer), 2-3 (type)
            //
            // Here we change the first edge to 0-2. After all edges leading to this node are
            // inlined, node #1 is not needed anymore
            ValueReference::Virtual(reference) => {
                // TODO. detect cycles - get the target unless it is concrete + collect seen idx
                //   if idx is seen, then it is a cycle or refs without concrete nodes in between
                // Find the concrete reference by comparing the pointers
                // All virtual references point to values that are always in the `values` vector
                // Therefore this loop should always find such a reference
                // TODO: Avoid O^2 - store seen targets & their indices.
                //       If not seen - check only the right side.
                for (target_idx, target) in values.iter().enumerate() {
                    if let ValueReference::Concrete(target) = target {
                        if std::ptr::eq(*reference, *target) {
                            edge.target = target_idx;
                        }
                    }
                }
            }
        }
    }
    (keywords, new_edges)
}

fn compress(mut edges: Vec<RawEdge>) -> (Vec<usize>, Vec<CompressedEdge>) {
    edges.sort_unstable_by_key(|edge| edge.source);
    let max_node_id = match edges
        .iter()
        .map(|edge| std::cmp::max(edge.source, edge.target))
        .max()
    {
        None => return (vec![0], vec![]),
        Some(id) => id,
    };
    let mut iter = edges.iter().peekable();
    let mut edges = Vec::new();
    let mut offsets = vec![0_usize; max_node_id + 2];
    let mut rows = offsets.iter_mut();
    let mut start = 0;
    'outer: for (node, row) in (&mut rows).enumerate() {
        *row = start;
        'inner: loop {
            if let Some(edge) = iter.peek() {
                if edge.source != node {
                    break 'inner;
                }
                edges.push(edge.compress());
                start += 1;
            } else {
                break 'outer;
            }
            iter.next();
        }
    }
    for row in rows {
        *row = start;
    }
    (offsets, edges)
}

#[cfg(test)]
mod tests {
    use super::{applicator::Properties, edges::EdgeLabel, validation::Maximum, *};
    use bench_helpers::read_json;
    use serde_json::{json as j, Value};
    use std::fs;
    use test_case::test_case;

    macro_rules! c {
        ($el:tt) => {
            ValueReference::Concrete(&j!($el))
        };
    }

    macro_rules! v {
        ($el:tt) => {
            ValueReference::Virtual(&j!($el))
        };
    }

    macro_rules! k {
        ($el:expr) => {
            Keyword::from($el)
        };
    }

    /// All schema nodes are present as concrete references
    fn assert_all_schema_nodes(schema: &Value, values: &[ValueReference]) {
        let mut stack = vec![schema];
        'outer: while let Some(value) = stack.pop() {
            match value {
                Value::Object(object) => stack.extend(object.values()),
                Value::Array(items) => stack.extend(items.iter()),
                _ => {}
            }
            for reference in values {
                if let ValueReference::Concrete(target) = reference {
                    if std::ptr::eq(value, *target) {
                        continue 'outer;
                    }
                }
            }
            panic!("Value `{}` is not found as a concrete reference", value)
        }
    }

    /// Ensure every concrete reference is unique
    fn assert_concrete_references(values: &[ValueReference]) {
        let mut seen = HashMap::new();
        for (index, value) in values.iter().enumerate() {
            if let ValueReference::Concrete(reference) = value {
                if let Some(existing_index) = seen.insert(*reference as *const _, index) {
                    panic!(
                        "Concrete reference `{}` at index {} was already seen at index {}",
                        reference, index, existing_index
                    )
                }
            }
        }
    }

    /// Ensure every virtual reference points to a concrete one.
    fn assert_virtual_references(values: &[ValueReference]) {
        'outer: for (reference_index, value) in values.iter().enumerate() {
            if let ValueReference::Virtual(reference) = value {
                for (target_index, target) in values.iter().enumerate() {
                    if let ValueReference::Concrete(target) = target {
                        println!(
                            "Compare\n  `{}` ({:p}) at {} vs `{}` ({:p}) at {}",
                            reference,
                            *reference as *const _,
                            reference_index,
                            target,
                            *target as *const _,
                            target_index
                        );
                        if std::ptr::eq(*reference, *target) {
                            // Found! Check the next one
                            println!(
                                "Found for `{}` ({:p}) at {}",
                                reference, *reference as *const _, reference_index
                            );
                            continue 'outer;
                        }
                    }
                }
                panic!(
                    "Failed to find a concrete reference for a virtual reference `{}` at index {}",
                    reference, reference_index
                )
            }
        }
    }

    fn edge(source: usize, target: usize, label: impl Into<EdgeLabel>) -> RawEdge {
        RawEdge {
            source,
            target,
            label: label.into(),
        }
    }

    fn print_values(values: &[ValueReference]) {
        for (i, v) in values.iter().enumerate() {
            match v {
                ValueReference::Concrete(r) => {
                    println!("C({}): {}", i, r)
                }
                ValueReference::Virtual(r) => {
                    println!("V({}): {}", i, r)
                }
            }
        }
    }

    const SELF_REF: &str = "#";
    const REMOTE_REF: &str = "http://localhost:1234/subSchemas.json#/integer";
    const REMOTE_BASE: &str = "http://localhost:1234/subSchemas.json";

    #[test_case(
        j!(true),
        &[c!(true)],
        &[],
        &[];
        "Boolean schema"
    )]
    #[test_case(
        j!({"maximum": 5}),
        &[
            c!({"maximum": 5}),
            c!(5),
        ],
        &[edge(0, 1, "maximum")],
        &[];
        "No reference"
    )]
    #[test_case(
        j!({
            "properties": {
                "$ref": {
                    "type": "string"
                }
            }
        }),
        &[
            c!({"properties":{"$ref":{"type":"string"}}}),
            c!({"$ref":{"type":"string"}}),
            c!({"type":"string"}),
            c!("string"),
        ],
        &[
            edge(0, 1, "properties"),
            edge(1, 2, "$ref"),
            edge(2, 3, "type"),
        ],
        &[];
        "Not a reference"
    )]
    #[test_case(
        j!({"$ref": SELF_REF}),
        &[
            c!({"$ref": SELF_REF}),
            v!({"$ref": SELF_REF}),
            c!(SELF_REF),
        ],
        &[edge(0, 2, "$ref")],
        &[];
        "Self reference"
    )]
    #[test_case(
        j!({"$ref": REMOTE_REF}),
        &[
            c!({"$ref": REMOTE_REF}),
            v!({"type": "integer"}),
            c!(REMOTE_REF),
            c!({"type": "integer"}),
            c!("integer"),
        ],
        &[
            edge(0, 2, "$ref"),
            edge(0, 3, "$ref"),
            edge(3, 4, "type")
        ],
        &[REMOTE_BASE];
        "Remote reference"
    )]
    #[test_case(
        j!({
            "$id": "http://localhost:1234/",
            "items": {
                "$id": "baseUriChange/",
                "items": {"$ref": "folderInteger.json"}
            }
        }),
        &[
            c!({
                "$id": "http://localhost:1234/",
                "items": {
                    "$id": "baseUriChange/",
                    "items": {"$ref": "folderInteger.json"}
                }
            }),
            c!({
                "$id": "baseUriChange/",
                "items": {"$ref": "folderInteger.json"}
            }),
            c!({"$ref": "folderInteger.json"}),
            v!({"type": "integer"}),
            c!("folderInteger.json"),
            c!({"type": "integer"}),
            c!("integer"),
            c!("baseUriChange/"),
            c!("http://localhost:1234/"),
        ],
        &[
            edge(0, 1, "items"),
            edge(1, 2, "items"),
            edge(2, 4, "$ref"),
            edge(2, 5, "$ref"),
            edge(5, 6, "type"),
            edge(1, 7, "$id"),
            edge(0, 8, "$id"),
        ],
        &["http://localhost:1234/baseUriChange/folderInteger.json"];
        "Base URI change"
    )]
    #[test_case(
        j!({
            "$id": "http://localhost:1234/scope_change_defs1.json",
            "properties": {
                "list": {"$ref": "#/definitions"}
            },
            "definitions": {
                "$id": "baseUriChangeFolder/",
            }
        }),
        &[
            c!({
                "$id": "http://localhost:1234/scope_change_defs1.json",
                "properties": {
                    "list": {"$ref": "#/definitions"}
                },
                "definitions": {
                    "$id": "baseUriChangeFolder/",
                }
            }),
            c!({"list":{"$ref":"#/definitions"}}),
            c!({"$ref":"#/definitions"}),
            v!({"$id":"baseUriChangeFolder/"}),
            c!("#/definitions"),
            c!({"$id":"baseUriChangeFolder/"}),
            c!("baseUriChangeFolder/"),
            c!("http://localhost:1234/scope_change_defs1.json"),
        ],
        &[
            edge(0, 1, "properties"),
            edge(1, 2, "list"),
            edge(2, 4, "$ref"),
            edge(0, 5, "definitions"),
            edge(5, 6, "$id"),
            edge(0, 7, "$id"),
        ],
        &[];
        "Base URI change - change folder"
    )]
    #[test_case(
        j!({
            "$ref": "http://localhost:1234/subSchemas.json#/refToInteger"
        }),
        &[
            c!({
                "$ref": "http://localhost:1234/subSchemas.json#/refToInteger"
            }),
            v!({"$ref":"#/integer"}),
            c!("http://localhost:1234/subSchemas.json#/refToInteger"),
            c!({"$ref":"#/integer"}),
            v!({"type":"integer"}),
            c!("#/integer"),
            c!({"type":"integer"}),
            c!("integer"),
        ],
        &[
            edge(0, 2, "$ref"),
            edge(0, 3, "$ref"),
            edge(3, 5, "$ref"),
            edge(3, 6, "$ref"),
            edge(6, 7, "type"),
        ],
        &["http://localhost:1234/subSchemas.json"];
        "Reference within remote reference"
    )]
    #[test_case(
        j!({
            "$id": "http://localhost:1234/root",
            "properties": {
                "A": {
                    "$ref": "http://localhost:1234/root"
                }
            }
        }),
        &[
            c!({
                "$id": "http://localhost:1234/root",
                "properties": {
                    "A": {
                        "$ref": "http://localhost:1234/root"
                    }
                }
            }),
            c!({
                "A": {
                    "$ref": "http://localhost:1234/root"
                }
            }),
            c!({"$ref": "http://localhost:1234/root"}),
            v!({
                "$id": "http://localhost:1234/root",
                "properties": {
                    "A": {
                        "$ref": "http://localhost:1234/root"
                    }
                }
            }),
            c!("http://localhost:1234/root"),
            c!("http://localhost:1234/root"),
        ],
        &[
            edge(0, 1, "properties"),
            edge(1, 2, "A"),
            edge(2, 4, "$ref"),
            edge(0, 5, "$id"),
        ],
        &[];
        "Absolute reference to the same schema"
    )]
    #[test_case(
        j!({
            "allOf": [
                {"$ref": "#/allOf/1"},
                {"$ref": "#/allOf/0"}
            ]
        }),
        &[
            c!({"allOf":[{"$ref":"#/allOf/1"},{"$ref":"#/allOf/0"}]}),
            c!([{"$ref":"#/allOf/1"},{"$ref":"#/allOf/0"}]),
            c!({"$ref":"#/allOf/0"}),
            v!({"$ref":"#/allOf/1"}),
            c!("#/allOf/0"),
            c!({"$ref":"#/allOf/1"}),
            v!({"$ref":"#/allOf/0"}),
            c!("#/allOf/1"),
        ],
        &[
            edge(0, 1, "allOf"),
            edge(1, 2, 1),
            edge(2, 4, "$ref"),
            edge(1, 5, 0),
            edge(5, 7, "$ref"),
        ],
        &[];
        "Multiple references to the same target"
    )]
    #[test_case(
        j!({
            "$id": "http://localhost:1234/tree",
            "d": {
                "$id": "http://localhost:1234/node",
                "s": {
                    "$ref": "tree"
                },
            },
            "n": {
                "$ref": "node"
            }
        }),
        &[
            c!({"$id":"http://localhost:1234/tree","d":{"$id":"http://localhost:1234/node","s":{"$ref":"tree"}},"n":{"$ref":"node"}}),
            c!({"$ref":"node"}),
            v!({"$id":"http://localhost:1234/node","s":{"$ref":"tree"}}),
            c!("node"),
            c!({"$id":"http://localhost:1234/node","s":{"$ref":"tree"}}),
            c!({"$ref":"tree"}),
            v!({"$id":"http://localhost:1234/tree","d":{"$id":"http://localhost:1234/node","s":{"$ref":"tree"}},"n":{"$ref":"node"}}),
            c!("tree"),
            c!("http://localhost:1234/node"),
            c!("http://localhost:1234/tree"),
        ],
        &[
            edge(0, 1, "n"),
            edge(1, 3, "$ref"),
            edge(0, 4, "d"),
            edge(4, 5, "s"),
            edge(5, 7, "$ref"),
            edge(4, 8, "$id"),
            edge(0, 9, "$id"),
        ],
        &[];
        "Recursive references between schemas"
    )]
    fn values_and_edges(
        schema: Value,
        values: &[ValueReference],
        edges: &[RawEdge],
        keys: &[&str],
    ) {
        let root = Resolver::new(&schema, scope_of(&schema).unwrap());
        let external = fetch_external(&schema, &root).unwrap();
        let resolvers = build_resolvers(&external);
        let (values_, edges_) = collect(&schema, &resolvers).unwrap();
        print_values(&values_);
        assert_eq!(values_, values);
        assert_all_schema_nodes(&schema, &values_);
        assert_concrete_references(&values_);
        assert_virtual_references(&values_);
        assert_eq!(edges_, edges);
        assert_eq!(resolvers.keys().cloned().collect::<Vec<&str>>(), keys);
    }

    #[test]
    fn all_schemas() {
        for draft in &[7] {
            let paths =
                fs::read_dir(format!("../jsonschema/tests/suite/tests/draft{}", draft)).unwrap();
            for path in paths {
                let entry = path.unwrap();
                let path = entry.path();
                if path.is_file() && path.extension().unwrap().to_str().unwrap() == "json" {
                    println!("File: {}", path.display());
                    let data = read_json(path.as_path().to_str().unwrap());
                    for (idx, case) in data.as_array().unwrap().iter().enumerate() {
                        println!("Case: {}", idx);
                        let schema = &case["schema"];
                        let root = Resolver::new(&schema, scope_of(&schema).unwrap());
                        let external = fetch_external(schema, &root).unwrap();
                        let resolvers = build_resolvers(&external);
                        let (values_, _) = collect(&schema, &resolvers).unwrap();
                        print_values(&values_);
                        assert_all_schema_nodes(&schema, &values_);
                        assert_concrete_references(&values_);
                        assert_virtual_references(&values_);
                    }
                }
            }
        }
    }

    #[test_case(
        j!({"$ref": "http://localhost:1234/subSchemas.json"}),
        &["http://localhost:1234/subSchemas.json"];
        "One remote"
    )]
    #[test_case(
        j!({"$ref": "http://localhost:1234/draft2019-09/metaschema-no-validation.json"}),
        &[
            "http://localhost:1234/draft2019-09/metaschema-no-validation.json",
            "https://json-schema.org/draft/2019-09/meta/applicator",
            "https://json-schema.org/draft/2019-09/meta/core"
        ];
        "One + two remote"
    )]
    #[test_case(
        j!({"$id": REMOTE_BASE, "defs": {"type": "string"}, "$ref": REMOTE_BASE}),
        &[];
        "Absolute reference to the root schema which is not a remote"
    )]
    // TODO. Remote without ID
    fn external_schemas(schema: Value, expected: &[&str]) {
        let root = Resolver::new(&schema, scope_of(&schema).unwrap());
        let external = fetch_external(&schema, &root).unwrap();
        assert!(
            expected
                .iter()
                .all(|key| external.contains_key(&Url::parse(key).unwrap())),
            "{:?}",
            external.keys()
        )
    }

    #[test_case(
        j!({"maximum": 1}),
        vec![
            k!(Maximum::build(1)),
        ],
        vec![];
        "Single keyword"
    )]
    #[test_case(
        j!({"properties": {"A": {"maximum": 1}}}),
        vec![
            k!(Properties::build()),
            k!(Maximum::build(1)),
        ],
        vec![
            edge(0, 1, "A"),
        ];
        "Applicator with validator"
    )]
    fn materialization(
        schema: Value,
        expected_keywords: Vec<Keyword>,
        expected_edges: Vec<RawEdge>,
    ) {
        let resolvers = HashMap::new();
        let (values, mut edges) = collect(&schema, &resolvers).unwrap();
        print_values(&values);
        let (keywords, edges) = materialize(values, &mut edges);
        assert_eq!(edges, expected_edges);
        assert_eq!(keywords, expected_keywords);
    }

    #[test_case(j!({"maximum": 5}), j!(4), j!(6))]
    #[test_case(j!({"properties": {"A": {"maximum": 5}}}), j!({"A": 4}), j!({"A": 6}))]
    fn is_valid(schema: Value, valid: Value, invalid: Value) {
        let compiled = JsonSchema::new(&schema).unwrap();
        assert!(compiled.is_valid(&valid));
        assert!(!compiled.is_valid(&invalid));
    }
}
