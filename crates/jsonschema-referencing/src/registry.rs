use std::{
    collections::VecDeque,
    fmt::Debug,
    hash::{Hash, Hasher},
    sync::Arc,
};

use ahash::{AHashMap, AHashSet, AHasher};
use fluent_uri::Uri;
use once_cell::sync::Lazy;
use serde_json::Value;

use crate::{
    anchors::{AnchorKey, AnchorKeyRef},
    meta, uri, Anchor, DefaultRetriever, Draft, Error, Resolver, Resource, Retrieve,
};

type ResourceMap = AHashMap<Uri<String>, Arc<Resource>>;

pub static SPECIFICATIONS: Lazy<Registry> = Lazy::new(|| {
    let pairs = meta::META_SCHEMAS.into_iter().map(|(uri, schema)| {
        (
            uri,
            Resource::from_contents(schema.clone()).expect("Invalid resource"),
        )
    });
    // The capacity is known upfront
    let mut resources = ResourceMap::with_capacity(18);
    let mut anchors = AHashMap::with_capacity(8);
    process_resources(
        pairs,
        &DefaultRetriever,
        &mut resources,
        &mut anchors,
        Draft::default(),
    )
    .expect("Failed to process meta schemas");
    Registry { resources, anchors }
});

/// A registry of JSON Schema resources, each identified by their canonical URIs.
///
/// Registries store a collection of in-memory resources and their anchors.
/// They eagerly process all added resources, including their subresources and anchors.
/// This means that subresources contained within any added resources are immediately
/// discoverable and retrievable via their own IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registry {
    resources: ResourceMap,
    anchors: AHashMap<AnchorKey, Anchor>,
}

/// Configuration options for creating a [`Registry`].
pub struct RegistryOptions {
    retriever: Box<dyn Retrieve>,
    draft: Draft,
}

impl RegistryOptions {
    /// Create a new [`RegistryOptions`] with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            retriever: Box::new(DefaultRetriever),
            draft: Draft::default(),
        }
    }
    /// Set a custom retriever for the [`Registry`].
    #[must_use]
    pub fn retriever(mut self, retriever: Box<dyn Retrieve>) -> Self {
        self.retriever = retriever;
        self
    }
    /// Set specification version under which the resources should be interpreted under.
    #[must_use]
    pub fn draft(mut self, draft: Draft) -> Self {
        self.draft = draft;
        self
    }
    /// Create a [`Registry`] with a single resource using these options.
    ///
    /// # Errors
    ///
    /// Returns an error if the URI is invalid or if there's an issue processing the resource.
    pub fn try_new(self, uri: impl Into<String>, resource: Resource) -> Result<Registry, Error> {
        Registry::try_new_impl(uri, resource, &*self.retriever, self.draft)
    }
    /// Create a [`Registry`] from multiple resources using these options.
    ///
    /// # Errors
    ///
    /// Returns an error if any URI is invalid or if there's an issue processing the resources.
    pub fn try_from_resources(
        self,
        pairs: impl Iterator<Item = (impl Into<String>, Resource)>,
    ) -> Result<Registry, Error> {
        Registry::try_from_resources_impl(pairs, &*self.retriever, self.draft)
    }
}

impl Default for RegistryOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    /// Get [`RegistryOptions`] for configuring a new [`Registry`].
    #[must_use]
    pub fn options() -> RegistryOptions {
        RegistryOptions::new()
    }
    /// Create a new [`Registry`] with a single resource.
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI of the resource.
    /// * `resource` - The resource to add.
    ///
    /// # Errors
    ///
    /// Returns an error if the URI is invalid or if there's an issue processing the resource.
    pub fn try_new(uri: impl Into<String>, resource: Resource) -> Result<Self, Error> {
        Self::try_new_impl(uri, resource, &DefaultRetriever, Draft::default())
    }
    /// Create a new [`Registry`] from an iterator of (URI, Resource) pairs.
    ///
    /// # Arguments
    ///
    /// * `pairs` - An iterator of (URI, Resource) pairs.
    ///
    /// # Errors
    ///
    /// Returns an error if any URI is invalid or if there's an issue processing the resources.
    pub fn try_from_resources(
        pairs: impl Iterator<Item = (impl Into<String>, Resource)>,
    ) -> Result<Self, Error> {
        Self::try_from_resources_impl(pairs, &DefaultRetriever, Draft::default())
    }
    fn try_new_impl(
        uri: impl Into<String>,
        resource: Resource,
        retriever: &dyn Retrieve,
        draft: Draft,
    ) -> Result<Self, Error> {
        Self::try_from_resources_impl([(uri, resource)].into_iter(), retriever, draft)
    }
    fn try_from_resources_impl(
        pairs: impl Iterator<Item = (impl Into<String>, Resource)>,
        retriever: &dyn Retrieve,
        draft: Draft,
    ) -> Result<Self, Error> {
        let mut resources = ResourceMap::new();
        let mut anchors = AHashMap::new();
        process_resources(pairs, retriever, &mut resources, &mut anchors, draft)?;
        Ok(Registry { resources, anchors })
    }
    /// Create a new registry with a new resource.
    ///
    /// # Errors
    ///
    /// Returns an error if the URI is invalid or if there's an issue processing the resource.
    pub fn try_with_resource(
        self,
        uri: impl Into<String>,
        resource: Resource,
    ) -> Result<Registry, Error> {
        let draft = resource.draft();
        self.try_with_resources([(uri, resource)].into_iter(), draft)
    }
    /// Create a new registry with a new resource and using the given retriever.
    ///
    /// # Errors
    ///
    /// Returns an error if the URI is invalid or if there's an issue processing the resource.
    pub fn try_with_resource_and_retriever(
        self,
        uri: impl Into<String>,
        resource: Resource,
        retriever: &dyn Retrieve,
    ) -> Result<Registry, Error> {
        let draft = resource.draft();
        self.try_with_resources_and_retriever([(uri, resource)].into_iter(), retriever, draft)
    }
    /// Create a new registry with new resources.
    ///
    /// # Errors
    ///
    /// Returns an error if any URI is invalid or if there's an issue processing the resources.
    pub fn try_with_resources(
        self,
        pairs: impl Iterator<Item = (impl Into<String>, Resource)>,
        draft: Draft,
    ) -> Result<Registry, Error> {
        self.try_with_resources_and_retriever(pairs, &DefaultRetriever, draft)
    }
    /// Create a new registry with new resources and using the given retriever.
    ///
    /// # Errors
    ///
    /// Returns an error if any URI is invalid or if there's an issue processing the resources.
    pub fn try_with_resources_and_retriever(
        self,
        pairs: impl Iterator<Item = (impl Into<String>, Resource)>,
        retriever: &dyn Retrieve,
        draft: Draft,
    ) -> Result<Registry, Error> {
        let mut resources = self.resources;
        let mut anchors = self.anchors;
        process_resources(pairs, retriever, &mut resources, &mut anchors, draft)?;
        Ok(Registry { resources, anchors })
    }
    /// Create a new [`Resolver`] for this registry with the given base URI.
    ///
    /// # Errors
    ///
    /// Returns an error if the base URI is invalid.
    pub fn try_resolver(&self, base_uri: &str) -> Result<Resolver, Error> {
        let base = uri::from_str(base_uri)?;
        Ok(self.resolver(base))
    }
    /// Create a new [`Resolver`] for this registry with a known valid base URI.
    #[must_use]
    pub fn resolver(&self, base_uri: Uri<String>) -> Resolver {
        Resolver::new(self, base_uri)
    }
    #[must_use]
    pub fn resolver_from_raw_parts(
        &self,
        base_uri: Uri<String>,
        scopes: VecDeque<Uri<String>>,
    ) -> Resolver {
        Resolver::from_parts(self, base_uri, scopes)
    }
    pub(crate) fn get_or_retrieve<'r>(&'r self, uri: &Uri<String>) -> Result<&'r Resource, Error> {
        if let Some(resource) = self.resources.get(uri) {
            Ok(resource)
        } else {
            Err(Error::unretrievable(
                uri.as_str(),
                Some(
                    "Retrieving external resources is not supported once the registry is populated"
                        .into(),
                ),
            ))
        }
    }
    pub(crate) fn anchor<'a>(&self, uri: &'a Uri<String>, name: &'a str) -> Result<&Anchor, Error> {
        let key = AnchorKeyRef::new(uri, name);
        if let Some(value) = self.anchors.get(key.borrow_dyn()) {
            return Ok(value);
        }
        let resource = &self.resources[uri];
        if let Some(id) = resource.id() {
            let uri = uri::from_str(id)?;
            let key = AnchorKeyRef::new(&uri, name);
            if let Some(value) = self.anchors.get(key.borrow_dyn()) {
                return Ok(value);
            }
        }
        if name.contains('/') {
            Err(Error::invalid_anchor(name.to_string()))
        } else {
            Err(Error::no_such_anchor(name.to_string()))
        }
    }
}

fn process_resources(
    pairs: impl Iterator<Item = (impl Into<String>, Resource)>,
    retriever: &dyn Retrieve,
    resources: &mut ResourceMap,
    anchors: &mut AHashMap<AnchorKey, Anchor>,
    default_draft: Draft,
) -> Result<(), Error> {
    let mut queue = VecDeque::with_capacity(32);
    let mut seen = AHashSet::new();
    let mut external = AHashSet::new();

    // Populate the resources & queue from the input
    for (uri, resource) in pairs {
        let uri = uri::from_str(uri.into().trim_end_matches('#'))?;
        let resource = Arc::new(resource);
        resources.insert(uri.clone(), Arc::clone(&resource));
        queue.push_back((uri, resource));
    }

    loop {
        if queue.is_empty() && external.is_empty() {
            break;
        }

        // Process current queue and collect references to external resources
        while let Some((mut base, resource)) = queue.pop_front() {
            if let Some(id) = resource.id() {
                base = uri::resolve_against(&base.borrow(), id)?;
            }

            // Look for anchors
            for anchor in resource.anchors() {
                anchors.insert(
                    AnchorKey::new(base.clone(), anchor.name().to_string()),
                    anchor,
                );
            }

            // Collect references to external resources in this resource
            collect_external_references(&base, resource.contents(), &mut external, &mut seen)?;

            // Process subresources
            for subresource in resource.subresources() {
                let subresource = Arc::new(subresource?);
                // Collect references to external resources at this level
                if let Some(sub_id) = subresource.id() {
                    let base = uri::resolve_against(&base.borrow(), sub_id)?;
                    collect_external_references(
                        &base,
                        subresource.contents(),
                        &mut external,
                        &mut seen,
                    )?;
                } else {
                    collect_external_references(
                        &base,
                        subresource.contents(),
                        &mut external,
                        &mut seen,
                    )?;
                };
                queue.push_back((base.clone(), subresource));
            }
            if resource.id().is_some() {
                resources.insert(base, resource);
            }
        }
        // Retrieve external resources
        for uri in external.drain() {
            if !resources.contains_key(&uri) {
                let retrieved = retriever
                    .retrieve(&uri.borrow())
                    .map_err(|err| Error::unretrievable(uri.as_str(), Some(err)))?;
                let resource = Arc::new(Resource::from_contents_and_specification(
                    retrieved,
                    default_draft,
                )?);
                resources.insert(uri.clone(), Arc::clone(&resource));
                queue.push_back((uri, resource));
            }
        }
    }

    Ok(())
}

fn collect_external_references(
    base: &Uri<String>,
    contents: &Value,
    collected: &mut AHashSet<Uri<String>>,
    seen: &mut AHashSet<u64>,
) -> Result<(), Error> {
    if base.scheme().as_str() == "urn" {
        return Ok(());
    }
    if let Some(reference) = contents.get("$ref").and_then(Value::as_str) {
        if reference.starts_with('#') {
            // Not an external reference
            return Ok(());
        }
        let mut hasher = AHasher::default();
        (base.as_str(), reference).hash(&mut hasher);
        let hash = hasher.finish();
        if !seen.insert(hash) {
            // Reference has already been seen
            return Ok(());
        }
        let resolved = uri::resolve_against(&base.borrow(), reference)?;
        let builder = Uri::builder();
        let base_uri = match resolved.authority() {
            Some(auth) => builder
                .scheme(resolved.scheme())
                .authority(auth)
                .path(resolved.path()),
            None => builder.scheme(resolved.scheme()).path(resolved.path()),
        }
        .build()?;
        collected.insert(base_uri);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error::Error as _;

    use ahash::AHashMap;
    use fluent_uri::Uri;
    use serde_json::{json, Value};
    use test_case::test_case;

    use crate::{uri::from_str, Draft, Registry, Resource, Retrieve};

    use super::{RegistryOptions, SPECIFICATIONS};

    #[test]
    fn test_invalid_uri_on_registry_creation() {
        let schema = Draft::Draft202012.create_resource(json!({}));
        let result = Registry::try_new(":/example.com", schema);
        let error = result.expect_err("Should fail");

        assert_eq!(
            error.to_string(),
            "Invalid URI: unexpected character at index 0"
        );
        let source_error = error.source().expect("Should have a source");
        assert_eq!(source_error.to_string(), "unexpected character at index 0");
        let inner_source = source_error.source().expect("Should have a source");
        assert_eq!(inner_source.to_string(), "unexpected character at index 0");
    }

    #[test]
    fn test_lookup_unresolvable_url() {
        // Create a registry with a single resource
        let schema = Draft::Draft202012.create_resource(json!({
            "type": "object",
            "properties": {
                "foo": { "type": "string" }
            }
        }));
        let registry =
            Registry::try_new("http://example.com/schema1", schema).expect("Invalid resources");

        // Attempt to create a resolver for a URL not in the registry
        let resolver = registry
            .try_resolver("http://example.com/non_existent_schema")
            .expect("Invalid base URI");

        let result = resolver.lookup("");

        assert_eq!(
            result.unwrap_err().to_string(),
            "Resource 'http://example.com/non_existent_schema' is not present in a registry and retrieving it failed: Retrieving external resources is not supported once the registry is populated"
        );
    }

    struct TestRetriever {
        schemas: AHashMap<String, Value>,
    }

    impl TestRetriever {
        fn new(schemas: AHashMap<String, Value>) -> Self {
            TestRetriever { schemas }
        }
    }

    impl Retrieve for TestRetriever {
        fn retrieve(
            &self,
            uri: &Uri<&str>,
        ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
            if let Some(value) = self.schemas.get(uri.as_str()) {
                Ok(value.clone())
            } else {
                Err(format!("Failed to find {uri}").into())
            }
        }
    }

    fn create_test_retriever(schemas: &[(&str, Value)]) -> TestRetriever {
        TestRetriever::new(
            schemas
                .iter()
                .map(|&(k, ref v)| (k.to_string(), v.clone()))
                .collect(),
        )
    }

    struct TestCase {
        input_resources: Vec<(&'static str, Value)>,
        remote_resources: Vec<(&'static str, Value)>,
        expected_resolved_uris: Vec<&'static str>,
    }

    #[test_case(
        TestCase {
            input_resources: vec![
                ("http://example.com/schema1", json!({"$ref": "http://example.com/schema2"})),
            ],
            remote_resources: vec![
                ("http://example.com/schema2", json!({"type": "object"})),
            ],
            expected_resolved_uris: vec!["http://example.com/schema1", "http://example.com/schema2"],
        }
    ;"External ref at top")]
    #[test_case(
        TestCase {
            input_resources: vec![
                ("http://example.com/schema1", json!({
                    "$defs": {
                        "subschema": {"type": "string"}
                    },
                    "$ref": "#/$defs/subschema"
                })),
            ],
            remote_resources: vec![],
            expected_resolved_uris: vec!["http://example.com/schema1"],
        }
    ;"Internal ref at top")]
    #[test_case(
        TestCase {
            input_resources: vec![
                ("http://example.com/schema1", json!({"$ref": "http://example.com/schema2"})),
                ("http://example.com/schema2", json!({"type": "object"})),
            ],
            remote_resources: vec![],
            expected_resolved_uris: vec!["http://example.com/schema1", "http://example.com/schema2"],
        }
    ;"Ref to later resource")]
    #[test_case(
    TestCase {
            input_resources: vec![
                ("http://example.com/schema1", json!({
                    "type": "object",
                    "properties": {
                        "prop1": {"$ref": "http://example.com/schema2"}
                    }
                })),
            ],
            remote_resources: vec![
                ("http://example.com/schema2", json!({"type": "string"})),
            ],
            expected_resolved_uris: vec!["http://example.com/schema1", "http://example.com/schema2"],
        }
    ;"External ref in subresource")]
    #[test_case(
        TestCase {
            input_resources: vec![
                ("http://example.com/schema1", json!({
                    "type": "object",
                    "properties": {
                        "prop1": {"$ref": "#/$defs/subschema"}
                    },
                    "$defs": {
                        "subschema": {"type": "string"}
                    }
                })),
            ],
            remote_resources: vec![],
            expected_resolved_uris: vec!["http://example.com/schema1"],
        }
    ;"Internal ref in subresource")]
    #[test_case(
        TestCase {
            input_resources: vec![
                ("file:///schemas/main.json", json!({"$ref": "file:///schemas/external.json"})),
            ],
            remote_resources: vec![
                ("file:///schemas/external.json", json!({"type": "object"})),
            ],
            expected_resolved_uris: vec!["file:///schemas/main.json", "file:///schemas/external.json"],
        }
    ;"File scheme: external ref at top")]
    #[test_case(
        TestCase {
            input_resources: vec![
                ("file:///schemas/main.json", json!({"$ref": "subfolder/schema.json"})),
            ],
            remote_resources: vec![
                ("file:///schemas/subfolder/schema.json", json!({"type": "string"})),
            ],
            expected_resolved_uris: vec!["file:///schemas/main.json", "file:///schemas/subfolder/schema.json"],
        }
    ;"File scheme: relative path ref")]
    #[test_case(
        TestCase {
            input_resources: vec![
                ("file:///schemas/main.json", json!({
                    "type": "object",
                    "properties": {
                        "local": {"$ref": "local.json"},
                        "remote": {"$ref": "http://example.com/schema"}
                    }
                })),
            ],
            remote_resources: vec![
                ("file:///schemas/local.json", json!({"type": "string"})),
                ("http://example.com/schema", json!({"type": "number"})),
            ],
            expected_resolved_uris: vec![
                "file:///schemas/main.json",
                "file:///schemas/local.json",
                "http://example.com/schema"
            ],
        }
    ;"File scheme: mixing with http scheme")]
    #[test_case(
        TestCase {
            input_resources: vec![
                ("file:///C:/schemas/main.json", json!({"$ref": "/D:/other_schemas/schema.json"})),
            ],
            remote_resources: vec![
                ("file:///D:/other_schemas/schema.json", json!({"type": "boolean"})),
            ],
            expected_resolved_uris: vec![
                "file:///C:/schemas/main.json",
                "file:///D:/other_schemas/schema.json"
            ],
        }
    ;"File scheme: absolute path in Windows style")]
    #[test_case(
        TestCase {
            input_resources: vec![
                ("http://example.com/schema1", json!({"$ref": "http://example.com/schema2"})),
            ],
            remote_resources: vec![
                ("http://example.com/schema2", json!({"$ref": "http://example.com/schema3"})),
                ("http://example.com/schema3", json!({"$ref": "http://example.com/schema4"})),
                ("http://example.com/schema4", json!({"$ref": "http://example.com/schema5"})),
                ("http://example.com/schema5", json!({"type": "object"})),
            ],
            expected_resolved_uris: vec![
                "http://example.com/schema1",
                "http://example.com/schema2",
                "http://example.com/schema3",
                "http://example.com/schema4",
                "http://example.com/schema5",
            ],
        }
    ;"Four levels of external references")]
    #[test_case(
        TestCase {
            input_resources: vec![
                ("http://example.com/schema1", json!({"$ref": "http://example.com/schema2"})),
            ],
            remote_resources: vec![
                ("http://example.com/schema2", json!({"$ref": "http://example.com/schema3"})),
                ("http://example.com/schema3", json!({"$ref": "http://example.com/schema4"})),
                ("http://example.com/schema4", json!({"$ref": "http://example.com/schema5"})),
                ("http://example.com/schema5", json!({"$ref": "http://example.com/schema6"})),
                ("http://example.com/schema6", json!({"$ref": "http://example.com/schema1"})),
            ],
            expected_resolved_uris: vec![
                "http://example.com/schema1",
                "http://example.com/schema2",
                "http://example.com/schema3",
                "http://example.com/schema4",
                "http://example.com/schema5",
                "http://example.com/schema6",
            ],
        }
    ;"Five levels of external references with circular reference")]
    fn test_references_processing(test_case: TestCase) {
        let retriever = create_test_retriever(&test_case.remote_resources);

        let input_pairs = test_case
            .input_resources
            .clone()
            .into_iter()
            .map(|(uri, value)| {
                (
                    uri,
                    Resource::from_contents(value).expect("Invalid resource"),
                )
            });

        let registry = Registry::options()
            .retriever(Box::new(retriever))
            .try_from_resources(input_pairs)
            .expect("Invalid resources");
        // Verify that all expected URIs are resolved and present in resources
        for uri in test_case.expected_resolved_uris {
            let resolver = registry.try_resolver("").expect("Invalid base URI");
            assert!(resolver.lookup(uri).is_ok());
        }
    }

    #[test]
    fn test_default_retriever_with_remote_refs() {
        let result = Registry::try_from_resources(
            [(
                "http://example.com/schema1",
                Resource::from_contents(json!({"$ref": "http://example.com/schema2"}))
                    .expect("Invalid resource"),
            )]
            .into_iter(),
        );
        let error = result.expect_err("Should fail");
        assert_eq!(error.to_string(), "Resource 'http://example.com/schema2' is not present in a registry and retrieving it failed: Default retriever does not fetch resources");
        assert!(error.source().is_some());
    }

    #[test]
    fn test_options() {
        let _registry = RegistryOptions::default()
            .try_new("", Draft::default().create_resource(json!({})))
            .expect("Invalid resources");
    }

    #[test]
    fn test_registry_with_base_uri_fragment() {
        let input_resources = vec![
            (
                "http://example.com/schema#base",
                Draft::default().create_resource(json!({
                    "type": "object",
                    "properties": {
                        "prop": { "$ref": "other.json" }
                    }
                })),
            ),
            (
                "http://example.com/other.json",
                Draft::default().create_resource(json!({ "type": "string" })),
            ),
        ];

        let result = Registry::try_from_resources(input_resources.into_iter());
        let error = result.expect_err("Should fail");
        assert_eq!(error.to_string(), "Invalid URI: base URI/IRI with fragment");
        let source_error = error.source().expect("Should have a source");
        assert_eq!(source_error.to_string(), "base URI/IRI with fragment");
        let inner_source = source_error.source().expect("Should have a source");
        assert_eq!(inner_source.to_string(), "base URI/IRI with fragment");
    }

    #[test]
    fn test_registry_with_duplicate_input_uris() {
        let input_resources = vec![
            (
                "http://example.com/schema",
                json!({
                    "type": "object",
                    "properties": {
                        "foo": { "type": "string" }
                    }
                }),
            ),
            (
                "http://example.com/schema",
                json!({
                    "type": "object",
                    "properties": {
                        "bar": { "type": "number" }
                    }
                }),
            ),
        ];

        let result = Registry::try_from_resources(
            input_resources
                .into_iter()
                .map(|(uri, value)| (uri, Draft::Draft202012.create_resource(value))),
        );

        assert!(
            result.is_ok(),
            "Failed to create registry with duplicate input URIs"
        );
        let registry = result.unwrap();

        let resource = registry
            .resources
            .get(&from_str("http://example.com/schema").expect("Invalid URI"))
            .unwrap();
        let properties = resource
            .contents()
            .get("properties")
            .and_then(|v| v.as_object())
            .unwrap();

        assert!(
            properties.contains_key("bar"),
            "Registry should contain the last added schema"
        );
        assert!(
            !properties.contains_key("foo"),
            "Registry should not contain the overwritten schema"
        );
    }

    #[test]
    fn test_resolver_debug() {
        let registry = SPECIFICATIONS
            .clone()
            .try_with_resource(
                "http://example.com",
                Resource::from_contents(json!({})).expect("Invalid resource"),
            )
            .expect("Invalid resource");
        let resolver = registry
            .try_resolver("http://127.0.0.1/schema")
            .expect("Invalid base URI");
        assert_eq!(
            format!("{resolver:?}"),
            "Resolver { base_uri: \"http://127.0.0.1/schema\", scopes: \"[]\" }"
        );
    }

    #[test]
    fn test_try_with_resource() {
        let registry = SPECIFICATIONS
            .clone()
            .try_with_resource(
                "http://example.com",
                Resource::from_contents(json!({})).expect("Invalid resource"),
            )
            .expect("Invalid resource");
        let resolver = registry.try_resolver("").expect("Invalid base URI");
        let resolved = resolver
            .lookup("http://json-schema.org/draft-06/schema#/definitions/schemaArray")
            .expect("Lookup failed");
        assert_eq!(
            resolved.contents(),
            &json!({
                "type": "array",
                "minItems": 1,
                "items": { "$ref": "#" }
            })
        );
    }

    #[test]
    fn test_try_with_resource_and_retriever() {
        let retriever =
            create_test_retriever(&[("http://example.com/schema2", json!({"type": "object"}))]);
        let registry = SPECIFICATIONS
            .clone()
            .try_with_resource_and_retriever(
                "http://example.com",
                Resource::from_contents(json!({"$ref": "http://example.com/schema2"}))
                    .expect("Invalid resource"),
                &retriever,
            )
            .expect("Invalid resource");
        let resolver = registry.try_resolver("").expect("Invalid base URI");
        let resolved = resolver
            .lookup("http://example.com/schema2")
            .expect("Lookup failed");
        assert_eq!(resolved.contents(), &json!({"type": "object"}));
    }
}
