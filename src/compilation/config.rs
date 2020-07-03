use crate::{
    content_encoding::{
        ContentTypeCheckType, ContentTypeConverterType, DEFAULT_CONTENT_ENCODING_CHECKS,
        DEFAULT_CONTENT_ENCODING_CONVERTERS,
    },
    content_media_type::{ContentMediaTypeCheckType, DEFAULT_CONTENT_MEDIA_TYPE_CHECKS},
    schemas,
};
use serde_json::Value;
use std::{collections::HashMap, fmt};

#[allow(missing_docs)]
#[derive(Clone, Default)]
pub struct CompilationConfig {
    pub(crate) draft: Option<schemas::Draft>,
    content_media_type_checks: HashMap<String, Option<ContentMediaTypeCheckType>>,
    content_encoding_checks: HashMap<String, Option<ContentTypeCheckType>>,
    content_encoding_converters: HashMap<String, Option<ContentTypeConverterType>>,
}

#[allow(missing_docs)]
impl CompilationConfig {
    pub(crate) fn draft(&self) -> schemas::Draft {
        self.draft
            .expect("JSONSchema::compile should have defined a specific draft version.")
    }

    pub(crate) fn set_draft_if_missing(&mut self, schema: &Value) -> &mut Self {
        if self.draft.is_none() {
            self.draft = Some(schemas::draft_from_schema(schema).unwrap_or(schemas::Draft::Draft7));
        }
        self
    }

    pub fn set_draft(&mut self, draft: schemas::Draft) -> &mut Self {
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

    pub fn add_content_media_type_check<IS: Into<String>>(
        &mut self,
        media_type: IS,
        media_type_check: Option<ContentMediaTypeCheckType>,
    ) -> &mut Self {
        self.content_media_type_checks
            .insert(media_type.into(), media_type_check);
        self
    }

    pub(crate) fn content_encoding_check(
        &self,
        content_encoding: &str,
    ) -> Option<ContentTypeCheckType> {
        if let Some(value) = self.content_encoding_checks.get(content_encoding) {
            *value
        } else if let Some(value) = DEFAULT_CONTENT_ENCODING_CHECKS.get(content_encoding) {
            Some(*value)
        } else {
            None
        }
    }

    pub fn add_content_encoding_check<IS: Into<String>>(
        &mut self,
        content_encoding: IS,
        content_encoding_check: Option<ContentTypeCheckType>,
    ) -> &mut Self {
        self.content_encoding_checks
            .insert(content_encoding.into(), content_encoding_check);
        self
    }

    pub(crate) fn content_encoding_convert(
        &self,
        content_encoding: &str,
    ) -> Option<ContentTypeConverterType> {
        if let Some(value) = self.content_encoding_converters.get(content_encoding) {
            *value
        } else if let Some(value) = DEFAULT_CONTENT_ENCODING_CONVERTERS.get(content_encoding) {
            Some(*value)
        } else {
            None
        }
    }

    pub fn add_content_encoding_convert<IS: Into<String>>(
        &mut self,
        content_encoding: IS,
        content_encoding_convert: Option<ContentTypeConverterType>,
    ) -> &mut Self {
        self.content_encoding_converters
            .insert(content_encoding.into(), content_encoding_convert);
        self
    }
}

impl fmt::Debug for CompilationConfig {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("CompilationConfig")
            .field("draft", &self.draft)
            .field(
                "content_media_type_checks",
                &self.content_media_type_checks.keys(),
            )
            .field(
                "content_encoding_checks",
                &self.content_encoding_checks.keys(),
            )
            .field(
                "content_encoding_converts",
                &self.content_encoding_converters.keys(),
            )
            .finish()
    }
}
