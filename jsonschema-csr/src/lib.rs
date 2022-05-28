mod compilation;
mod vocabularies;

pub use compilation::JsonSchema;

#[cfg(feature = "benchmark-internals")]
pub use compilation::resolver::{scope_of, Resolver};
