pub(crate) mod additional_items;
pub(crate) mod additional_properties;
pub(crate) mod all_of;
pub(crate) mod any_of;
pub(crate) mod boolean;
pub(crate) mod const_;
pub(crate) mod contains;
pub(crate) mod content;
pub(crate) mod dependencies;
pub(crate) mod enum_;
pub(crate) mod exclusive_maximum;
pub(crate) mod exclusive_minimum;
pub(crate) mod format;
pub(crate) mod if_;
pub(crate) mod items;
pub(crate) mod max_items;
pub(crate) mod max_length;
pub(crate) mod max_properties;
pub(crate) mod maximum;
pub(crate) mod min_items;
pub(crate) mod min_length;
pub(crate) mod min_properties;
pub(crate) mod minimum;
pub(crate) mod multiple_of;
pub(crate) mod not;
pub(crate) mod one_of;
pub(crate) mod pattern;
pub(crate) mod pattern_properties;
pub(crate) mod properties;
pub(crate) mod property_names;
pub(crate) mod ref_;
pub(crate) mod required;
pub(crate) mod type_;
pub(crate) mod unique_items;
use crate::error;
use crate::validator::JSONSchema;
use serde_json::Value;
use std::fmt::{Debug, Error, Formatter};

pub trait Validate<'a>: Send + Sync + 'a {
    fn validate(&self, config: &JSONSchema, instance: &Value) -> ValidationResult;
    // The same as above, but does not construct Result.
    // It is faster for cases when the result is not needed (like anyOf), since errors are
    // not constructed
    fn is_valid(&self, config: &JSONSchema, instance: &Value) -> bool {
        self.validate(config, instance).is_ok() // TODO. remove it and implement everywhere
    }
    fn name(&self) -> String {
        "<validator>".to_string()
    }
}

impl<'a> Debug for dyn Validate<'a> + Send + Sync + 'a {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(&self.name())
    }
}

pub type ValidationResult = Result<(), error::ValidationError>;
pub type CompilationResult<'a> = Result<BoxedValidator<'a>, error::CompilationError>;
pub type BoxedValidator<'a> = Box<dyn Validate<'a> + Send + Sync + 'a>;
pub type Validators<'a> = Vec<BoxedValidator<'a>>;
