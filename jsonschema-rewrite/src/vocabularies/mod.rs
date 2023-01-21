use crate::Schema;
use std::ops::Range;

pub(crate) mod applicator;
pub(crate) mod validation;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub(crate) enum KeywordName {
    AllOf,
    Items,
    Maximum,
    MaxLength,
    MinProperties,
    Properties,
    Ref,
    Type,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Keyword {
    AllOf(applicator::AllOf),
    ItemsArray(applicator::ItemsArray),
    MaxLength(validation::MaxLength),
    Maximum(validation::Maximum),
    MinProperties(validation::MinProperties),
    Properties(applicator::Properties),
    Type(validation::Type),
}

impl From<applicator::AllOf> for Keyword {
    fn from(v: applicator::AllOf) -> Keyword {
        Keyword::AllOf(v)
    }
}
impl From<applicator::ItemsArray> for Keyword {
    fn from(v: applicator::ItemsArray) -> Keyword {
        Keyword::ItemsArray(v)
    }
}
impl From<validation::MaxLength> for Keyword {
    fn from(v: validation::MaxLength) -> Keyword {
        Keyword::MaxLength(v)
    }
}
impl From<validation::Maximum> for Keyword {
    fn from(v: validation::Maximum) -> Keyword {
        Keyword::Maximum(v)
    }
}
impl From<validation::MinProperties> for Keyword {
    fn from(v: validation::MinProperties) -> Keyword {
        Keyword::MinProperties(v)
    }
}
impl From<applicator::Properties> for Keyword {
    fn from(v: applicator::Properties) -> Keyword {
        Keyword::Properties(v)
    }
}
impl From<validation::Type> for Keyword {
    fn from(v: validation::Type) -> Keyword {
        Keyword::Type(v)
    }
}

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
