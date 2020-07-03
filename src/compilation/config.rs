use crate::schemas;
use serde_json::Value;

#[allow(missing_docs)]
#[derive(Clone, Debug, Default)]
pub struct CompilationConfig {
    pub(crate) draft: Option<schemas::Draft>,
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
}
