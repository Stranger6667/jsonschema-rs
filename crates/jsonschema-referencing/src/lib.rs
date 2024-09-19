mod anchors;
mod error;
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
pub use registry::{Registry, RegistryOptions, SPECIFICATIONS};
pub use resolver::{Resolved, Resolver};
pub use resource::{Resource, ResourceRef};
pub use retriever::{DefaultRetriever, Retrieve};
pub(crate) use segments::Segments;
pub use specification::Draft;

pub type Uri = fluent_uri::UriRef<String>;
pub type UriRef<'a> = fluent_uri::UriRef<&'a str>;
