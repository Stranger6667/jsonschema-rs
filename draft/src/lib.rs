use heck::{SnakeCase, TitleCase};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use serde_json::{from_reader, Value};
use std::{fs, fs::File, path::Path};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitStr, Token,
};

#[derive(Debug)]
struct InputConfig {
    dir_name: String,
    test_to_exclude: Vec<String>,
}

fn parse_string_from_stream(parse_stream: ParseStream) -> Result<String, syn::Error> {
    Ok(parse_stream.parse::<LitStr>()?.value())
}

fn string_to_ident(string: &str) -> Ident {
    Ident::new(string, Span::call_site())
}
impl Parse for InputConfig {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let dir_name = parse_string_from_stream(input)?;
        let test_to_exclude: Vec<String> = if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;

            let content;
            let _ = braced!(content in input);
            content
                .parse_terminated::<_, Token![,]>(parse_string_from_stream)?
                .iter()
                .cloned()
                .collect::<Vec<_>>()
        } else {
            Vec::with_capacity(0)
        };
        Ok(Self {
            dir_name,
            test_to_exclude,
        })
    }
}

// Usage example
// test_draft!("path/to/the/directory/containing/the/tests")
// test_draft!("path/to/the/directory/containing/the/tests", {"this_test_name_will_not_be_rendered"})
#[proc_macro]
pub fn test_draft(input: TokenStream) -> TokenStream {
    let test_case = parse_macro_input!(input as InputConfig);

    let dir = Path::new(&test_case.dir_name);
    let draft = test_case
        .dir_name
        .trim_end_matches('/')
        .split('/')
        .last()
        .unwrap()
        .to_string();
    let mut rendered_tests = Vec::new();
    for (file_name, test) in load_tests(&dir, String::from("")) {
        for (i, suite) in test.as_array().unwrap().iter().enumerate() {
            let schema_str = suite.get("schema").unwrap().to_string();
            for (j, test) in suite
                .get("tests")
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                let data_str = test.get("data").unwrap().to_string();
                let valid = test.get("valid").unwrap().as_bool().unwrap();
                let test_case_name = format!("{}_{}_{}", file_name, i, j);

                rendered_tests.push(render_test(
                    &draft,
                    &test_case_name,
                    valid,
                    &schema_str,
                    &data_str,
                    test_case.test_to_exclude.contains(&test_case_name),
                ))
            }
        }
    }

    let test_valid = test_valid_token_stream();
    let test_invalid = test_invalid_token_stream();
    let draft_ident = string_to_ident(&draft);

    let output = quote! {
        mod #draft_ident {
            use serde_json::{from_str, Value};
            use jsonschema::{Draft, JSONSchema, ValidationError};

            #test_valid
            #test_invalid

            #(#rendered_tests)*
        }
    };
    output.into()
}

fn load_tests(dir: &Path, prefix: String) -> Vec<(String, Value)> {
    let content = fs::read_dir(dir).unwrap();
    let mut tests = vec![];
    for entry in content {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            let mut prefix = prefix.clone();
            prefix.push_str(path.to_str().unwrap().split('/').last().unwrap());
            prefix.push('_');
            let more = load_tests(&path, prefix);
            tests.extend(more)
        } else {
            let file = File::open(&path).unwrap();
            let data: Value = from_reader(file).unwrap();
            let filename = path.file_name().unwrap().to_str().unwrap();
            tests.push((
                format!(
                    "{}{}",
                    prefix,
                    filename[..filename.len() - 5].to_owned().to_snake_case()
                ),
                data,
            ))
        }
    }
    tests
}

fn test_valid_token_stream() -> TokenStream2 {
    quote! {
        fn test_valid(draft: Draft, schema_str: &str, data_str: &str) {
            let schema: Value = from_str(schema_str).unwrap();
            let data: Value = from_str(data_str).unwrap();

            let compiled = JSONSchema::compile(&schema, Some(draft)).unwrap();

            let result = compiled.validate(&data);

            if let Err(mut errors_iterator) = result {
                let first_error = errors_iterator.next();
                assert!(
                    first_error.is_none(),
                    format!(
                        "Schema: {}\nInstance: {}\nError: {:?}",
                        schema, data, first_error,
                    )
                )
            }
        }
    }
}

fn test_invalid_token_stream() -> TokenStream2 {
    quote! {
        fn test_invalid(draft: Draft, schema_str: &str, data_str: &str) {
            let schema: Value = from_str(schema_str).unwrap();
            let data: Value = from_str(data_str).unwrap();

            let compiled = JSONSchema::compile(&schema, Some(draft)).unwrap();

            let result = compiled.validate(&data);
            assert!(
                !compiled.is_valid(&data),
                format!(
                    "Schema: {}\nInstance: {}\nError: It is supposed to be INVALID!",
                    schema, data,
                )
            )
        }
    }
}

fn render_test(
    draft: &str,
    test_case_name: &str,
    valid: bool,
    schema_str: &str,
    data_str: &str,
    should_ignore: bool,
) -> TokenStream2 {
    let test_case_name_ident = string_to_ident(test_case_name);
    let version_ident = string_to_ident(&draft.to_title_case());
    let maybe_ignore_attr: Option<syn::Attribute> = if should_ignore {
        Some(syn::parse_quote! { #[ignore] })
    } else {
        None
    };
    if valid {
        quote! {
            #maybe_ignore_attr
            #[test]
            fn #test_case_name_ident() {
                test_valid(
                    Draft::#version_ident,
                    #schema_str,
                    #data_str
                )
            }
        }
    } else {
        quote! {
            #maybe_ignore_attr
            #[test]
            fn #test_case_name_ident() {
                test_invalid(
                    Draft::#version_ident,
                    #schema_str,
                    #data_str
                )
            }
        }
    }
}
