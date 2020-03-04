use crate::context::CompilationContext;
use crate::error::CompilationError;
use crate::keywords::{ValidationResult, Validators};
use crate::resolver::Resolver;
use crate::schemas::{id_of, Draft};
use crate::{keywords, schemas};
use serde_json::Value;

pub(crate) const DOCUMENT_PROTOCOL: &str = "json-schema:///";

pub struct JSONSchema<'a> {
    pub(crate) draft: Draft,
    pub(crate) schema: &'a Value,
    pub(crate) validators: Validators<'a>,
    pub(crate) resolver: Resolver<'a>,
}

impl<'a> JSONSchema<'a> {
    pub fn compile(
        schema: &'a Value,
        draft: Option<Draft>,
    ) -> Result<JSONSchema<'a>, CompilationError> {
        let draft = draft.unwrap_or_else(|| {
            schemas::draft_from_schema(schema).unwrap_or(schemas::Draft::Draft7)
        });
        let base_url = match id_of(draft, schema) {
            Some(url) => url.to_string(),
            None => DOCUMENT_PROTOCOL.to_string(),
        };
        let scope = url::Url::parse(&base_url)?;
        let resolver = Resolver::new(draft, &scope, schema)?;
        let context = CompilationContext::new(scope, draft);
        let validators = compile_validators(schema, &context)?;
        Ok(JSONSchema {
            draft,
            schema,
            resolver,
            validators,
        })
    }

    pub fn validate(&self, instance: &Value) -> ValidationResult {
        for v in self.validators.iter() {
            v.validate(self, instance)?
        }
        Ok(())
    }

    pub fn is_valid(&self, instance: &Value) -> bool {
        self.validators
            .iter()
            .all(|validator| validator.is_valid(self, instance))
    }
}

pub(crate) fn compile_validators<'a>(
    schema: &'a Value,
    context: &CompilationContext,
) -> Result<Validators<'a>, CompilationError> {
    let context = context.push(schema);
    match schema {
        Value::Bool(value) => {
            let mut validators = Vec::with_capacity(1);
            if let Some(validator) = keywords::boolean::compile(*value) {
                validators.push(validator?)
            }
            Ok(validators)
        }
        Value::Object(object) => {
            if let Some(reference) = object.get("$ref") {
                if let Value::String(reference) = reference {
                    let mut validators = Vec::with_capacity(1);
                    if let Some(validator) = keywords::ref_::compile(schema, reference, &context) {
                        validators.push(validator?)
                    }
                    Ok(validators)
                } else {
                    Err(CompilationError::SchemaError)
                }
            } else {
                let mut validators = Vec::with_capacity(object.len());
                for (keyword, subschema) in object {
                    if let Some(compilation_func) = context.draft.get_validator(keyword) {
                        if let Some(validator) = compilation_func(object, subschema, &context) {
                            validators.push(validator?)
                        }
                    }
                }
                Ok(validators)
            }
        }
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::*;
    use std::borrow::Cow;
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;
    use url::Url;

    fn load(path: &str, idx: usize) -> Value {
        let path = Path::new(path);
        let mut file = File::open(&path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).ok().unwrap();
        let data: Value = from_str(&content).unwrap();
        let case = &data.as_array().unwrap()[idx];
        case.get("schema").unwrap().clone()
    }

    #[test]
    fn only_keyword() {
        // When only one keyword is specified
        let schema = json!({"type": "string"});
        let value1 = json!("AB");
        let value2 = json!(1);
        let compiled = JSONSchema::compile(&schema, None).unwrap();
        // And only this validator
        assert_eq!(compiled.validators.len(), 1);
        assert!(compiled.validate(&value1).is_ok());
        assert!(compiled.validate(&value2).is_err())
    }

    #[test]
    fn resolve_ref() {
        let schema = load("tests/suite/tests/draft7/ref.json", 4);
        let compiled = JSONSchema::compile(&schema, None).unwrap();
        let url = Url::parse("json-schema:///#/definitions/a").unwrap();
        if let (resource, Cow::Borrowed(resolved)) = compiled
            .resolver
            .resolve_fragment(Draft::Draft7, &url, &schema)
            .unwrap()
        {
            assert_eq!(resource, Url::parse("json-schema:///").unwrap());
            assert_eq!(resolved, schema.pointer("/definitions/a").unwrap());
        }
    }

    #[test]
    fn validate_ref() {
        let schema = load("tests/suite/tests/draft7/ref.json", 1);
        let value = json!({"bar": 3});
        let compiled = JSONSchema::compile(&schema, None).unwrap();
        assert!(compiled.validate(&value).is_ok());
        let value = json!({"bar": true});
        assert!(compiled.validate(&value).is_err());
    }
}
