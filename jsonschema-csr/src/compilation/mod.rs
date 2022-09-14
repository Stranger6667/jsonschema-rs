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

mod collection;
pub(crate) mod edges;
mod error;
pub mod resolver;

use error::Result;

use crate::{
    compilation::{
        edges::EdgeLabel,
        resolver::{parse_reference, Reference},
    },
    vocabularies::{applicator, validation, KeywordName},
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
        let mut values = materialize(values, &mut edges);
        minimize(&mut values, &mut edges);
        let (head_end, keywords, edges) = build(values, &mut edges);
        // And finally, compress the edges into the CSR format
        let (offsets, edges) = compress(edges);
        Ok(JsonSchema {
            keywords: keywords.into_boxed_slice(),
            head_end,
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
    ($stack: expr, $scope: expr, $idx: expr, $value: expr, $label: expr, $seen: expr, $resolver: expr, $folders: expr) => {
        // Push the new value only if it wasn't seen yet.
        // Insert it immediately to avoid pushing references to the same target from multiple places
        if $seen.insert($value) {
            $stack.push((
                $scope,
                Some(($idx, crate::compilation::edges::label($label))),
                $folders.clone(),
                $resolver,
                $value,
            ));
        }
    };
}

macro_rules! push_schema {
    ($stack: expr, $idx: expr, $value: expr, $label: expr, $seen: expr, $resolver: expr, $folders: expr) => {
        push!(
            $stack,
            crate::compilation::Scope::Schema,
            $idx,
            $value,
            $label,
            $seen,
            $resolver,
            $folders
        )
    };
}

macro_rules! push_not_schema {
    ($stack: expr, $idx: expr, $value: expr, $label: expr, $seen: expr, $resolver: expr, $folders: expr) => {
        push!(
            $stack,
            crate::compilation::Scope::NotSchema,
            $idx,
            $value,
            $label,
            $seen,
            $resolver,
            $folders
        )
    };
}

enum Scope {
    Schema,
    NotSchema,
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
    // todo - reuse resolver
    let resolver = Resolver::new(schema, scope_of(schema)?);
    let mut stack = vec![(Scope::Schema, None, vec![], &resolver, schema)];
    let mut seen = SeenValues::new();
    while let Some((scope, parent, mut folders, mut resolver, node)) = stack.pop() {
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
                    match key.as_str() {
                        "$ref" => {
                            if let Value::String(ref_string) = value {
                                match parse_reference(ref_string)? {
                                    Reference::Absolute(location) => {
                                        let resolved = if let Some(resolver) =
                                            resolvers.get(location.as_str())
                                        {
                                            let (folders, resolved) =
                                                resolver.resolve(ref_string)?;
                                            push_schema!(
                                                stack, node_idx, resolved, REF, seen, resolver,
                                                folders
                                            );
                                            resolved
                                        } else {
                                            let (_, resolved) = resolver.resolve(ref_string)?;
                                            resolved
                                        };
                                        values.add_virtual(resolved);
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
                                        // Push the resolved value onto the stack to explore them further
                                        push_schema!(
                                            stack, node_idx, resolved, REF, seen, resolver, folders
                                        );
                                    }
                                };
                                edges.push(RawEdge::new(node_idx, values.len() - 1, REF.into()));
                            } else {
                                // The `$ref` value is not a string - explore it further
                                push_schema!(stack, node_idx, value, REF, seen, resolver, folders);
                            };
                        }
                        "maximum" => {
                            push_not_schema!(
                                stack,
                                node_idx,
                                value,
                                KeywordName::Maximum,
                                seen,
                                resolver,
                                folders
                            );
                        }
                        "properties" => {
                            push_not_schema!(
                                stack,
                                node_idx,
                                value,
                                KeywordName::Properties,
                                seen,
                                resolver,
                                folders
                            );
                        }
                        unknown => {
                            println!("Unknown keyword: {}", unknown)
                        }
                    }
                }
            }
            Value::Array(items) => {
                for (idx, item) in items.iter().enumerate() {
                    push_schema!(stack, node_idx, item, idx, seen, resolver, folders);
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

/// Make all virtual edges point to concrete values.
fn materialize<'a>(values: Vec<ValueReference<'a>>, edges: &mut Vec<RawEdge>) -> Vec<&'a Value> {
    let mut concrete = vec![];
    for edge in edges.iter_mut() {
        if let ValueReference::Virtual(reference) = &values[edge.target] {
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
    // Build a new vector of value by pushing only concrete values
    // When a virtual one occurs, then all further concrete nodes will have a smaller index
    let mut should_shift = false;
    for value in &values {
        match value {
            ValueReference::Concrete(reference) => {
                concrete.push(*reference);
                if should_shift {
                    for edge in &mut *edges {
                        if edge.source >= concrete.len() {
                            edge.source -= 1;
                        }
                        if edge.target >= concrete.len() {
                            edge.target -= 1;
                        }
                    }
                    should_shift = false;
                }
            }
            ValueReference::Virtual(_) => {
                should_shift = true;
            }
        }
    }
    // TODO. Is it possible to avoid deduplication?
    edges.sort_unstable_by_key(|edge| (edge.source, edge.target));
    edges.dedup();
    concrete
}

/// Remove not needed values & edges.
fn minimize(values: &mut Vec<&Value>, edges: &mut Vec<RawEdge>) {}

/// Build a keyword graph.
fn build(values: Vec<&Value>, edges: &mut Vec<RawEdge>) -> (usize, Vec<Keyword>, Vec<RawEdge>) {
    // TODO: edges should also be new - input ones refer to the original graph
    // TODO: Some edges are useless - they don't represent any useful keyword
    let mut new_edges = vec![];
    let mut keywords = Vec::with_capacity(values.len());
    edges.sort_unstable_by_key(|edge| edge.source);
    let mut head_end = 0_usize;
    for edge in edges {
        match &edge.label {
            EdgeLabel::Keyword(KeywordName::Maximum) => {
                if edge.source == 0 {
                    head_end += 1;
                }
                let value = values[edge.target];
                keywords.push(validation::Maximum::build(value.as_u64().unwrap()))
            }
            EdgeLabel::Keyword(KeywordName::Properties) => {
                if edge.source == 0 {
                    head_end += 1;
                }
                keywords.push(applicator::Properties::build())
            }
            _ => {}
        }
    }
    (head_end, keywords, new_edges)
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
        &[j!(true)],
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
        &[edge(0, 1, KeywordName::Maximum)],
        &[j!({"maximum": 5}), j!(5)],
        &[edge(0, 1, KeywordName::Maximum)],
        &[];
        "No reference"
    )]
    #[test_case(
        j!({"properties": {"maximum": true}}),
        &[
            c!({"properties": {"maximum": true}}),
            c!({"maximum": true}),
            c!(true),
        ],
        &[
            edge(0, 1, KeywordName::Properties),
            edge(1, 2, "maximum"),
        ],
        &[j!({"maximum": 5}), j!(5)],
        &[edge(0, 1, KeywordName::Maximum)],
        &[];
        "Not a keyword"
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
        &[
            j!({"properties":{"$ref":{"type":"string"}}}),
            j!({"$ref":{"type":"string"}}),
            j!({"type":"string"}),
            j!("string"),
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
        ],
        &[
            edge(0, 1, "$ref"),
        ],
        &[
            j!({"$ref": SELF_REF}),
        ],
        &[
            edge(0, 0, "$ref"),
        ],
        &[];
        "Self reference"
    )]
    #[test_case(
        j!({"$ref": REMOTE_REF}),
        &[
            c!({"$ref": REMOTE_REF}),
            v!({"type": "integer"}),
            c!({"type": "integer"}),
            c!("integer"),
        ],
        &[
            edge(0, 1, "$ref"),
            edge(0, 2, "$ref"),
            edge(2, 3, "type")
        ],
        &[
            j!({"$ref": REMOTE_REF}),
            j!({"type": "integer"}),
            j!("integer"),
        ],
        &[
            edge(0, 1, "$ref"),
            edge(1, 2, "type")
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
            c!({"type": "integer"}),
            c!("integer"),
            c!("baseUriChange/"),
            c!("http://localhost:1234/"),
        ],
        &[
            edge(0, 1, "items"),
            edge(2, 3, "$ref"),
            edge(1, 2, "items"),
            edge(2, 4, "$ref"),
            edge(4, 5, "type"),
            edge(1, 6, "$id"),
            edge(0, 7, "$id"),
        ],
        &[
            j!({
                "$id": "http://localhost:1234/",
                "items": {
                    "$id": "baseUriChange/",
                    "items": {"$ref": "folderInteger.json"}
                }
            }),
            j!({
                "$id": "baseUriChange/",
                "items": {"$ref": "folderInteger.json"}
            }),
            j!({"$ref": "folderInteger.json"}),
            j!({"type": "integer"}),
            j!("integer"),
            j!("baseUriChange/"),
            j!("http://localhost:1234/"),
        ],
        &[
            edge(0, 1, "items"),
            edge(0, 6, "$id"),
            edge(1, 2, "items"),
            edge(1, 5, "$id"),
            edge(2, 3, "$ref"),
            edge(3, 4, "type"),
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
            c!({"$id":"baseUriChangeFolder/"}),
            c!("baseUriChangeFolder/"),
            c!("http://localhost:1234/scope_change_defs1.json"),
        ],
        &[
            edge(0, 1, "properties"),
            edge(2, 3, "$ref"),
            edge(1, 2, "list"),
            edge(0, 4, "definitions"),
            edge(4, 5, "$id"),
            edge(0, 6, "$id"),
        ],
        &[
            j!({
                "$id": "http://localhost:1234/scope_change_defs1.json",
                "properties": {
                    "list": {"$ref": "#/definitions"}
                },
                "definitions": {
                    "$id": "baseUriChangeFolder/",
                }
            }),
            j!({"list":{"$ref":"#/definitions"}}),
            j!({"$ref":"#/definitions"}),
            j!({"$id":"baseUriChangeFolder/"}),
            j!("baseUriChangeFolder/"),
            j!("http://localhost:1234/scope_change_defs1.json"),
        ],
        &[
            edge(0, 1, "properties"),
            edge(0, 3, "definitions"),
            edge(0, 5, "$id"),
            edge(1, 2, "list"),
            edge(2, 3, "$ref"),
            edge(3, 4, "$id"),
        ],
        &[];
        "Base URI change - change folder"
    )]
    #[test_case(
        j!({
            "$ref": "http://localhost:1234/subSchemas.json#/refToInteger"
        }),
        &[
            c!({"$ref": "http://localhost:1234/subSchemas.json#/refToInteger"}),
            v!({"$ref":"#/integer"}),
            c!({"$ref":"#/integer"}),
            v!({"type":"integer"}),
            c!({"type":"integer"}),
            c!("integer"),
        ],
        &[
            edge(0, 1, "$ref"),
            edge(2, 3, "$ref"),
            edge(0, 2, "$ref"),
            edge(2, 4, "$ref"),
            edge(4, 5, "type"),
        ],
        &[
            j!({"$ref": "http://localhost:1234/subSchemas.json#/refToInteger"}),
            j!({"$ref":"#/integer"}),
            j!({"type":"integer"}),
            j!("integer"),
        ],
        &[
            edge(0, 1, "$ref"),
            edge(1, 2, "$ref"),
            edge(2, 3, "type"),
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
        ],
        &[
            edge(0, 1, "properties"),
            edge(2, 3, "$ref"),
            edge(1, 2, "A"),
            edge(0, 4, "$id"),
        ],
        &[
            j!({
                "$id": "http://localhost:1234/root",
                "properties": {
                    "A": {
                        "$ref": "http://localhost:1234/root"
                    }
                }
            }),
            j!({
                "A": {
                    "$ref": "http://localhost:1234/root"
                }
            }),
            j!({"$ref": "http://localhost:1234/root"}),
            j!("http://localhost:1234/root"),
        ],
        &[
            edge(0, 1, "properties"),
            edge(0, 3, "$id"),
            edge(1, 2, "A"),
            edge(2, 0, "$ref"),
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
            c!({"$ref":"#/allOf/1"}),
            v!({"$ref":"#/allOf/0"}),
        ],
        &[
            edge(0, 1, "allOf"),
            edge(2, 3, "$ref"),
            edge(1, 2, 1),
            edge(4, 5, "$ref"),
            edge(1, 4, 0),
        ],
        &[
            j!({"allOf":[{"$ref":"#/allOf/1"},{"$ref":"#/allOf/0"}]}),
            j!([{"$ref":"#/allOf/1"},{"$ref":"#/allOf/0"}]),
            j!({"$ref":"#/allOf/0"}),
            j!({"$ref":"#/allOf/1"}),
        ],
        &[
            edge(0, 1, "allOf"),
            edge(1, 2, 1),
            edge(1, 3, 0),
            edge(2, 3, "$ref"),
            edge(3, 2, "$ref"),
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
            c!({"$id":"http://localhost:1234/node","s":{"$ref":"tree"}}),
            c!({"$ref":"tree"}),
            v!({"$id":"http://localhost:1234/tree","d":{"$id":"http://localhost:1234/node","s":{"$ref":"tree"}},"n":{"$ref":"node"}}),
            c!("http://localhost:1234/node"),
            c!("http://localhost:1234/tree"),
        ],
        &[
            edge(1, 2, "$ref"),
            edge(0, 1, "n"),
            edge(0, 3, "d"),
            edge(4, 5, "$ref"),
            edge(3, 4, "s"),
            edge(3, 6, "$id"),
            edge(0, 7, "$id"),
        ],
        &[
            j!({"$id":"http://localhost:1234/tree","d":{"$id":"http://localhost:1234/node","s":{"$ref":"tree"}},"n":{"$ref":"node"}}),
            j!({"$ref":"node"}),
            j!({"$id":"http://localhost:1234/node","s":{"$ref":"tree"}}),
            j!({"$ref":"tree"}),
            j!("http://localhost:1234/node"),
            j!("http://localhost:1234/tree"),
        ],
        &[
            edge(0, 1, "n"),
            edge(0, 2, "d"),
            edge(0, 5, "$id"),
            edge(1, 2, "$ref"),
            edge(2, 3, "s"),
            edge(2, 4, "$id"),
            edge(3, 0, "$ref"),
        ],
        &[];
        "Recursive references between schemas"
    )]
    fn values_and_edges(
        schema: Value,
        values: &[ValueReference],
        edges: &[RawEdge],
        concrete_values: &[Value],
        concrete_edges: &[RawEdge],
        keys: &[&str],
    ) {
        let root = Resolver::new(&schema, scope_of(&schema).unwrap());
        let external = fetch_external(&schema, &root).unwrap();
        let resolvers = build_resolvers(&external);
        let (values_, mut edges_) = collect(&schema, &resolvers).unwrap();
        print_values(&values_);
        assert_eq!(values_, values);
        assert_concrete_references(&values_);
        assert_virtual_references(&values_);
        assert_eq!(edges_, edges);
        assert_eq!(resolvers.keys().cloned().collect::<Vec<&str>>(), keys);
        let mut values = materialize(values_, &mut edges_);
        minimize(&mut values, &mut edges_);
        assert_eq!(
            values.into_iter().cloned().collect::<Vec<Value>>(),
            concrete_values
        );
        assert_eq!(edges_, concrete_edges);
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

    #[test_case(j!({"maximum": 5}), j!(4), j!(6))]
    #[test_case(j!({"properties": {"A": {"maximum": 5}}}), j!({"A": 4}), j!({"A": 6}))]
    fn is_valid(schema: Value, valid: Value, invalid: Value) {
        let compiled = JsonSchema::new(&schema).unwrap();
        assert!(compiled.is_valid(&valid));
        assert!(!compiled.is_valid(&invalid));
    }
}
