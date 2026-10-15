//! Canonicalizes every named schema of one `OpenAPI` / Swagger document.
//!
//! `canonical-ecosystem <document>` prints one JSON line per schema: `pointer`, `status`, `hash`.
//! `canonical-ecosystem <document> <pointer>` prints that schema's canonical form.
use std::{
    error::Error,
    hash::{DefaultHasher, Hash, Hasher},
    process::ExitCode,
};

use jsonschema::{
    canonical::{self, CanonicalKind, CanonicalSchema, CanonicalizeOptions},
    PatternOptions,
};
use serde_json::{json, Value};

/// What a test generator canonicalizing an `OpenAPI` document uses.
fn options() -> CanonicalizeOptions<'static> {
    canonical::options()
        .should_validate_formats(true)
        .with_pattern_options(
            PatternOptions::fancy_regex()
                .backtrack_limit(usize::MAX)
                .size_limit(128 * 1024 * 1024)
                .dfa_size_limit(128 * 1024 * 1024),
        )
}

fn pointers(document: &Value) -> Vec<String> {
    let (prefix, definitions) = if let Some(schemas) = document.pointer("/components/schemas") {
        ("/components/schemas/", schemas)
    } else if let Some(definitions) = document.get("definitions") {
        ("/definitions/", definitions)
    } else {
        return Vec::new();
    };
    definitions
        .as_object()
        .into_iter()
        .flat_map(|map| map.keys())
        .map(|name| {
            let mut pointer = prefix.to_string();
            referencing::write_escaped_str(&mut pointer, name);
            pointer
        })
        .collect()
}

fn hash(text: &str) -> String {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn render(
    result: &Result<CanonicalSchema, canonical::CanonicalizationError>,
) -> (&'static str, String) {
    match result {
        Ok(schema) => {
            let status = if matches!(schema.kind(), CanonicalKind::Raw) {
                "raw"
            } else {
                "ok"
            };
            let json = serde_json::to_string_pretty(&schema.to_json_schema())
                .expect("canonical output serializes");
            (status, json)
        }
        Err(error) => ("err", error.to_string()),
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .ok_or("usage: canonical-ecosystem <document> [pointer]")?;
    let bytes = std::fs::read(&path).map_err(|error| format!("{path}: {error}"))?;
    let document: Value =
        serde_json::from_slice(&bytes).map_err(|error| format!("{path}: {error}"))?;
    let pointer = args.next();
    let prepared = match options().prepare(&document) {
        Ok(prepared) => prepared,
        Err(error) if pointer.is_some() => {
            println!("{error}");
            return Ok(());
        }
        // The message names one of several unresolvable references, picked in no fixed order.
        Err(_) => {
            println!(
                "{}",
                json!({"pointer": "-", "status": "prepare-error", "hash": "-"})
            );
            return Ok(());
        }
    };
    if let Some(pointer) = pointer {
        // Earlier reads of a prepared document can shape later outputs, so replay the run's order.
        for earlier in pointers(&document)
            .into_iter()
            .take_while(|earlier| *earlier != pointer)
        {
            let _ = prepared.canonicalize_at(&earlier);
        }
        println!("{}", render(&prepared.canonicalize_at(&pointer)).1);
        return Ok(());
    }
    for pointer in pointers(&document) {
        let (status, output) = render(&prepared.canonicalize_at(&pointer));
        println!(
            "{}",
            json!({"pointer": pointer, "status": status, "hash": hash(&output)})
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
