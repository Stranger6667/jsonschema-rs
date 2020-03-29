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
use crate::compilation::JSONSchema;
use crate::error;
use crate::error::ErrorIterator;
use serde_json::Value;
use std::fmt::{Debug, Error, Formatter};

pub trait Validate: Send + Sync {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a>;
    // The same as above, but does not construct ErrorIterator.
    // It is faster for cases when the result is not needed (like anyOf), since errors are
    // not constructed
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        self.validate(schema, instance).next().is_none()
    }
    fn name(&self) -> String {
        "<validator>".to_string()
    }
}

impl Debug for dyn Validate + Send + Sync {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(&self.name())
    }
}

pub type CompilationResult = Result<BoxedValidator, error::CompilationError>;
pub type BoxedValidator = Box<dyn Validate + Send + Sync>;
pub type Validators = Vec<BoxedValidator>;

#[cfg(test)]
mod tests {
    use super::JSONSchema;
    use serde_json::json;

    macro_rules! t {
        ($t:ident : $schema:tt => $expected:expr) => {
            #[test]
            fn $t() {
                let schema = json!($schema);
                let compiled = JSONSchema::compile(&schema, None).unwrap();
                assert_eq!(format!("{:?}", compiled.validators[0]), $expected);
            }
        };
    }
    t!(content_media_type_validator: {"contentMediaType": "application/json"} => "<contentMediaType: application/json>");
    t!(content_encoding_validator: {"contentEncoding": "base64"} => "<contentEncoding: base64>");
    t!(combined_validator: {"contentEncoding": "base64", "contentMediaType": "application/json"} => "<contentMediaType - contentEncoding: application/json - base64>");
    t!(date_format: {"format": "date"} => "<format: date>");
    t!(date_time_format: {"format": "date-time"} => "<format: date-time>");
    t!(email_format: {"format": "email"} => "<format: email>");
    t!(hostname_format: {"format": "hostname"} => "<format: hostname>");
    t!(idn_email_format: {"format": "idn-email"} => "<format: idn-email>");
    t!(idn_hostname_format: {"format": "idn-hostname"} => "<format: idn-hostname>");
    t!(ipv4_format: {"format": "ipv4"} => "<format: ipv4>");
    t!(ipv6_format: {"format": "ipv6"} => "<format: ipv6>");
    t!(iri_format: {"format": "iri"} => "<format: iri>");
    t!(iri_reference_format: {"format": "iri-reference"} => "<format: iri-reference>");
    t!(json_pointer_format: {"format": "json-pointer"} => "<format: json-pointer>");
    t!(regex_format: {"format": "regex"} => "<format: regex>");
    t!(relative_json_pointer_format: {"format": "relative-json-pointer"} => "<format: relative-json-pointer>");
    t!(time_format: {"format": "time"} => "<format: time>");
    t!(uri_format: {"format": "uri"} => "<format: uri>");
    t!(uri_reference_format: {"format": "uri-reference"} => "<format: uri-reference>");
    t!(uri_template_format: {"format": "uri-template"} => "<format: uri-template>");
}
