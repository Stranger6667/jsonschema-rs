use regex::Regex;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    LitStr, Token,
};

#[derive(Debug)]
pub(crate) struct AttrConfig {
    pub(crate) json_schema_test_suite_path: PathBuf,
    pub(crate) draft_folder: String,
    pub(crate) tests_to_exclude_regex: Vec<Regex>,
}

impl Parse for AttrConfig {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let json_schema_test_suite_path_str: String = input.parse::<LitStr>()?.value();
        let _ = input.parse::<Token![,]>()?;
        let draft_folder: String = input.parse::<LitStr>()?.value();
        let tests_to_exclude_regex: Vec<Regex> = if input.parse::<Token![,]>().is_ok() {
            let tests_to_exclude_tokens = {
                let braced_content;
                braced!(braced_content in input);
                #[allow(clippy::redundant_closure_for_method_calls)]
                let res: syn::punctuated::Punctuated<LitStr, Token![,]> =
                    braced_content.parse_terminated(|v| v.parse())?;
                res
            };
            tests_to_exclude_tokens
                .iter()
                .filter_map(|content| Regex::new(&format!("^{}$", content.value())).ok())
                .collect()
        } else {
            vec![]
        };

        let json_schema_test_suite_path =
            Path::new(&json_schema_test_suite_path_str.replace("/", &MAIN_SEPARATOR.to_string()))
                .to_path_buf();

        Ok(Self {
            json_schema_test_suite_path,
            draft_folder,
            tests_to_exclude_regex,
        })
    }
}
