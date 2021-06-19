//! Schema compilation.
//! The main idea is to compile the input JSON Schema to a validators tree that will contain
//! everything needed to perform such validation in runtime.
pub(crate) mod context;
pub(crate) mod options;

use crate::{
    error::ErrorIterator, keywords, keywords::Validators, paths::InstancePath, resolver::Resolver,
    ValidationError,
};
use context::CompilationContext;
use options::CompilationOptions;
use serde_json::Value;
use url::Url;

pub(crate) const DEFAULT_ROOT_URL: &str = "json-schema:///";

/// The structure that holds a JSON Schema compiled into a validation tree
#[derive(Debug)]
pub struct JSONSchema<'a> {
    pub(crate) schema: &'a Value,
    pub(crate) validators: Validators,
    pub(crate) resolver: Resolver<'a>,
    pub(crate) context: CompilationContext<'a>,
}

lazy_static::lazy_static! {
    pub static ref DEFAULT_SCOPE: Url = url::Url::parse(DEFAULT_ROOT_URL).expect("Is a valid URL");
}

impl<'a> JSONSchema<'a> {
    /// Return a default `CompilationOptions` that can configure
    /// `JSONSchema` compilaton flow.
    ///
    /// Using options you will be able to configure the draft version
    /// to use during `JSONSchema` compilation
    ///
    /// Example of usage:
    /// ```rust
    /// # use crate::jsonschema::{Draft, JSONSchema};
    /// # let schema = serde_json::json!({});
    /// let maybe_jsonschema: Result<JSONSchema, _> = JSONSchema::options()
    ///     .with_draft(Draft::Draft7)
    ///     .compile(&schema);
    /// ```
    #[must_use]
    pub fn options() -> CompilationOptions {
        CompilationOptions::default()
    }

    /// Compile the input schema into a validation tree.
    ///
    /// The method is equivalent to `JSONSchema::options().compile(schema)`
    pub fn compile(schema: &'a Value) -> Result<JSONSchema<'a>, ValidationError<'a>> {
        Self::options().compile(schema)
    }

    /// Run validation against `instance` and return an iterator over `ValidationError` in the error case.
    #[inline]
    pub fn validate(&'a self, instance: &'a Value) -> Result<(), ErrorIterator<'a>> {
        let mut errors = self
            .validators
            .iter()
            .flat_map(move |validator| validator.validate(self, instance, &InstancePath::new()))
            .peekable();
        if errors.peek().is_none() {
            Ok(())
        } else {
            Err(Box::new(errors))
        }
    }

    /// Run validation against `instance` but return a boolean result instead of an iterator.
    /// It is useful for cases, where it is important to only know the fact if the data is valid or not.
    /// This approach is much faster, than `validate`.
    #[must_use]
    #[inline]
    pub fn is_valid(&self, instance: &Value) -> bool {
        self.validators
            .iter()
            .all(|validator| validator.is_valid(self, instance))
    }
}

/// Compile JSON schema into a tree of validators.
#[inline]
pub(crate) fn compile_validators<'a, 'c>(
    schema: &'a Value,
    context: &'c CompilationContext,
) -> Result<Validators, ValidationError<'a>> {
    let context = context.push(schema)?;
    match schema {
        Value::Bool(value) => match value {
            true => Ok(vec![]),
            false => Ok(vec![keywords::boolean::FalseValidator::compile(
                context.into_pointer(),
            )
            .expect("Should always compile")]),
        },
        Value::Object(object) => {
            if let Some(reference) = object.get("$ref") {
                if let Value::String(reference) = reference {
                    Ok(vec![keywords::ref_::compile(schema, reference, &context)
                        .expect("Should always return Some")?])
                } else {
                    Err(ValidationError::schema(schema))
                }
            } else {
                let mut validators = Vec::with_capacity(object.len());
                for (keyword, subschema) in object {
                    if let Some(compilation_func) = context.config.draft().get_validator(keyword) {
                        if let Some(validator) = compilation_func(object, subschema, &context) {
                            validators.push(validator?)
                        }
                    }
                }
                Ok(validators)
            }
        }
        _ => Err(ValidationError::schema(schema)),
    }
}

#[cfg(test)]
mod tests {
    use super::JSONSchema;
    use crate::{error::ValidationError, schemas};
    use serde_json::{from_str, json, Value};
    use std::{borrow::Cow, fs::File, io::Read, path::Path};
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
        let compiled = JSONSchema::compile(&schema).unwrap();
        let value1 = json!("AB");
        let value2 = json!(1);
        // And only this validator
        assert_eq!(compiled.validators.len(), 1);
        assert!(compiled.validate(&value1).is_ok());
        assert!(compiled.validate(&value2).is_err());
    }

    #[test]
    fn resolve_ref() {
        let schema = load("tests/suite/tests/draft7/ref.json", 4);
        let compiled = JSONSchema::compile(&schema).unwrap();
        let url = Url::parse("json-schema:///#/definitions/a").unwrap();
        if let (resource, Cow::Borrowed(resolved)) = compiled
            .resolver
            .resolve_fragment(schemas::Draft::Draft7, &url, &schema)
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
        let compiled = JSONSchema::compile(&schema).unwrap();
        assert!(compiled.validate(&value).is_ok());
        let value = json!({"bar": true});
        assert!(compiled.validate(&value).is_err());
    }

    #[test]
    fn wrong_schema_type() {
        let schema = json!([1]);
        let compiled = JSONSchema::compile(&schema);
        assert!(compiled.is_err());
    }

    #[test]
    fn multiple_errors() {
        let schema = json!({"minProperties": 2, "propertyNames": {"minLength": 3}});
        let value = json!({"a": 3});
        let compiled = JSONSchema::compile(&schema).unwrap();
        let result = compiled.validate(&value);
        let errors: Vec<ValidationError> = result.unwrap_err().collect();
        assert_eq!(errors.len(), 2);
        assert_eq!(
            errors[0].to_string(),
            r#"{"a":3} has less than 2 properties"#
        );
        assert_eq!(errors[1].to_string(), r#""a" is shorter than 3 characters"#);
    }
}
