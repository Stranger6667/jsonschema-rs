mod compilation;
mod vocabularies;

pub use compilation::JsonSchema;

#[cfg(feature = "benchmark-internals")]
pub use compilation::resolving::{scope_of, Resolver};
