pub(crate) mod applicator;
pub(crate) mod ref_;
pub(crate) mod validation;

pub(crate) use applicator::{items::ItemsArray, properties::Properties};
pub(crate) use ref_::Ref;

pub enum Vocabulary {
    Validation,
    Applicator,
    Core,
}

pub trait Validate {
    fn is_valid(&self, instance: &serde_json::Value) -> bool;
}

#[derive(Debug)]
pub enum Keyword {
    ItemsArray(ItemsArray),
    Maximum(validation::Maximum),
    Properties(Properties),
    Ref(Ref),
}

impl From<ItemsArray> for Keyword {
    fn from(v: ItemsArray) -> Keyword {
        Keyword::ItemsArray(v)
    }
}
impl From<validation::Maximum> for Keyword {
    fn from(v: validation::Maximum) -> Keyword {
        Keyword::Maximum(v)
    }
}
impl From<Properties> for Keyword {
    fn from(v: Properties) -> Keyword {
        Keyword::Properties(v)
    }
}
impl From<Ref> for Keyword {
    fn from(v: Ref) -> Keyword {
        Keyword::Ref(v)
    }
}

impl Keyword {
    #[inline]
    pub fn vocabulary(&self) -> Vocabulary {
        match self {
            Keyword::ItemsArray(_) => Vocabulary::Applicator,
            Keyword::Maximum(_) => Vocabulary::Validation,
            Keyword::Properties(_) => Vocabulary::Applicator,
            Keyword::Ref(_) => Vocabulary::Core,
        }
    }
    #[inline]
    pub fn is_valid(&self, instance: &serde_json::Value) -> bool {
        match self {
            Keyword::ItemsArray(inner) => inner.is_valid(instance),
            Keyword::Maximum(inner) => inner.is_valid(instance),
            Keyword::Properties(inner) => inner.is_valid(instance),
            Keyword::Ref(inner) => inner.is_valid(instance),
        }
    }
}
