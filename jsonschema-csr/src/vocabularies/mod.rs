use crate::JsonSchema;

pub(crate) mod applicator;
pub(crate) mod core;
pub(crate) mod validation;

pub trait Validate {
    fn is_valid(&self, schema: &JsonSchema, instance: &serde_json::Value) -> bool;
}

#[derive(Debug, Eq, PartialEq)]
pub enum Keyword {
    ItemsArray(applicator::ItemsArray),
    Maximum(validation::Maximum),
    Properties(applicator::Properties),
    Ref(core::Ref),
}

impl From<applicator::ItemsArray> for Keyword {
    fn from(v: applicator::ItemsArray) -> Keyword {
        Keyword::ItemsArray(v)
    }
}
impl From<validation::Maximum> for Keyword {
    fn from(v: validation::Maximum) -> Keyword {
        Keyword::Maximum(v)
    }
}
impl From<applicator::Properties> for Keyword {
    fn from(v: applicator::Properties) -> Keyword {
        Keyword::Properties(v)
    }
}
impl From<core::Ref> for Keyword {
    fn from(v: core::Ref) -> Keyword {
        Keyword::Ref(v)
    }
}

impl Keyword {
    #[inline]
    pub fn is_valid(&self, schema: &JsonSchema, instance: &serde_json::Value) -> bool {
        match self {
            Keyword::ItemsArray(inner) => inner.is_valid(schema, instance),
            Keyword::Maximum(inner) => inner.is_valid(schema, instance),
            Keyword::Properties(inner) => inner.is_valid(schema, instance),
            Keyword::Ref(inner) => inner.is_valid(schema, instance),
        }
    }
}
