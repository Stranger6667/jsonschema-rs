use crate::{
    compilation::{compile_validators, context::CompilationContext, Validator, DEFAULT_SCOPE},
    content_encoding::{
        ContentEncodingCheckType, ContentEncodingConverterType,
        DEFAULT_CONTENT_ENCODING_CHECKS_AND_CONVERTERS,
    },
    content_media_type::{ContentMediaTypeCheckType, DEFAULT_CONTENT_MEDIA_TYPE_CHECKS},
    keywords::{custom::KeywordFactory, format::Format},
    paths::JsonPointer,
    resolver::{DefaultResolver, Resolver, SchemaResolver},
    schemas, Keyword, ValidationError,
};
use ahash::AHashMap;
use once_cell::sync::Lazy;
use std::{borrow::Cow, fmt, sync::Arc};

macro_rules! schema {
    ($name:ident, $path:expr) => {
        static $name: Lazy<Arc<serde_json::Value>> = Lazy::new(|| {
            Arc::new(serde_json::from_slice(include_bytes!($path)).expect("Invalid schema"))
        });
    };
}

schema!(DRAFT4, "../../meta_schemas/draft4.json");
schema!(DRAFT6, "../../meta_schemas/draft6.json");
schema!(DRAFT7, "../../meta_schemas/draft7.json");
schema!(DRAFT201909, "../../meta_schemas/draft2019-09/schema.json");
schema!(
    DRAFT201909_APPLICATOR,
    "../../meta_schemas/draft2019-09/meta/applicator.json"
);
schema!(
    DRAFT201909_CONTENT,
    "../../meta_schemas/draft2019-09/meta/content.json"
);
schema!(
    DRAFT201909_CORE,
    "../../meta_schemas/draft2019-09/meta/core.json"
);
schema!(
    DRAFT201909_FORMAT,
    "../../meta_schemas/draft2019-09/meta/format.json"
);
schema!(
    DRAFT201909_META_DATA,
    "../../meta_schemas/draft2019-09/meta/meta-data.json"
);
schema!(
    DRAFT201909_VALIDATION,
    "../../meta_schemas/draft2019-09/meta/validation.json"
);
schema!(DRAFT202012, "../../meta_schemas/draft2020-12/schema.json");
schema!(
    DRAFT202012_CORE,
    "../../meta_schemas/draft2020-12/meta/core.json"
);
schema!(
    DRAFT202012_APPLICATOR,
    "../../meta_schemas/draft2020-12/meta/applicator.json"
);
schema!(
    DRAFT202012_UNEVALUATED,
    "../../meta_schemas/draft2020-12/meta/unevaluated.json"
);
schema!(
    DRAFT202012_VALIDATION,
    "../../meta_schemas/draft2020-12/meta/validation.json"
);
schema!(
    DRAFT202012_META_DATA,
    "../../meta_schemas/draft2020-12/meta/meta-data.json"
);
schema!(
    DRAFT202012_FORMAT_ANNOTATION,
    "../../meta_schemas/draft2020-12/meta/format-annotation.json"
);
schema!(
    DRAFT202012_CONTENT,
    "../../meta_schemas/draft2020-12/meta/content.json"
);

static META_SCHEMAS: Lazy<AHashMap<Cow<'static, str>, Arc<serde_json::Value>>> = Lazy::new(|| {
    let mut store = AHashMap::with_capacity(3);
    store.insert(
        "http://json-schema.org/draft-04/schema".into(),
        Arc::clone(&DRAFT4),
    );
    store.insert(
        "http://json-schema.org/draft-06/schema".into(),
        Arc::clone(&DRAFT6),
    );
    store.insert(
        "http://json-schema.org/draft-07/schema".into(),
        Arc::clone(&DRAFT7),
    );
    store.insert(
        "https://json-schema.org/draft/2019-09/schema".into(),
        Arc::clone(&DRAFT201909),
    );
    store.insert(
        "https://json-schema.org/draft/2019-09/meta/applicator".into(),
        Arc::clone(&DRAFT201909_APPLICATOR),
    );
    store.insert(
        "https://json-schema.org/draft/2019-09/meta/content".into(),
        Arc::clone(&DRAFT201909_CONTENT),
    );
    store.insert(
        "https://json-schema.org/draft/2019-09/meta/core".into(),
        Arc::clone(&DRAFT201909_CORE),
    );
    store.insert(
        "https://json-schema.org/draft/2019-09/meta/format".into(),
        Arc::clone(&DRAFT201909_FORMAT),
    );
    store.insert(
        "https://json-schema.org/draft/2019-09/meta/meta-data".into(),
        Arc::clone(&DRAFT201909_META_DATA),
    );
    store.insert(
        "https://json-schema.org/draft/2019-09/meta/validation".into(),
        Arc::clone(&DRAFT201909_VALIDATION),
    );
    store.insert(
        "https://json-schema.org/draft/2020-12/schema".into(),
        Arc::clone(&DRAFT202012),
    );
    store.insert(
        "https://json-schema.org/draft/2020-12/meta/core".into(),
        Arc::clone(&DRAFT202012_CORE),
    );
    store.insert(
        "https://json-schema.org/draft/2020-12/meta/applicator".into(),
        Arc::clone(&DRAFT202012_APPLICATOR),
    );
    store.insert(
        "https://json-schema.org/draft/2020-12/meta/unevaluated".into(),
        Arc::clone(&DRAFT202012_UNEVALUATED),
    );
    store.insert(
        "https://json-schema.org/draft/2020-12/meta/validation".into(),
        Arc::clone(&DRAFT202012_VALIDATION),
    );
    store.insert(
        "https://json-schema.org/draft/2020-12/meta/meta-data".into(),
        Arc::clone(&DRAFT202012_META_DATA),
    );
    store.insert(
        "https://json-schema.org/draft/2020-12/meta/format-annotation".into(),
        Arc::clone(&DRAFT202012_FORMAT_ANNOTATION),
    );
    store.insert(
        "https://json-schema.org/draft/2020-12/meta/content".into(),
        Arc::clone(&DRAFT202012_CONTENT),
    );
    store
});

const EXPECT_MESSAGE: &str = "Invalid meta-schema";
static META_SCHEMA_VALIDATORS: Lazy<AHashMap<schemas::Draft, Validator>> = Lazy::new(|| {
    let mut store = AHashMap::with_capacity(3);
    store.insert(
        schemas::Draft::Draft4,
        crate::options()
            .without_schema_validation()
            .build(&DRAFT4)
            .expect(EXPECT_MESSAGE),
    );
    store.insert(
        schemas::Draft::Draft6,
        crate::options()
            .without_schema_validation()
            .build(&DRAFT6)
            .expect(EXPECT_MESSAGE),
    );
    store.insert(
        schemas::Draft::Draft7,
        crate::options()
            .without_schema_validation()
            .build(&DRAFT7)
            .expect(EXPECT_MESSAGE),
    );
    let mut options = crate::options();
    options.store.insert(
        "https://json-schema.org/draft/2019-09/meta/applicator".into(),
        Arc::clone(&DRAFT201909_APPLICATOR),
    );
    options.store.insert(
        "https://json-schema.org/draft/2019-09/meta/content".into(),
        Arc::clone(&DRAFT201909_CONTENT),
    );
    options.store.insert(
        "https://json-schema.org/draft/2019-09/meta/core".into(),
        Arc::clone(&DRAFT201909_CORE),
    );
    options.store.insert(
        "https://json-schema.org/draft/2019-09/meta/format".into(),
        Arc::clone(&DRAFT201909_FORMAT),
    );
    options.store.insert(
        "https://json-schema.org/draft/2019-09/meta/meta-data".into(),
        Arc::clone(&DRAFT201909_META_DATA),
    );
    options.store.insert(
        "https://json-schema.org/draft/2019-09/meta/validation".into(),
        Arc::clone(&DRAFT201909_VALIDATION),
    );
    store.insert(
        schemas::Draft::Draft201909,
        options
            .without_schema_validation()
            .build(&DRAFT201909)
            .expect(EXPECT_MESSAGE),
    );
    let mut options = crate::options();
    options.store.insert(
        "https://json-schema.org/draft/2020-12/meta/applicator".into(),
        Arc::clone(&DRAFT202012_APPLICATOR),
    );
    options.store.insert(
        "https://json-schema.org/draft/2020-12/meta/core".into(),
        Arc::clone(&DRAFT202012_CORE),
    );
    options.store.insert(
        "https://json-schema.org/draft/2020-12/meta/applicator".into(),
        Arc::clone(&DRAFT202012_APPLICATOR),
    );
    options.store.insert(
        "https://json-schema.org/draft/2020-12/meta/unevaluated".into(),
        Arc::clone(&DRAFT202012_UNEVALUATED),
    );
    options.store.insert(
        "https://json-schema.org/draft/2020-12/meta/validation".into(),
        Arc::clone(&DRAFT202012_VALIDATION),
    );
    options.store.insert(
        "https://json-schema.org/draft/2020-12/meta/meta-data".into(),
        Arc::clone(&DRAFT202012_META_DATA),
    );
    options.store.insert(
        "https://json-schema.org/draft/2020-12/meta/format-annotation".into(),
        Arc::clone(&DRAFT202012_FORMAT_ANNOTATION),
    );
    options.store.insert(
        "https://json-schema.org/draft/2020-12/meta/content".into(),
        Arc::clone(&DRAFT202012_CONTENT),
    );
    store.insert(
        schemas::Draft::Draft202012,
        options
            .without_schema_validation()
            .build(&DRAFT202012)
            .expect(EXPECT_MESSAGE),
    );
    store
});

/// Configuration options for JSON Schema validation.
#[derive(Clone)]
pub struct ValidationOptions {
    external_resolver: Arc<dyn SchemaResolver>,
    draft: Option<schemas::Draft>,
    content_media_type_checks: AHashMap<&'static str, Option<ContentMediaTypeCheckType>>,
    content_encoding_checks_and_converters:
        AHashMap<&'static str, Option<(ContentEncodingCheckType, ContentEncodingConverterType)>>,
    store: AHashMap<Cow<'static, str>, Arc<serde_json::Value>>,
    formats: AHashMap<String, Arc<dyn Format>>,
    validate_formats: Option<bool>,
    validate_schema: bool,
    ignore_unknown_formats: bool,
    keywords: AHashMap<String, Arc<dyn KeywordFactory>>,
}

impl Default for ValidationOptions {
    fn default() -> Self {
        ValidationOptions {
            external_resolver: Arc::new(DefaultResolver),
            validate_schema: true,
            draft: Option::default(),
            content_media_type_checks: AHashMap::default(),
            content_encoding_checks_and_converters: AHashMap::default(),
            store: META_SCHEMAS.clone(),
            formats: AHashMap::default(),
            validate_formats: None,
            ignore_unknown_formats: true,
            keywords: AHashMap::default(),
        }
    }
}

impl ValidationOptions {
    /// Return the draft version, or the default if not set.
    pub(crate) fn draft(&self) -> schemas::Draft {
        self.draft.unwrap_or_default()
    }
    /// Build a JSON Schema validator using the current options.
    ///
    /// # Example
    ///
    /// ```rust
    /// use serde_json::json;
    ///
    /// let schema = json!({"type": "string"});
    /// let validator = jsonschema::options()
    ///     .build(&schema)
    ///     .expect("A valid schema");
    ///
    /// assert!(validator.is_valid(&json!("Hello")));
    /// assert!(!validator.is_valid(&json!(42)));
    /// ```
    pub fn build(&self, schema: &serde_json::Value) -> Result<Validator, ValidationError<'static>> {
        // Draft is detected in the following precedence order:
        //   - Explicitly specified;
        //   - $schema field in the document;
        //   - Draft::default()
        let mut config = self.clone();
        if self.draft.is_none() {
            if let Some(draft) = schemas::draft_from_schema(schema) {
                config.with_draft(draft);
            }
        }
        let config = Arc::new(config);

        let draft = config.draft();

        let scope = match schemas::id_of(draft, schema) {
            Some(url) => url::Url::parse(url)?,
            None => DEFAULT_SCOPE.clone(),
        };
        let schema_json = Arc::new(schema.clone());
        let resolver = Arc::new(Resolver::new(
            self.external_resolver.clone(),
            draft,
            &scope,
            schema_json,
            self.store.clone(),
        )?);
        let context = CompilationContext::new(scope.into(), Arc::clone(&config), resolver);

        if self.validate_schema {
            if let Some(mut errors) = META_SCHEMA_VALIDATORS
                .get(&draft)
                .expect("Existing draft")
                .validate(schema)
                .err()
            {
                return Err(errors
                    .next()
                    .expect("Should have at least one element")
                    .into_owned());
            }
        }

        let node = compile_validators(schema, &context).map_err(|err| err.into_owned())?;

        Ok(Validator { node, config })
    }
    /// Build a JSON Schema validator using the current options.
    ///
    /// **DEPRECATED**: Use [`ValidationOptions::build`] instead.
    #[deprecated(since = "0.20.0", note = "Use `ValidationOptions::build` instead")]
    pub fn compile<'a>(
        &self,
        schema: &'a serde_json::Value,
    ) -> Result<Validator, ValidationError<'a>> {
        self.build(schema)
    }
    /// Sets the JSON Schema draft version.
    ///
    /// ```rust
    /// use jsonschema::Draft;
    ///
    /// let options = jsonschema::options()
    ///     .with_draft(Draft::Draft4);
    /// ```
    #[inline]
    pub fn with_draft(&mut self, draft: schemas::Draft) -> &mut Self {
        self.draft = Some(draft);
        self
    }

    pub(crate) fn content_media_type_check(
        &self,
        media_type: &str,
    ) -> Option<ContentMediaTypeCheckType> {
        if let Some(value) = self.content_media_type_checks.get(media_type) {
            *value
        } else {
            DEFAULT_CONTENT_MEDIA_TYPE_CHECKS.get(media_type).copied()
        }
    }

    /// Add support for a custom content media type validation.
    ///
    /// # Example
    ///
    /// ```rust
    /// fn check_custom_media_type(instance_string: &str) -> bool {
    ///     instance_string.starts_with("custom:")
    /// }
    ///
    /// let options = jsonschema::options()
    ///     .with_content_media_type("application/custom", check_custom_media_type);
    /// ```
    pub fn with_content_media_type(
        &mut self,
        media_type: &'static str,
        media_type_check: ContentMediaTypeCheckType,
    ) -> &mut Self {
        self.content_media_type_checks
            .insert(media_type, Some(media_type_check));
        self
    }

    /// Set a custom resolver for external references.
    pub fn with_resolver(&mut self, resolver: impl SchemaResolver + 'static) -> &mut Self {
        self.external_resolver = Arc::new(resolver);
        self
    }

    /// Remove support for a specific content media type validation.
    pub fn without_content_media_type_support(&mut self, media_type: &'static str) -> &mut Self {
        self.content_media_type_checks.insert(media_type, None);
        self
    }

    #[inline]
    fn content_encoding_check_and_converter(
        &self,
        content_encoding: &str,
    ) -> Option<(ContentEncodingCheckType, ContentEncodingConverterType)> {
        if let Some(value) = self
            .content_encoding_checks_and_converters
            .get(content_encoding)
        {
            *value
        } else {
            DEFAULT_CONTENT_ENCODING_CHECKS_AND_CONVERTERS
                .get(content_encoding)
                .copied()
        }
    }

    pub(crate) fn content_encoding_check(
        &self,
        content_encoding: &str,
    ) -> Option<ContentEncodingCheckType> {
        if let Some((check, _)) = self.content_encoding_check_and_converter(content_encoding) {
            Some(check)
        } else {
            None
        }
    }

    pub(crate) fn content_encoding_convert(
        &self,
        content_encoding: &str,
    ) -> Option<ContentEncodingConverterType> {
        if let Some((_, converter)) = self.content_encoding_check_and_converter(content_encoding) {
            Some(converter)
        } else {
            None
        }
    }
    /// Add support for a custom content encoding.
    ///
    /// # Arguments
    ///
    /// * `encoding`: Name of the content encoding (e.g., "base64")
    /// * `check`: Validates the input string (return `true` if valid)
    /// * `converter`: Converts the input string, returning:
    ///   - `Err(ValidationError)`: For supported errors
    ///   - `Ok(None)`: If input is invalid
    ///   - `Ok(Some(content))`: If valid, with decoded content
    ///
    /// # Example
    ///
    /// ```rust
    /// use jsonschema::ValidationError;
    ///
    /// fn check(s: &str) -> bool {
    ///     s.starts_with("valid:")
    /// }
    ///
    /// fn convert(s: &str) -> Result<Option<String>, ValidationError<'static>> {
    ///     if s.starts_with("valid:") {
    ///         Ok(Some(s[6..].to_string()))
    ///     } else {
    ///         Ok(None)
    ///     }
    /// }
    ///
    /// let options = jsonschema::options()
    ///     .with_content_encoding("custom", check, convert);
    /// ```
    pub fn with_content_encoding(
        &mut self,
        encoding: &'static str,
        check: ContentEncodingCheckType,
        converter: ContentEncodingConverterType,
    ) -> &mut Self {
        self.content_encoding_checks_and_converters
            .insert(encoding, Some((check, converter)));
        self
    }
    /// Remove support for a specific content encoding.
    ///
    /// # Example
    ///
    /// ```rust
    /// let options = jsonschema::options()
    ///     .without_content_encoding_support("base64");
    /// ```
    pub fn without_content_encoding_support(
        &mut self,
        content_encoding: &'static str,
    ) -> &mut Self {
        self.content_encoding_checks_and_converters
            .insert(content_encoding, None);
        self
    }
    /// Add meta schemas for supported JSON Schema drafts.
    /// It is helpful if your schema has references to JSON Schema meta-schemas:
    ///
    /// ```json
    /// {
    ///     "schema": {
    ///         "multipleOf": {
    ///             "$ref": "http://json-schema.org/draft-04/schema#/properties/multipleOf"
    ///         },
    ///         "maximum": {
    ///             "$ref": "http://json-schema.org/draft-04/schema#/properties/maximum"
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// The example above is taken from the Swagger 2.0 JSON schema.
    #[inline]
    #[deprecated(since = "0.19.0", note = "Meta schemas are now included by default")]
    pub fn with_meta_schemas(&mut self) -> &mut Self {
        self
    }
    /// Add a document to the store.
    ///
    /// Acts as a cache to avoid network calls for remote schemas referenced by `$ref`.
    #[inline]
    pub fn with_document(&mut self, id: String, document: serde_json::Value) -> &mut Self {
        self.store.insert(id.into(), Arc::new(document));
        self
    }
    /// Register a custom format validator.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use serde_json::json;
    /// fn my_format(s: &str) -> bool {
    ///    // Your awesome format check!
    ///    s.ends_with("42!")
    /// }
    /// # fn foo() {
    /// let schema = json!({"type": "string", "format": "custom"});
    /// let validator = jsonschema::options()
    ///     .with_format("custom", my_format)
    ///     .build(&schema)
    ///     .expect("Valid schema");
    ///
    /// assert!(!validator.is_valid(&json!("foo")));
    /// assert!(validator.is_valid(&json!("foo42!")));
    /// # }
    /// ```
    pub fn with_format<N, F>(&mut self, name: N, format: F) -> &mut Self
    where
        N: Into<String>,
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.formats.insert(name.into(), Arc::new(format));
        self
    }
    pub(crate) fn get_format(&self, format: &str) -> Option<(&String, &Arc<dyn Format>)> {
        self.formats.get_key_value(format)
    }
    /// Disable schema validation during compilation.
    ///
    /// Used internally to prevent infinite recursion when validating meta-schemas.
    /// **Note**: Manually-crafted `ValidationError`s may still occur during compilation.
    #[inline]
    pub(crate) fn without_schema_validation(&mut self) -> &mut Self {
        self.validate_schema = false;
        self
    }
    /// Set whether to validate formats.
    ///
    /// Default behavior depends on the draft version. This method overrides
    /// the default, enabling or disabling format validation regardless of draft.
    #[inline]
    pub fn should_validate_formats(&mut self, yes: bool) -> &mut Self {
        self.validate_formats = Some(yes);
        self
    }
    pub(crate) fn validate_formats(&self) -> bool {
        self.validate_formats
            .unwrap_or_else(|| self.draft().validate_formats_by_default())
    }
    /// Set whether to ignore unknown formats.
    ///
    /// By default, unknown formats are silently ignored. Set to `false` to report
    /// unrecognized formats as validation errors.
    pub fn should_ignore_unknown_formats(&mut self, yes: bool) -> &mut Self {
        self.ignore_unknown_formats = yes;
        self
    }

    pub(crate) const fn are_unknown_formats_ignored(&self) -> bool {
        self.ignore_unknown_formats
    }
    /// Register a custom keyword validator.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use jsonschema::{
    /// #    paths::{JsonPointer, JsonPointerNode},
    /// #    ErrorIterator, Keyword, ValidationError,
    /// # };
    /// # use serde_json::{json, Map, Value};
    /// # use std::iter::once;
    ///
    /// struct MyCustomValidator;
    ///
    /// impl Keyword for MyCustomValidator {
    ///     fn validate<'instance>(
    ///         &self,
    ///         instance: &'instance Value,
    ///         instance_path: &JsonPointerNode,
    ///     ) -> ErrorIterator<'instance> {
    ///         // ... validate instance ...
    ///         if !instance.is_object() {
    ///             let error = ValidationError::custom(
    ///                 JsonPointer::default(),
    ///                 instance_path.into(),
    ///                 instance,
    ///                 "Boom!",
    ///             );
    ///             Box::new(once(error))
    ///         } else {
    ///             Box::new(None.into_iter())
    ///         }
    ///     }
    ///     fn is_valid(&self, instance: &Value) -> bool {
    ///         // ... determine if instance is valid ...
    ///         true
    ///     }
    /// }
    ///
    /// // You can create a factory function, or use a closure to create new validator instances.
    /// fn custom_validator_factory<'a>(
    ///     parent: &'a Map<String, Value>,
    ///     value: &'a Value,
    ///     path: JsonPointer,
    /// ) -> Result<Box<dyn Keyword>, ValidationError<'a>> {
    ///     Ok(Box::new(MyCustomValidator))
    /// }
    ///
    /// let validator = jsonschema::options()
    ///     .with_keyword("my-type", custom_validator_factory)
    ///     .with_keyword("my-type-with-closure", |_, _, _| Ok(Box::new(MyCustomValidator)))
    ///     .build(&json!({ "my-type": "my-schema"}))
    ///     .expect("A valid schema");
    ///
    /// assert!(validator.is_valid(&json!({ "a": "b"})));
    /// ```
    pub fn with_keyword<N, F>(&mut self, name: N, factory: F) -> &mut Self
    where
        N: Into<String>,
        F: for<'a> Fn(
                &'a serde_json::Map<String, serde_json::Value>,
                &'a serde_json::Value,
                JsonPointer,
            ) -> Result<Box<dyn Keyword>, ValidationError<'a>>
            + Send
            + Sync
            + 'static,
    {
        self.keywords.insert(name.into(), Arc::new(factory));
        self
    }

    pub(crate) fn get_keyword_factory(&self, name: &str) -> Option<&Arc<dyn KeywordFactory>> {
        self.keywords.get(name)
    }
}

impl fmt::Debug for ValidationOptions {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("CompilationConfig")
            .field("draft", &self.draft)
            .field("content_media_type", &self.content_media_type_checks.keys())
            .field(
                "content_encoding",
                &self.content_encoding_checks_and_converters.keys(),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::ValidationOptions;
    use crate::schemas::Draft;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(Some(Draft::Draft4), &json!({}) => Draft::Draft4)]
    #[test_case(None, &json!({"$schema": "http://json-schema.org/draft-06/schema#"}) => Draft::Draft6)]
    #[test_case(None, &json!({}) => Draft::default())]
    fn test_ensure_that_draft_detection_is_honored(
        draft_version_in_options: Option<Draft>,
        schema: &Value,
    ) -> Draft {
        let mut options = ValidationOptions::default();
        if let Some(draft_version) = draft_version_in_options {
            options.with_draft(draft_version);
        }
        let validator = options.build(schema).unwrap();
        validator.draft()
    }

    #[test]
    fn test_with_document() {
        let schema = json!({"$ref": "http://example.json/schema.json#/rule"});
        let validator = crate::options()
            .with_document(
                "http://example.json/schema.json".to_string(),
                json!({"rule": {"minLength": 5}}),
            )
            .build(&schema)
            .expect("Valid schema");
        assert!(!validator.is_valid(&json!("foo")));
        assert!(validator.is_valid(&json!("foobar")));
    }

    fn custom(s: &str) -> bool {
        s.ends_with("42!")
    }

    #[test]
    fn custom_format() {
        let schema = json!({"type": "string", "format": "custom"});
        let validator = crate::options()
            .with_format("custom", custom)
            .build(&schema)
            .expect("Valid schema");
        assert!(!validator.is_valid(&json!("foo")));
        assert!(validator.is_valid(&json!("foo42!")));
    }
}
