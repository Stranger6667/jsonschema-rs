pub mod basic;
pub mod combined;
use crate::{error, validator::Validate};

pub type CompilationResult = Result<BoxedValidator, error::CompilationError>;
pub type BoxedValidator = Box<dyn Validate + Send + Sync>;
pub type Validators = Vec<BoxedValidator>;

fn format_validators(validators: &[BoxedValidator]) -> String {
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

fn format_vec_of_validators(validators: &[Validators]) -> String {
    validators
        .iter()
        .map(|v| format_validators(v))
        .collect::<Vec<String>>()
        .join(", ")
}

fn format_key_value_validators(validators: &[(String, Validators)]) -> String {
    validators
        .iter()
        .map(|(name, validators)| format!("{}: {}", name, format_validators(validators)))
        .collect::<Vec<String>>()
        .join(", ")
}
