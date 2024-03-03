pub(crate) use json_schema_test_suite_test_case::TestCase;
use serde::Deserialize;
use serde_json::{from_reader, Value};
use std::{ffi::OsStr, fs, fs::File, path::Path};

#[derive(Debug, Deserialize)]
struct JSONSchemaTest {
    data: Value,
    description: String,
    valid: bool,
}

#[derive(Debug, Deserialize)]
struct JSONSchemaTestGroup {
    description: String,
    schema: Value,
    tests: Vec<JSONSchemaTest>,
}

/// Extract the draft version from the path of the test file.
fn draft_version(json_schema_test_suite_path: &Path, file_path: &Path) -> String {
    file_path
        .strip_prefix(json_schema_test_suite_path.join("tests"))
        .ok()
        .and_then(Path::to_str)
        .map(|v| v.split(std::path::MAIN_SEPARATOR))
        .and_then(|mut v| v.next())
        .map_or_else(
            || {
                panic!(
                    "No issues are expected while extracting the draft-version from the file_path. json_schema_test_suite_path={}, file_path={}",
                    json_schema_test_suite_path.display(),
                    file_path.display()
                )
            },
            ToString::to_string,
        )
}

fn load_inner(json_schema_test_suite_path: &Path, dir: &Path, prefix: &str) -> Vec<TestCase> {
    let mut tests = vec![];
    for result_entry in fs::read_dir(dir).unwrap_or_else(|_| {
        panic!(
            r#"JSON Schema Test Suite not found.
Please ensure the test suite has been initialized correctly.
Run `git submodule init` and `git submodule update` in the root directory to initialize it.
If the issue persists, please verify the path to `{}` is correct."#,
            dir.display()
        )
    }) {
        if let Ok(entry) = result_entry {
            let path = entry.path();
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                tests.extend(load_inner(
                    json_schema_test_suite_path,
                    &path,
                    &format!(
                        "{}{}_",
                        prefix,
                        path.file_name().and_then(OsStr::to_str).unwrap_or_else(|| {
                            panic!(
                                "No issues are expected while extracting the filename from path={}",
                                path.display()
                            )
                        })
                    ),
                ));
            } else if let Ok(file_reader) = File::open(&path) {
                let test_groups: Vec<JSONSchemaTestGroup> = from_reader(file_reader).unwrap_or_else(|_| {
                    panic!(
                        r#"{} does not contain valid content. Expected something like [{{"schema": ..., "tests": [{{"data": ..., "is_valid": ...}}, ...]}}]"#,
                        path.display()
                    );
                });

                tests.extend(test_groups.iter().enumerate().flat_map(|(gid, test_group)| {
                    test_group
                        .tests
                        .iter()
                        .enumerate()
                        .map(|(tid, test_case)| TestCase {
                            name: format!(
                                "{}{}_{}_{}",
                                prefix,
                                path.file_stem()
                                    .and_then(OsStr::to_str)
                                    .unwrap_or_else(|| {
                                        panic!(
                                            "No issues are expected while extracting the filename (without extension) from path={}",
                                            path.display()
                                        );
                                    })
                                    .replace('-', "_"),
                                gid,
                                tid
                            ),
                            draft_version: draft_version(json_schema_test_suite_path, &path),
                            group_description: test_group.description.clone(),
                            test_case_description: test_case.description.clone(),
                            schema: test_group.schema.clone(),
                            instance: test_case.data.clone(),
                            is_valid: test_case.valid,
                        })
                        .collect::<Vec<_>>()
                }))
            }
        }
    }
    tests
}

/// Load all the test cases present into `draft_folder`
pub(crate) fn load(json_schema_test_suite_path: &Path, draft_folder: &str) -> Vec<TestCase> {
    load_inner(
        json_schema_test_suite_path,
        &json_schema_test_suite_path.join("tests").join(draft_folder),
        "",
    )
}

pub(crate) struct WrappedTestCase<'a>(&'a TestCase);
impl<'a> From<&'a TestCase> for WrappedTestCase<'a> {
    fn from(value: &'a TestCase) -> Self {
        Self(value)
    }
}

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
impl<'a> ToTokens for WrappedTestCase<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.0.name;
        let draft_version = &self.0.draft_version;
        let group_description = &self.0.group_description;
        let test_case_description = &self.0.test_case_description;
        let schema_str = self.0.schema.to_string();
        let instance_str = self.0.instance.to_string();
        let is_valid = self.0.is_valid;

        let output = quote! {
            json_schema_test_suite::TestCase {
                name: #name.to_string(),
                draft_version: #draft_version.to_string(),
                group_description: #group_description.to_string(),
                test_case_description: #test_case_description.to_string(),
                schema: serde_json::from_str(#schema_str).unwrap(),
                instance: serde_json::from_str(#instance_str).unwrap(),
                is_valid: #is_valid,
            }
        };
        tokens.extend(output);
    }
}
