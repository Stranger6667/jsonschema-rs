pub(crate) mod applicator;
pub(crate) mod ref_;
pub(crate) mod validation;

use enum_dispatch::enum_dispatch;

pub(crate) use applicator::{items::ItemsArray, properties::Properties};
pub(crate) use ref_::Ref;
pub(crate) use validation::maximum::Maximum;

pub enum Vocabulary {
    Validation,
    Applicator,
    Core,
}

#[enum_dispatch]
pub trait Validate {
    fn vocabulary(&self) -> Vocabulary;
    fn is_valid(&self, instance: &serde_json::Value) -> bool;
}

#[enum_dispatch(Validate)]
#[derive(Debug)]
pub enum Keyword {
    ItemsArray,
    Maximum,
    Properties,
    Ref,
}
