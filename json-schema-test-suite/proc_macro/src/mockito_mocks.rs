use proc_macro2::TokenStream;
use quote::quote;
use std::{fs, path::Path};

pub(crate) fn setup(json_schema_test_suite_path: &Path) -> Vec<TokenStream> {
    fn remote_paths(dir: &Path) -> Vec<String> {
        let mut paths = vec![];
        for result_entry in fs::read_dir(dir).unwrap_or_else(|_| panic!("Remotes directory not found: {}", dir.display())) {
            if let Ok(entry) = result_entry {
                let path = entry.path();
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    paths.extend(remote_paths(&path));
                } else {
                    paths.push(path.to_str().map_or_else(
                        || {
                            panic!("No issues are expected while converting path={} to string", path.display());
                        },
                        ToString::to_string,
                    ));
                }
            }
        }
        paths
    }

    let remote_base_path = json_schema_test_suite_path.join("remotes");
    let base_path = remote_base_path.to_str().unwrap_or_else(|| {
        panic!(
            "No issues are expected while converting remote_base_path={} to string",
            remote_base_path.display()
        );
    });
    remote_paths(&remote_base_path)
        .iter()
        .filter_map(|remote_path| {
            let path = remote_path.trim_start_matches(base_path).replace(std::path::MAIN_SEPARATOR, "/");
            if let Ok(file_content) = std::fs::read_to_string(remote_path) {
                Some(quote! {
                    mockito::mock("GET", #path)
                        .with_body(
                            #file_content
                                // Replace static links to localhost:1234 to the mockito generated server address
                                .replace("localhost:1234", &mockito::server_address().to_string())
                        )
                        .create()
                })
            } else {
                None
            }
        })
        .collect()
}
