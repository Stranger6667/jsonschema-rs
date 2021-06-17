use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema, DEFAULT_SCOPE},
    content_encoding::{
        ContentEncodingCheckType, ContentEncodingConverterType,
        DEFAULT_CONTENT_ENCODING_CHECKS_AND_CONVERTERS,
    },
    content_media_type::{ContentMediaTypeCheckType, DEFAULT_CONTENT_MEDIA_TYPE_CHECKS},
    resolver::Resolver,
    schemas, ValidationError,
};
use ahash::AHashMap;
use std::{borrow::Cow, fmt};

const EXPECT_MESSAGE: &str = "Valid meta-schema!";

lazy_static::lazy_static! {
    static ref DRAFT4:serde_json::Value = serde_json::from_str(include_str!("../../meta_schemas/draft4.json")).expect("Valid schema!");
    static ref DRAFT6:serde_json::Value = serde_json::from_str(include_str!("../../meta_schemas/draft6.json")).expect("Valid schema!");
    static ref DRAFT7:serde_json::Value = serde_json::from_str(include_str!("../../meta_schemas/draft7.json")).expect("Valid schema!");

    static ref META_SCHEMAS: AHashMap<String, serde_json::Value> = {
        let mut store = AHashMap::with_capacity(3);
        store.insert(
            "http://json-schema.org/draft-04/schema".to_string(),
            DRAFT4.clone()
        );
        store.insert(
            "http://json-schema.org/draft-06/schema".to_string(),
            DRAFT6.clone()
        );
        store.insert(
            "http://json-schema.org/draft-07/schema".to_string(),
            DRAFT7.clone()
        );
        store
    };

    static ref META_SCHEMA_VALIDATORS: AHashMap<schemas::Draft, JSONSchema<'static>> = {
        let mut store = AHashMap::with_capacity(3);
        store.insert(
            schemas::Draft::Draft4,
            JSONSchema::options().without_schema_validation().compile(&DRAFT4).expect(EXPECT_MESSAGE)
        );
        store.insert(
            schemas::Draft::Draft6,
            JSONSchema::options().without_schema_validation().compile(&DRAFT6).expect(EXPECT_MESSAGE)
        );
        store.insert(
            schemas::Draft::Draft7,
            JSONSchema::options().without_schema_validation().compile(&DRAFT7).expect(EXPECT_MESSAGE)
        );
        store
    };
}

/// Full configuration to guide the `JSONSchema` compilation.
///
/// Using a `CompilationOptions` instance you can configure the supported draft,
/// content media types and more (check the exposed methods)
#[derive(Clone)]
pub struct CompilationOptions {
    draft: Option<schemas::Draft>,
    content_media_type_checks: AHashMap<&'static str, Option<ContentMediaTypeCheckType>>,
    content_encoding_checks_and_converters:
        AHashMap<&'static str, Option<(ContentEncodingCheckType, ContentEncodingConverterType)>>,
    store: AHashMap<String, serde_json::Value>,
    validate_schema: bool,
}

impl Default for CompilationOptions {
    fn default() -> Self {
        CompilationOptions {
            validate_schema: true,
            draft: Option::default(),
            content_media_type_checks: AHashMap::default(),
            content_encoding_checks_and_converters: AHashMap::default(),
            store: AHashMap::default(),
        }
    }
}

impl CompilationOptions {
    pub(crate) fn draft(&self) -> schemas::Draft {
        self.draft.unwrap_or_default()
    }

    /// Compile `schema` into `JSONSchema` using the currently defined options.
    pub fn compile<'a>(
        &self,
        schema: &'a serde_json::Value,
    ) -> Result<JSONSchema<'a>, ValidationError<'a>> {
        // Draft is detected in the following precedence order:
        //   - Explicitly specified;
        //   - $schema field in the document;
        //   - Draft::default()

        // Clone needed because we are going to store a Copy-on-Write (Cow) instance
        // into the final JSONSchema as well as passing `self` (the instance and not
        // the reference) would require Copy trait implementation from
        // `CompilationOptions` which is something that we would like to avoid as
        // options might contain heap-related objects (ie. an HashMap) and we want the
        // memory-related operations to be explicit
        let mut config = self.clone();
        if self.draft.is_none() {
            if let Some(draft) = schemas::draft_from_schema(schema) {
                config.with_draft(draft);
            }
        }
        let processed_config: Cow<'_, CompilationOptions> = Cow::Owned(config);
        let draft = processed_config.draft();

        let scope = match schemas::id_of(draft, schema) {
            Some(url) => url::Url::parse(url)?,
            None => DEFAULT_SCOPE.clone(),
        };
        let resolver = Resolver::new(draft, &scope, schema, self.store.clone())?;
        let context = CompilationContext::new(scope, processed_config);

        if self.validate_schema {
            if let Some(mut errors) = META_SCHEMA_VALIDATORS
                .get(&draft)
                .expect("Existing draft")
                .validate(schema)
                .err()
            {
                return Err(errors.next().expect("Should have at least one element"));
            }
        }

        let mut validators = compile_validators(schema, &context)?;
        validators.shrink_to_fit();

        Ok(JSONSchema {
            schema,
            validators,
            resolver,
            context,
        })
    }

    /// Ensure that the schema is going to be compiled using the defined Draft.
    ///
    /// ```rust
    /// # use jsonschema::{Draft, CompilationOptions};
    /// # let mut options = CompilationOptions::default();
    /// options.with_draft(Draft::Draft4);
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

    /// Ensure that compiled schema is going to support the provided content media type.
    ///
    /// Arguments:
    /// * `media_type`: Name of the content media type to support (ie. "application/json")
    /// * `media_type_check`: Method checking the validity of the input string according to
    ///     the media type.
    ///     The method should return `true` if the input is valid, `false` otherwise.
    ///
    /// Example:
    /// ```rust
    /// # use jsonschema::CompilationOptions;
    /// # let mut options = CompilationOptions::default();
    /// fn check_custom_media_type(instance_string: &str) -> bool {
    ///     // In reality the check might be a bit more different ;)
    ///     instance_string != "not good"
    /// }
    /// // Add support for application/jsonschema-test
    /// options.with_content_media_type("application/jsonschema-test", check_custom_media_type);
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

    /// Ensure that compiled schema is not supporting the provided content media type.
    ///
    /// ```rust
    /// # use jsonschema::CompilationOptions;
    /// # let mut options = CompilationOptions::default();
    /// // Disable support for application/json (which is supported by jsonschema crate)
    /// options.without_content_media_type_support("application/json");
    /// ```
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

    /// Ensure that compiled schema is going to support the provided content encoding.
    ///
    /// Arguments:
    /// * `content_encoding`: Name of the content encoding to support (ie. "base64")
    /// * `content_encoding_check`: Method checking the validity of the input string
    ///     according to content encoding.
    ///     The method should return `true` if the input is valid, `false` otherwise.
    /// * `content_encoding_converter`: Method converting the input string into a string
    ///     representation (generally output of the decoding of the content encoding).
    ///     The method should return:
    ///     * `Err(ValidationError instance)`: in case of a `jsonschema` crate supported error (obtained via `?` or `From::from` APIs)
    ///     * `Ok(None)`: if the input string is not valid according to the content encoding
    ///     * `Ok(Some(content))`: if the input string is valid according to the content encoding, `content` will contain
    ///         the string representation of the decoded input
    ///
    /// Example:
    /// ```rust
    /// # use jsonschema::{CompilationOptions, ValidationError};
    /// # let mut options = CompilationOptions::default();
    /// // The instance_string contains a number (representing the length of the string)
    /// // a space and then the string (whose length should match the expectation).
    /// // Example: "3 The" or "4  123"
    /// fn check_custom_encoding(instance_string: &str) -> bool {
    ///     if let Some(first_space_index) = instance_string.find(' ') {
    ///         if let Ok(value) = instance_string[..first_space_index].parse::<u64>() {
    ///             return instance_string[first_space_index + 1..].chars().count() == value as usize;
    ///         }
    ///     }
    ///     false
    /// }
    /// fn converter_custom_encoding(instance_string: &str) -> Result<Option<String>, ValidationError<'static>> {
    ///     if let Some(first_space_index) = instance_string.find(' ') {
    ///         if let Ok(value) = instance_string[..first_space_index].parse::<u64>() {
    ///             if instance_string[first_space_index + 1..].chars().count() == value as usize {
    ///                 return Ok(Some(instance_string[first_space_index + 1..].to_string()));
    ///             }
    ///         }
    ///     }
    ///     Ok(None)
    /// }
    /// // Add support for prefix-length-string
    /// options.with_content_encoding("prefix-length-string", check_custom_encoding, converter_custom_encoding);
    /// ```
    pub fn with_content_encoding(
        &mut self,
        content_encoding: &'static str,
        content_encoding_check: ContentEncodingCheckType,
        content_encoding_converter: ContentEncodingConverterType,
    ) -> &mut Self {
        self.content_encoding_checks_and_converters.insert(
            content_encoding,
            Some((content_encoding_check, content_encoding_converter)),
        );
        self
    }

    /// Ensure that compiled schema is not supporting the provided content encoding.
    ///
    /// ```rust
    /// # use jsonschema::CompilationOptions;
    /// # use serde_json::Value;
    /// # let mut options = CompilationOptions::default();
    /// // Disable support for base64 (which is supported by jsonschema crate)
    /// options.without_content_encoding_support("base64");
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
    pub fn with_meta_schemas(&mut self) -> &mut Self {
        self.store.extend(META_SCHEMAS.clone().into_iter());
        self
    }

    /// Add a new document to the store. It works as a cache to avoid making additional network
    /// calls to remote schemas via the `$ref` keyword.
    #[inline]
    pub fn with_document(&mut self, id: String, document: serde_json::Value) -> &mut Self {
        self.store.insert(id, document);
        self
    }

    /// Do not perform schema validation during compilation.
    /// This method is only used to disable meta-schema validation for meta-schemas itself to avoid
    /// infinite recursion.
    /// The end-user will still receive `ValidationError` that are crafted manually during
    /// compilation.
    #[inline]
    pub(crate) fn without_schema_validation(&mut self) -> &mut Self {
        self.validate_schema = false;
        self
    }
}

impl fmt::Debug for CompilationOptions {
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
    use super::CompilationOptions;
    use crate::{schemas::Draft, JSONSchema};
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(Some(Draft::Draft4), &json!({}) => Draft::Draft4)]
    #[test_case(None, &json!({"$schema": "http://json-schema.org/draft-06/schema#"}) => Draft::Draft6)]
    #[test_case(None, &json!({}) => Draft::default())]
    fn test_ensure_that_draft_detection_is_honored(
        draft_version_in_options: Option<Draft>,
        schema: &Value,
    ) -> Draft {
        let mut options = CompilationOptions::default();
        if let Some(draft_version) = draft_version_in_options {
            options.with_draft(draft_version);
        }
        let compiled = options.compile(schema).unwrap();
        compiled.context.config.draft()
    }

    #[test]
    fn test_with_document() {
        let schema = json!({"$ref": "http://example.json/schema.json#/rule"});
        let compiled = JSONSchema::options()
            .with_document(
                "http://example.json/schema.json".to_string(),
                json!({"rule": {"minLength": 5}}),
            )
            .compile(&schema)
            .unwrap();
        assert!(!compiled.is_valid(&json!("foo")));
        assert!(compiled.is_valid(&json!("foobar")));
    }
}
