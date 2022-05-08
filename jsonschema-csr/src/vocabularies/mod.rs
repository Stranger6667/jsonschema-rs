pub(crate) mod applicator;
pub(crate) mod ref_;
pub(crate) mod validation;

use enum_dispatch::enum_dispatch;

pub(crate) use applicator::{items::ItemsArray, properties::Properties};
pub(crate) use ref_::Ref;
pub(crate) use validation::maximum::Maximum;

#[enum_dispatch]
pub trait Validate {
    fn is_valid(&self, keywords: &[Keyword], instance: &serde_json::Value) -> bool;
}

#[enum_dispatch(Validate)]
#[derive(Debug)]
pub enum Keyword {
    ItemsArray,
    Maximum,
    Properties,
    Ref,
}

#[derive(Debug)]
pub enum LeafKeyword {
    Maximum,
    Ref,
}

#[derive(Debug)]
pub enum CompositeKeyword {
    ItemsArray,
    Properties,
}
