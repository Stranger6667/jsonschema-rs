/*!
The `jsonschema` crate provides a fast and extensible JSON Schema validator.
*/
mod schema;
#[cfg(test)]
pub(crate) mod testing;
pub(crate) mod value_type;
mod vocabularies;

pub use schema::Schema;
