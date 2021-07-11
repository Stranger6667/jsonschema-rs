use crate::{paths::InstancePath, ErrorIterator, JSONSchema};
use serde_json::Value;
use std::fmt;

// Hasher for validator reuse.

pub(crate) use crate::compilation::context::{ValidatorBuf, ValidatorRef};

pub(crate) trait Validate: Send + Sync + core::fmt::Display {
    fn validate<'a>(
        &self,
        schema: &'a JSONSchema,
        instance: &'a Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'a>;
    // The same as above, but does not construct ErrorIterator.
    // It is faster for cases when the result is not needed (like anyOf), since errors are
    // not constructed
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool;
}

impl fmt::Debug for dyn Validate + Send + Sync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_string())
    }
}

pub(crate) type Validators = Vec<ValidatorRef>;

pub(crate) fn format_validators(validators: &[ValidatorRef]) -> String {
    match validators.len() {
        0 => "{}".to_string(),
        1 => {
            let name = validators[0].to_string();
            match name.as_str() {
                // boolean validators are represented as is, without brackets because if they
                // occur in a vector, then the schema is not a key/value mapping
                "true" | "false" => name,
                _ => format!("{{{}}}", name),
            }
        }
        _ => format!(
            "{{{}}}",
            validators
                .iter()
                .map(|validator| format!("{:?}", validator))
                .collect::<Vec<String>>()
                .join(", ")
        ),
    }
}

pub(crate) fn format_vec_of_validators(validators: &[Validators]) -> String {
    validators
        .iter()
        .map(|v| format_validators(v))
        .collect::<Vec<String>>()
        .join(", ")
}

pub(crate) fn format_key_value_validators(validators: &[(String, Validators)]) -> String {
    validators
        .iter()
        .map(|(name, validators)| format!("{}: {}", name, format_validators(validators)))
        .collect::<Vec<String>>()
        .join(", ")
}
