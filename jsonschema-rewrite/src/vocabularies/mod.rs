use crate::Schema;
use std::ops::Range;

pub(crate) mod applicator;
pub(crate) mod validation;

use applicator::{AllOf, ItemsArray, Properties};
use validation::{MaxLength, Maximum, MinProperties, Type};

macro_rules! keywords {
    ($($kw:ident),+) => {
        #[derive(Debug, Eq, PartialEq)]
        pub enum Keyword {
            $(
                $kw($kw),
            )+
        }

        $(
        impl From<$kw> for Keyword {
            fn from(v: $kw) -> Keyword {
                Keyword::$kw(v)
            }
        }
        )+

    };
}

keywords!(
    AllOf,
    ItemsArray,
    Properties,
    MaxLength,
    Maximum,
    MinProperties,
    Type
);

impl Keyword {
    #[inline]
    pub fn is_valid(&self, schema: &Schema, instance: &serde_json::Value) -> bool {
        // TODO: maybe match by type here - ie if value is Number, only then pass the inner one to keyword
        // match (self, instance) {
        //     (Keyword::Maximum(inner), serde_json::Value::Number(number)) => {
        //         inner.is_valid_number(number)
        //     }
        //     _ => {}
        // }
        match self {
            Keyword::AllOf(inner) => inner.is_valid(schema, instance),
            Keyword::ItemsArray(inner) => inner.is_valid(schema, instance),
            Keyword::Maximum(inner) => inner.is_valid(instance),
            Keyword::MaxLength(inner) => inner.is_valid(instance),
            Keyword::MinProperties(inner) => inner.is_valid(instance),
            Keyword::Properties(inner) => inner.is_valid(schema, instance),
            Keyword::Type(inner) => inner.is_valid(instance),
        }
    }

    #[inline]
    pub fn validate(&self, schema: &Schema, instance: &serde_json::Value) -> Option<u64> {
        Some(42)
    }

    pub(crate) fn edges(&self) -> Option<Range<usize>> {
        match self {
            Keyword::AllOf(inner) => Some(inner.edges.clone()),
            Keyword::Properties(inner) => Some(inner.edges.clone()),
            _ => None,
        }
    }
}
