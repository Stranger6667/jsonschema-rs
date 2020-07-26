use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema, DEFAULT_SCOPE},
    content_encoding::{
        ContentEncodingCheckType, ContentEncodingConverterType,
        DEFAULT_CONTENT_ENCODING_CHECKS_AND_CONVERTERS,
    },
    content_media_type::{ContentMediaTypeCheckType, DEFAULT_CONTENT_MEDIA_TYPE_CHECKS},
    error::CompilationError,
    resolver::Resolver,
    schemas,
};
use serde_json::Value;
use std::{borrow::Cow, collections::HashMap, fmt};

/// Full configuration to guide the `JSONSchema` compilation.
///
/// Using a `CompilationOptions` instance you can configure the supported draft,
/// content media types and more (check the exposed methods)
#[derive(Clone, Default)]
pub struct CompilationOptions {
    draft: Option<schemas::Draft>,
    content_media_type_checks: HashMap<&'static str, Option<ContentMediaTypeCheckType>>,
    content_encoding_checks_and_converters:
        HashMap<&'static str, Option<(ContentEncodingCheckType, ContentEncodingConverterType)>>,
}

impl CompilationOptions {
    pub(crate) fn draft(&self) -> schemas::Draft {
        self.draft.unwrap_or_default()
    }

    /// Compile `schema` into `JSONSchema` using the currently defined options.
    pub fn compile<'a>(&self, schema: &'a Value) -> Result<JSONSchema<'a>, CompilationError> {
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
        let resolver = Resolver::new(draft, &scope, schema)?;
        let context = CompilationContext::new(scope, processed_config);

        let mut validators = compile_validators(schema, &context)?;
        validators.shrink_to_fit();

        Ok(JSONSchema {
            schema,
            resolver,
            validators,
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
        } else if let Some(value) = DEFAULT_CONTENT_MEDIA_TYPE_CHECKS.get(media_type) {
            Some(*value)
        } else {
            None
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
        } else if let Some(value) =
            DEFAULT_CONTENT_ENCODING_CHECKS_AND_CONVERTERS.get(content_encoding)
        {
            Some(*value)
        } else {
            None
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
        let mut options = CompilationOptions::default();
        if let Some(draft_version) = draft_version_in_options {
            options.with_draft(draft_version);
        }
        let compiled = options.compile(schema).unwrap();
        compiled.context.config.draft()
    }
}
