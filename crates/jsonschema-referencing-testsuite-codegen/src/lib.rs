use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn};

mod generator;
mod loader;

/// A procedural macro that generates tests from
/// [JSON-Referencing-Test-Suite](https://github.com/python-jsonschema/referencing-suite).
#[proc_macro_attribute]
pub fn suite(args: TokenStream, input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(args as testsuite_common::SuiteConfig);
    let test_func = parse_macro_input!(input as ItemFn);
    let test_func_ident = &test_func.sig.ident;

    let mut output = quote! {
        #test_func
    };

    for (idx, draft) in config.drafts.iter().enumerate() {
        let suite = match loader::load_suite(&config.path, draft) {
            Ok(suite) => suite,
            Err(e) => {
                let err = e.to_string();
                return TokenStream::from(quote! {
                    compile_error!(#err);
                });
            }
        };
        let modules = generator::generate_modules(&suite, &config.xfail, draft, idx);
        let draft = format_ident!("{}", draft.replace('-', "_"));
        output = quote! {
            #output

            mod #draft {
                use referencing_testsuite::Test;
                use super::#test_func_ident;

                #[inline]
                fn inner_test(draft: &'static str, test: Test) {
                    #test_func_ident(draft, test);
                }
                #modules
            }
        }
    }
    output.into()
}
