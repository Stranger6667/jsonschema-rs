//! This crate is supposed to support [`json_schema_test_suite`](https://crates.io/crates/json_schema_test_suite)
//! by exporting the base struct to allow users of `json_schema_test_suite` to have a single entity representing the test information.
//!
//! This crate is needed because currently, at the time of writing, we are not allowed to export structs from a proc-macro library
//! Please refer to [`json-schema-test-suite` docs](https://docs.rs/json-schema-test-suite) for more informaton.
#![warn(
    clippy::cast_possible_truncation,
    clippy::doc_markdown,
    clippy::explicit_iter_loop,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::needless_pass_by_value,
    clippy::option_map_unwrap_or,
    clippy::option_map_unwrap_or_else,
    clippy::option_unwrap_used,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::redundant_closure,
    clippy::result_map_unwrap_or_else,
    clippy::result_unwrap_used,
    clippy::trivially_copy_pass_by_ref,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    unreachable_pub,
    unsafe_code,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]

use serde_json::Value;

/// Detailed information about the individual test case in JSON-Schema-Test-Suite
#[derive(Clone, Debug)]
pub struct TestCase {
    /// Test name (unique identifier)
    pub name: String,
    /// String representation of the draft version (equialent to the test directory)
    /// This is usefull in case your test needs to be aware of the draft version under test
    pub draft_version: String,
    /// Description of the test group as provided by JSON-Schema-Test-Suite
    pub group_description: String,
    /// Description of the test group as provided by JSON-Schema-Test-Suite
    pub test_case_description: String,
    /// Schema to be tested
    pub schema: Value,
    /// Instance to be validated against the `schema
    pub instance: Value,
    /// Expected validity status as from JSON-Schema-Test-Suite
    pub is_valid: bool,
}
