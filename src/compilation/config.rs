use crate::{
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
}

impl fmt::Debug for CompilationConfig {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("CompilationConfig")
            .field("draft", &self.draft)
            .field(
                "content_media_type_checks",
                &self.content_media_type_checks.keys(),
            )
            .finish()
    }
}
