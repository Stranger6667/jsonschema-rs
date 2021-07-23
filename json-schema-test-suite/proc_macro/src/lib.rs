//! This crate is supposed to support [`json_schema_test_suite`](https://crates.io/crates/json_schema_test_suite)
//! by exporting `json_schema_test_suite` procedural macro.
//!
//! Please refer to [`json-schema-test-suite-proc-macro` docs](https://docs.rs/json-schema-test-suite) for more informaton.
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
mod attribute_config;
mod mockito_mocks;
mod test_case;

use json_schema_test_suite_test_case::TestCase;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use regex::Regex;
use syn::{parse_macro_input, Ident, ItemFn};

fn test_token_stream(
    tests_to_ignore_regex: &[Regex],
    wrapped_test_ident: &Ident,
    test: &TestCase,
) -> TokenStream2 {
    let name = Ident::new(&test.name, Span::call_site());

    let maybe_ignore = if tests_to_ignore_regex
        .iter()
        .any(|regex| regex.is_match(&test.name))
    {
        quote! { #[ignore] }
    } else {
        quote! {}
    };

    let wrapped_test_case = test_case::WrappedTestCase::from(test);
    quote! {
        #[test]
        #maybe_ignore
        fn #name() {
            setup_mocks();

            super::#wrapped_test_ident(
                &mockito::server_address().to_string(),
                #wrapped_test_case,
            );
        }
    }
}

/// Procedural macro that allows a test to be executed for all the configurations defined
/// by [JSON-Schema-Test-Suite](https://github.com/json-schema-org/JSON-Schema-Test-Suite)
///
/// The `proc_macro_attribute` should be used on a function with the current signature
/// ```rust
/// # use json_schema_test_suite_test_case::TestCase;
/// fn my_simple_test(
///     // address of the HTTP server providing the remote files of JSON-Schema-Test-Suite. The format will be: `hostname:port`
///     // This parameter is passed because by starting a mock server we might not start it into `localhost:1234` as expected by JSON-Schema-Test-Suite
///     server_address: &str,
///     // Representation of the test case (includes draft_version, descriptions, schema, instance, expected_valid)
///     test_case: TestCase,
/// ) {
///     // TODO: Add here your testing logic
/// }
/// ```
#[proc_macro_attribute]
pub fn json_schema_test_suite(attr: TokenStream, item: TokenStream) -> TokenStream {
    let proc_macro_attributes = parse_macro_input!(attr as attribute_config::AttrConfig);
    let item_fn = parse_macro_input!(item as ItemFn);

    let original_function_ident = &item_fn.sig.ident;
    let tests_token_stream: Vec<TokenStream2> = test_case::load(
        &proc_macro_attributes.json_schema_test_suite_path,
        &proc_macro_attributes.draft_folder,
    )
    .iter()
    .map(|test| {
        test_token_stream(
            &proc_macro_attributes.tests_to_exclude_regex,
            original_function_ident,
            test,
        )
    })
    .collect();

    let setup_mockito_mocks_token_stream =
        mockito_mocks::setup(&proc_macro_attributes.json_schema_test_suite_path);

    let mod_name = format_ident!(
        "{}_{}",
        original_function_ident,
        proc_macro_attributes.draft_folder
    );

    let output = quote! {
        #item_fn

        mod #mod_name {
            lazy_static::lazy_static! {
                static ref MOCKS: Vec<mockito::Mock> = vec![
                    #(#setup_mockito_mocks_token_stream),*
                ];
            }

            fn setup_mocks() {
                // Dereference to ensure that lazy_static actually invokes the mocks creation
                let _ = *MOCKS;
            }

            #(#tests_token_stream)*
        }
    };
    output.into()
}
