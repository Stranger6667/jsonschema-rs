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
pub(crate) mod helpers;
pub(crate) mod if_;
pub(crate) mod items;
pub(crate) mod legacy;
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
use crate::{compilation::JSONSchema, error, error::ErrorIterator};
use serde_json::Value;
use std::fmt::{Debug, Error, Formatter};

pub trait Validate: Send + Sync {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a>;
    // The same as above, but does not construct ErrorIterator.
    // It is faster for cases when the result is not needed (like anyOf), since errors are
    // not constructed
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool;
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
    use serde_json::{from_str, json, Value};
    use std::{fs::File, io::Read, path::Path};

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

    fn load(path: &str) -> Value {
        let full_path = format!("tests/suite/tests/draft7/{}", path);
        let path = Path::new(&full_path);
        let mut file = File::open(&path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).ok().unwrap();
        let data: Value = from_str(&content).unwrap();
        data
    }

    macro_rules! e {
        ($t:ident : $file:expr, $case:expr, $test_id:expr, $expected:expr) => {
            #[test]
            fn $t() {
                let content = load($file);
                let case = &content.as_array().unwrap()[$case];
                let schema = case.get("schema").unwrap();
                let instance = case.get("tests").unwrap().as_array().unwrap()[$test_id]
                    .get("data")
                    .unwrap();
                let compiled = JSONSchema::compile(&schema, None).unwrap();
                let errors: Vec<_> = compiled.validate(&instance).unwrap_err().collect();
                assert_eq!(format!("{}", errors[0]), $expected);
            }
        };
    }

    e!(e1: "additionalItems.json", 0, 1, r#"'"foo"' is not of type 'integer'"#);
    e!(e2: "additionalItems.json", 2, 2, r#"Additional items are not allowed (4 was unexpected)"#);
    e!(e3: "additionalProperties.json", 0, 1, r#"False schema does not allow '"quux"'"#);
    e!(e4: "const.json", 0, 1, r#"'2' was expected"#);
    e!(e5: "contains.json", 0, 3, r#"None of '[2,3,4]' are valid under the given schema"#);
    e!(e6: "enum.json", 0, 1, r#"'4' is not one of '[1,2,3]'"#);
    e!(e7: "exclusiveMaximum.json", 0, 1, r#"3.0 is greater than or equal to the maximum of 3"#);
    e!(e8: "exclusiveMinimum.json", 0, 1, r#"1.1 is less than or equal to the minimum of 1.1"#);
    e!(e9: "maxItems.json", 0, 2, r#"[1,2,3] has more than 2 items"#);
    e!(e10: "maxLength.json", 0, 2, r#"'"foo"' is longer than 2 characters"#);
    e!(e11: "maxProperties.json", 0, 2, r#"{"bar":2,"baz":3,"foo":1} has more than 2 properties"#);
    e!(e12: "minimum.json", 0, 2, r#"0.6 is less than the minimum of 1.1"#);
    e!(e13: "minItems.json", 0, 2, r#"[] has less than 1 item"#);
    e!(e14: "minLength.json", 0, 2, r#"'"f"' is shorter than 2 characters"#);
    e!(e15: "minProperties.json", 0, 2, r#"{} has less than 1 property"#);
    e!(e16: "multipleOf.json", 0, 1, r#"7 is not a multiple of 2"#);
    e!(e17: "not.json", 0, 1, r#"{"type":"integer"} is not allowed for 1"#);
    e!(e18: "pattern.json", 0, 1, r#"'"abc"' does not match '^a*$'"#);
    e!(e19: "required.json", 0, 1, r#"'foo' is a required property"#);
    e!(e20: "type.json", 0, 2, r#"'1.1' is not of type 'integer'"#);
    e!(e21: "uniqueItems.json", 0, 1, r#"'[1,1]' has non-unique elements"#);
}
