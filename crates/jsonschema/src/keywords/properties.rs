use crate::{
    compiler,
    error::{no_error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    node::SchemaNode,
    output::BasicOutput,
    paths::{JsonPointer, JsonPointerNode},
    primitive_type::PrimitiveType,
    validator::{PartialApplication, Validate},
};
use serde_json::{Map, Value};

pub(crate) struct PropertiesValidator {
    pub(crate) properties: Vec<(String, SchemaNode)>,
}

impl PropertiesValidator {
    #[inline]
    pub(crate) fn compile<'a>(ctx: &compiler::Context, schema: &'a Value) -> CompilationResult<'a> {
        match schema {
            Value::Object(map) => {
                let ctx = ctx.with_path("properties");
                let mut properties = Vec::with_capacity(map.len());
                for (key, subschema) in map {
                    let ctx = ctx.with_path(key.as_str());
                    properties.push((
                        key.clone(),
                        compiler::compile(&ctx, ctx.as_resource_ref(subschema))?,
                    ));
                }
                Ok(Box::new(PropertiesValidator { properties }))
            }
            _ => Err(ValidationError::single_type_error(
                JsonPointer::default(),
                ctx.clone().into_pointer(),
                schema,
                PrimitiveType::Object,
            )),
        }
    }
}

impl Validate for PropertiesValidator {
    fn is_valid(&self, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            self.properties.iter().all(move |(name, node)| {
                let option = item.get(name);
                option.into_iter().all(move |item| node.is_valid(item))
            })
        } else {
            true
        }
    }

    #[allow(clippy::needless_collect)]
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &JsonPointerNode,
    ) -> ErrorIterator<'instance> {
        if let Value::Object(item) = instance {
            let errors: Vec<_> = self
                .properties
                .iter()
                .flat_map(move |(name, node)| {
                    let option = item.get(name);
                    option.into_iter().flat_map(move |item| {
                        let instance_path = instance_path.push(name.as_str());
                        node.validate(item, &instance_path)
                    })
                })
                .collect();
            Box::new(errors.into_iter())
        } else {
            no_error()
        }
    }

    fn apply<'a>(
        &'a self,
        instance: &Value,
        instance_path: &JsonPointerNode,
    ) -> PartialApplication<'a> {
        if let Value::Object(props) = instance {
            let mut result = BasicOutput::default();
            let mut matched_props = Vec::with_capacity(props.len());
            for (prop_name, node) in &self.properties {
                if let Some(prop) = props.get(prop_name) {
                    let path = instance_path.push(prop_name.as_str());
                    matched_props.push(prop_name.clone());
                    result += node.apply_rooted(prop, &path);
                }
            }
            let mut application: PartialApplication = result.into();
            application.annotate(Value::from(matched_props).into());
            application
        } else {
            PartialApplication::valid_empty()
        }
    }
}

#[inline]
pub(crate) fn compile<'a>(
    ctx: &compiler::Context,
    parent: &'a Map<String, Value>,
    schema: &'a Value,
) -> Option<CompilationResult<'a>> {
    match parent.get("additionalProperties") {
        // This type of `additionalProperties` validator handles `properties` logic
        Some(Value::Bool(false)) | Some(Value::Object(_)) => None,
        _ => Some(PropertiesValidator::compile(ctx, schema)),
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::json;

    #[test]
    fn schema_path() {
        tests_util::assert_schema_path(
            &json!({"properties": {"foo": {"properties": {"bar": {"required": ["spam"]}}}}}),
            &json!({"foo": {"bar": {}}}),
            "/properties/foo/properties/bar/required",
        )
    }
}
