//! Schema compilation.
//! The main idea is to compile the input JSON Schema to a validators tree that will contain
//! everything needed to perform such validation in runtime.
pub(crate) mod context;
pub(crate) mod options;

use crate::{
    error::ErrorIterator,
    keywords,
    output::Output,
    paths::{InstancePath, JSONPointer},
    primitive_type::{PrimitiveType, PrimitiveTypesBitMap},
    schema_node::SchemaNode,
    validator::Validate,
    Draft, ValidationError,
    compilation::options::KeywordDefinition,
    keywords::custom_keyword_schema::compile_custom_keyword_validator
};
use ahash::AHashMap;
use context::CompilationContext;
use options::CompilationOptions;
use serde_json::{json, Value};
use std::sync::Arc;
use url::Url;

pub(crate) const DEFAULT_ROOT_URL: &str = "json-schema:///";

/// The structure that holds a JSON Schema compiled into a validation tree
#[derive(Debug)]
pub struct JSONSchema {
    pub(crate) node: SchemaNode,
    config: Arc<CompilationOptions>,
}

lazy_static::lazy_static! {
    pub static ref DEFAULT_SCOPE: Url = url::Url::parse(DEFAULT_ROOT_URL).expect("Is a valid URL");
}

impl JSONSchema {
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
    pub fn compile(schema: &Value) -> Result<JSONSchema, ValidationError> {
        Self::options().compile(schema)
    }

    /// Run validation against `instance` and return an iterator over `ValidationError` in the error case.
    #[inline]
    pub fn validate<'instance>(
        &'instance self,
        instance: &'instance Value,
    ) -> Result<(), ErrorIterator<'instance>> {
        let instance_path = InstancePath::new();
        let mut errors = self.node.validate(instance, &instance_path).peekable();
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
        self.node.is_valid(instance)
    }

    /// Apply the schema and return an `Output`. No actual work is done at this point, the
    /// evaluation of the schema is deferred until a method is called on the `Output`. This is
    /// because different output formats will have different performance characteristics.
    ///
    /// # Examples
    ///
    /// "basic" output format
    ///
    /// ```rust
    /// # use crate::jsonschema::{Draft, JSONSchema, output::{Output, BasicOutput}};
    /// let schema_json = serde_json::json!({
    ///     "title": "string value",
    ///     "type": "string"
    /// });
    /// let instance = serde_json::json!{"some string"};
    /// let schema = JSONSchema::options().compile(&schema_json).unwrap();
    /// let output: BasicOutput = schema.apply(&instance).basic();
    /// let output_json = serde_json::to_value(output).unwrap();
    /// assert_eq!(output_json, serde_json::json!({
    ///     "valid": true,
    ///     "annotations": [
    ///         {
    ///             "keywordLocation": "",
    ///             "instanceLocation": "",
    ///             "annotations": {
    ///                 "title": "string value"
    ///             }
    ///         }
    ///     ]
    /// }));
    /// ```
    pub const fn apply<'a, 'b>(&'a self, instance: &'b Value) -> Output<'a, 'b> {
        Output::new(self, &self.node, instance)
    }

    /// The [`Draft`] which this schema was compiled against
    pub fn draft(&self) -> Draft {
        self.config.draft()
    }

    /// The [`CompilationOptions`] that were used to compile this schema
    pub fn config(&self) -> Arc<CompilationOptions> {
        Arc::clone(&self.config)
    }
}

/// Compile JSON schema into a tree of validators.
#[inline]
pub(crate) fn compile_validators<'a, 'c>(
    schema: &'a Value,
    context: &'c CompilationContext,
) -> Result<SchemaNode, ValidationError<'a>> {
    let context = context.push(schema)?;
    let relative_path = context.clone().into_pointer();
    match schema {
        Value::Bool(value) => match value {
            true => Ok(SchemaNode::new_from_boolean(&context, None)),
            false => Ok(SchemaNode::new_from_boolean(
                &context,
                Some(
                    keywords::boolean::FalseValidator::compile(relative_path)
                        .expect("Should always compile"),
                ),
            )),
        },
        Value::Object(object) => {
            if let Some(reference) = object.get("$ref") {
                let unmatched_keywords = object
                    .iter()
                    .filter_map(|(k, v)| {
                        if k.as_str() != "$ref" {
                            Some((k.clone(), v.clone()))
                        } else {
                            None
                        }
                    })
                    .collect();
                let mut validators = Vec::new();
                if let Value::String(reference) = reference {
                    let validator = keywords::ref_::compile(schema, reference, &context)
                        .expect("Should always return Some")?;
                    validators.push(("$ref".to_string(), validator));
                    Ok(SchemaNode::new_from_keywords(
                        &context,
                        validators,
                        Some(unmatched_keywords),
                    ))
                } else {
                    Err(ValidationError::single_type_error(
                        JSONPointer::default(),
                        relative_path,
                        reference,
                        PrimitiveType::String,
                    ))
                }
            } else {
                let mut validators = Vec::with_capacity(object.len());
                let mut unmatched_keywords = AHashMap::new();
                let mut is_if = false;
                let mut is_props = false;
                for (keyword, subschema) in object {
                    if keyword == "if" {
                        is_if = true;
                    }
                    if keyword == "properties"
                        || keyword == "additionalProperties"
                        || keyword == "patternProperties"
                    {
                        is_props = true;
                    }
                    if let Some(validator) = context
                        .config
                        .draft()
                        .get_validator(keyword)
                        .and_then(|f| f(object, subschema, &context))
                    {
                        validators.push((keyword.clone(), validator?));
                    } else {
                        if let Some(keyword_definition) = context.config.custom_keyword_definition(&keyword) {
                            let validator = compile_custom_keyword_validator(
                                object,
                                subschema,
                                &context,
                                keyword.clone(),
                                keyword_definition.clone()
                            )?;
                            validators.push((keyword.clone(), validator));
                        } else {
                            unmatched_keywords.insert(keyword.to_string(), subschema.clone());
                        }
                    }
                }
                if is_if {
                    unmatched_keywords.remove("then");
                    unmatched_keywords.remove("else");
                }
                if is_props {
                    unmatched_keywords.remove("additionalProperties");
                    unmatched_keywords.remove("patternProperties");
                    unmatched_keywords.remove("properties");
                }
                let unmatched_keywords = if unmatched_keywords.is_empty() {
                    None
                } else {
                    Some(unmatched_keywords)
                };
                Ok(SchemaNode::new_from_keywords(
                    &context,
                    validators,
                    unmatched_keywords,
                ))
            }
        }
        _ => Err(ValidationError::multiple_type_error(
            JSONPointer::default(),
            relative_path,
            schema,
            PrimitiveTypesBitMap::new()
                .add_type(PrimitiveType::Boolean)
                .add_type(PrimitiveType::Object),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::JSONSchema;
    use crate::error::ValidationError;
    use serde_json::{from_str, json, Value};
    use std::{fs::File, io::Read, path::Path};
    use crate::KeywordDefinition;

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
        assert_eq!(compiled.node.validators().len(), 1);
        assert!(compiled.validate(&value1).is_ok());
        assert!(compiled.validate(&value2).is_err());
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

    #[test]
    fn custom_keyword() {
        let schema = json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "array",
                    "byteArray": true,
                    "minItems": 32,
                    "maxItems": 32,
                    "contentMediaType": "application/x.dash.dpp.identifier"
                },
            }
        });

        let json_schema = JSONSchema::options()
            .add_keyword(
                "byteArray",
                KeywordDefinition::Schema(json!({
                    "items": {
                        "type": "integer",
                        "minimum": 0,
                        "maximum": 255,
                    },
                })))
            .compile(&schema)
            .unwrap();

        let mut value = json!({ "id": [289,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32] });

        let result = json_schema.validate(&value);
        let errors: Vec<ValidationError> = result.unwrap_err().collect();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].instance_path.to_string(), "/id/0");
        assert_eq!(errors[0].schema_path.to_string(), "/properties/id/byteArray/items/maximum");
        assert_eq!(
            errors[0].to_string(),
            r#"289 is greater than the maximum of 255"#
        );

        let mut value = json!({ "id": ["string",2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32] });

        let result = json_schema.validate(&value);
        let errors: Vec<ValidationError> = result.unwrap_err().collect();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].instance_path.to_string(), "/id/0");
        assert_eq!(errors[0].schema_path.to_string(), "/properties/id/byteArray/items/type");
        assert_eq!(
            errors[0].to_string(),
            r#""string" is not of type "integer""#
        );
    }
}
