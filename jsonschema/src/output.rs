use std::{
    collections::VecDeque,
    iter::{FromIterator, Sum},
    ops::AddAssign,
};

use crate::{validator::PartialApplication, ValidationError};
use ahash::AHashMap;
use serde::ser::SerializeMap;

use crate::{
    paths::{AbsolutePath, InstancePath, JSONPointer},
    schema_node::SchemaNode,
    JSONSchema,
};

/// The output format resulting from the application of a schema. This can be
/// converted into various representations based on the definitions in
/// <https://json-schema.org/draft/2020-12/json-schema-core.html#rfc.section.12.2>
///
/// Currently only the "flag" and "basic" output formats are supported
#[derive(Debug, Clone)]
pub struct Output<'a, 'b> {
    schema: &'a JSONSchema,
    root_node: &'a SchemaNode,
    instance: &'b serde_json::Value,
}

impl<'a, 'b> Output<'a, 'b> {
    pub(crate) const fn new<'c, 'd>(
        schema: &'c JSONSchema,
        root_node: &'c SchemaNode,
        instance: &'d serde_json::Value,
    ) -> Output<'c, 'd> {
        Output {
            schema,
            root_node,
            instance,
        }
    }

    /// Indicates whether the schema was valid, corresponds to the "flag" output
    /// format
    pub fn flag(&self) -> bool {
        self.schema.is_valid(self.instance)
    }

    /// Output a list of errors and annotations for each element in the schema
    /// according to the basic output format
    pub fn basic(&self) -> BasicOutput<'a> {
        self.root_node
            .apply_rooted(self.schema, self.instance, &InstancePath::new())
    }
}

/// The "basic" output format
#[derive(Debug, PartialEq)]
pub enum BasicOutput<'a> {
    /// The schema was valid, collected annotations can be examined
    Valid(VecDeque<OutputUnit<Annotations<'a>>>),
    /// The schema was invalid
    Invalid(VecDeque<OutputUnit<ErrorDescription>>),
}

impl<'a> BasicOutput<'a> {
    pub(crate) const fn is_valid(&self) -> bool {
        match self {
            BasicOutput::Valid(..) => true,
            BasicOutput::Invalid(..) => false,
        }
    }
}

impl<'a> From<OutputUnit<Annotations<'a>>> for BasicOutput<'a> {
    fn from(unit: OutputUnit<Annotations<'a>>) -> Self {
        let mut units = VecDeque::new();
        units.push_front(unit);
        BasicOutput::Valid(units)
    }
}

impl<'a> AddAssign for BasicOutput<'a> {
    fn add_assign(&mut self, rhs: Self) {
        match (&mut *self, rhs) {
            (BasicOutput::Valid(ref mut anns), BasicOutput::Valid(anns_rhs)) => {
                anns.extend(anns_rhs);
            }
            (BasicOutput::Valid(..), BasicOutput::Invalid(errors)) => {
                *self = BasicOutput::Invalid(errors)
            }
            (BasicOutput::Invalid(..), BasicOutput::Valid(..)) => {}
            (BasicOutput::Invalid(errors), BasicOutput::Invalid(errors_rhs)) => {
                errors.extend(errors_rhs)
            }
        }
    }
}

impl<'a> Sum for BasicOutput<'a> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let result = BasicOutput::Valid(VecDeque::new());
        iter.fold(result, |mut acc, elem| {
            acc += elem;
            acc
        })
    }
}

impl<'a> Default for BasicOutput<'a> {
    fn default() -> Self {
        BasicOutput::Valid(VecDeque::new())
    }
}

impl<'a> From<BasicOutput<'a>> for PartialApplication<'a> {
    fn from(output: BasicOutput<'a>) -> Self {
        match output {
            BasicOutput::Valid(anns) => PartialApplication::Valid {
                annotations: None,
                child_results: anns,
            },
            BasicOutput::Invalid(errors) => PartialApplication::Invalid {
                errors: Vec::new(),
                child_results: errors,
            },
        }
    }
}

impl<'a> FromIterator<BasicOutput<'a>> for PartialApplication<'a> {
    fn from_iter<T: IntoIterator<Item = BasicOutput<'a>>>(iter: T) -> Self {
        iter.into_iter().sum::<BasicOutput<'_>>().into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputUnit<T> {
    keyword_location: JSONPointer,
    instance_location: JSONPointer,
    absolute_keyword_location: Option<AbsolutePath>,
    value: T,
}

impl<T> OutputUnit<T> {
    pub(crate) const fn annotations(
        keyword_location: JSONPointer,
        instance_location: JSONPointer,
        absolute_keyword_location: Option<AbsolutePath>,
        annotations: Annotations<'_>,
    ) -> OutputUnit<Annotations<'_>> {
        OutputUnit {
            keyword_location,
            instance_location,
            absolute_keyword_location,
            value: annotations,
        }
    }

    pub(crate) const fn error(
        keyword_location: JSONPointer,
        instance_location: JSONPointer,
        absolute_keyword_location: Option<AbsolutePath>,
        error: ErrorDescription,
    ) -> OutputUnit<ErrorDescription> {
        OutputUnit {
            keyword_location,
            instance_location,
            absolute_keyword_location,
            value: error,
        }
    }
}

#[derive(serde::Serialize, Debug, Clone, PartialEq)]
pub struct Annotations<'a>(AnnotationsInner<'a>);

#[derive(Debug, Clone, PartialEq)]
enum AnnotationsInner<'a> {
    UnmatchedKeywords(&'a AHashMap<String, serde_json::Value>),
    ValueRef(&'a serde_json::Value),
    Value(Box<serde_json::Value>),
}

impl<'a> From<&'a AHashMap<String, serde_json::Value>> for Annotations<'a> {
    fn from(anns: &'a AHashMap<String, serde_json::Value>) -> Self {
        Annotations(AnnotationsInner::UnmatchedKeywords(anns))
    }
}

impl<'a> From<&'a serde_json::Value> for Annotations<'a> {
    fn from(v: &'a serde_json::Value) -> Self {
        Annotations(AnnotationsInner::ValueRef(v))
    }
}

impl<'a> From<serde_json::Value> for Annotations<'a> {
    fn from(v: serde_json::Value) -> Self {
        Annotations(AnnotationsInner::Value(Box::new(v)))
    }
}

#[derive(serde::Serialize, Debug, Clone, PartialEq)]
pub struct ErrorDescription(String);

impl From<ValidationError<'_>> for ErrorDescription {
    fn from(e: ValidationError<'_>) -> Self {
        ErrorDescription(e.to_string())
    }
}

impl<'a> From<&'a str> for ErrorDescription {
    fn from(s: &'a str) -> Self {
        ErrorDescription(s.to_string())
    }
}

impl<'a> serde::Serialize for BasicOutput<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map_ser = serializer.serialize_map(Some(2))?;
        match self {
            BasicOutput::Valid(outputs) => {
                map_ser.serialize_entry("valid", &true)?;
                map_ser.serialize_entry("annotations", outputs)?;
            }
            BasicOutput::Invalid(errors) => {
                map_ser.serialize_entry("valid", &false)?;
                map_ser.serialize_entry("errors", errors)?;
            }
        }
        map_ser.end()
    }
}

impl<'a> serde::Serialize for AnnotationsInner<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::UnmatchedKeywords(kvs) => kvs.serialize(serializer),
            Self::Value(v) => v.serialize(serializer),
            Self::ValueRef(v) => v.serialize(serializer),
        }
    }
}

impl<'a> serde::Serialize for OutputUnit<Annotations<'a>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map_ser = serializer.serialize_map(Some(4))?;
        map_ser.serialize_entry("keywordLocation", &self.keyword_location)?;
        map_ser.serialize_entry("instanceLocation", &self.instance_location)?;
        if let Some(absolute) = &self.absolute_keyword_location {
            map_ser.serialize_entry("absoluteKeywordLocation", &absolute)?;
        }
        map_ser.serialize_entry("annotations", &self.value)?;
        map_ser.end()
    }
}

impl<'a> serde::Serialize for OutputUnit<ErrorDescription> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map_ser = serializer.serialize_map(Some(4))?;
        map_ser.serialize_entry("keywordLocation", &self.keyword_location)?;
        map_ser.serialize_entry("instanceLocation", &self.instance_location)?;
        if let Some(absolute) = &self.absolute_keyword_location {
            map_ser.serialize_entry("absoluteKeywordLocation", &absolute)?;
        }
        map_ser.serialize_entry("error", &self.value)?;
        map_ser.end()
    }
}
