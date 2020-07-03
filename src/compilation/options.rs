use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema, DEFAULT_SCOPE},
    error::CompilationError,
    resolver::Resolver,
    schemas,
};
use serde_json::Value;
use std::{borrow::Cow, fmt};

/// Full configuration to guide the `JSONSchema` compilation.
///
/// Using a `CompilationOptions` instance you can configure the supported draft,
/// content media types and more (check the exposed methods)
#[derive(Clone, Default)]
pub struct CompilationOptions {
    draft: Option<schemas::Draft>,
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
}

impl fmt::Debug for CompilationOptions {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("CompilationConfig")
            .field("draft", &self.draft)
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
