//! # referencing
//!
//! An implementation-agnostic JSON reference resolution library for Rust.
mod anchors;
mod error;
mod list;
pub mod meta;
mod registry;
mod resolver;
mod resource;
mod retriever;
mod segments;
mod specification;
pub mod uri;

pub(crate) use anchors::Anchor;
pub use error::{Error, UriError};
pub use list::List;
pub use registry::{Registry, RegistryOptions, SPECIFICATIONS};
pub use resolver::{Resolved, Resolver};
pub use resource::{Resource, ResourceRef};
pub use retriever::{DefaultRetriever, Retrieve};
pub(crate) use segments::Segments;
pub use specification::Draft;

pub type Uri<T> = fluent_uri::Uri<T>;
pub type Iri<T> = fluent_uri::Iri<T>;
pub type UriRef<T> = fluent_uri::UriRef<T>;
pub type IriRef<T> = fluent_uri::IriRef<T>;
