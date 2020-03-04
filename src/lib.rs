mod checks;
mod context;
mod error;
mod helpers;
mod keywords;
mod resolver;
mod schemas;
mod validator;
pub use error::ValidationError;
pub use schemas::Draft;
pub use validator::JSONSchema;

#[macro_use]
extern crate lazy_static;
