use crate::JsonSchema;

pub(crate) mod applicator;
pub(crate) mod validation;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub(crate) enum KeywordName {
    AllOf,
    Items,
    Maximum,
    Properties,
    Ref,
    Type,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Keyword {
    AllOf(applicator::AllOf),
    ItemsArray(applicator::ItemsArray),
    Maximum(validation::Maximum),
    Properties(applicator::Properties),
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

impl Keyword {
    #[inline]
    pub fn is_valid(&self, schema: &JsonSchema, instance: &serde_json::Value) -> bool {
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
            Keyword::Properties(inner) => inner.is_valid(schema, instance),
        }
    }
}
