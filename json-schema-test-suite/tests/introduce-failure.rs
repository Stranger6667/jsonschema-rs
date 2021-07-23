use json_schema_test_suite::{json_schema_test_suite, TestCase};

#[json_schema_test_suite("path/to/JSON-Schema-Test-Suite/repository", "draft7", {"ref_0_0", r"optional_format_regex_0_\d+"})]
fn no_op_test_with_some_failures(server_address: &str, test_case: TestCase) {
    if test_case.name == "ref_0_0" {
        // Test that failure is properly ignored via the macro argument
        panic!("The test should fail, but we ensure that this is ignored")
    } else if test_case.name == "ref_0_1" {
        // Test that mocks work as expected
        match reqwest::blocking::get(&format!("http://{}/integer.json", server_address))
            .and_then(reqwest::blocking::Response::json::<serde_json::Value>)
        {
            Ok(json_resp) => assert_eq!(json_resp, serde_json::json!({"type": "integer"})),
            Err(err) => panic!("Issue while interacting with mocks: {:?}", err),
        }
    } else if test_case.name.starts_with("optional_format_regex_0_") {
        panic!("We want to force a falure for all the test named like 'optional_format_regex_0_.*' to ensure that the test ignore list includes them")
    }
}
