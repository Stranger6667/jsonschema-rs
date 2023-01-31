use crate::Schema;
use std::ops::Range;

pub(crate) mod applicator;
pub(crate) mod references;
pub(crate) mod validation;

use applicator::{AllOf, Items, Properties};
use references::Ref;
use validation::{MaxLength, Maximum, MinProperties, Type};

macro_rules! keywords {
    ($($kw:ident),+) => {
        #[derive(Eq, PartialEq)]
        pub enum Keyword {
            $(
                $kw($kw),
            )+
        }

        // Display only the inner value to reduce visual clutter
        impl core::fmt::Debug for Keyword {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                match self {
                    $(
                        Self::$kw(inner) => inner.fmt(f),
                    )+
                }
            }
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
    Items,
    Properties,
    MaxLength,
    Maximum,
    MinProperties,
    Type,
    Ref
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
            Keyword::Items(inner) => inner.is_valid(schema, instance),
            Keyword::Maximum(inner) => inner.is_valid(instance),
            Keyword::MaxLength(inner) => inner.is_valid(instance),
            Keyword::MinProperties(inner) => inner.is_valid(instance),
            Keyword::Properties(inner) => inner.is_valid(schema, instance),
            Keyword::Type(inner) => inner.is_valid(instance),
            Keyword::Ref(inner) => inner.is_valid(schema, instance),
        }
    }

    #[inline]
    pub fn validate(&self, _schema: &Schema, _instance: &serde_json::Value) -> Option<u64> {
        Some(42)
    }

    pub(crate) fn edges(&self) -> Option<Range<usize>> {
        match self {
            Keyword::AllOf(inner) => Some(inner.edges.clone()),
            Keyword::Properties(inner) => Some(inner.edges.clone()),
            _ => None,
        }
    }
    pub(crate) fn edges_mut(&mut self) -> Option<&mut Range<usize>> {
        match self {
            Keyword::AllOf(inner) => Some(&mut inner.edges),
            Keyword::Properties(inner) => Some(&mut inner.edges),
            _ => None,
        }
    }
}
