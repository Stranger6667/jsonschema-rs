use json_schema_test_suite::{json_schema_test_suite, TestCase};

#[json_schema_test_suite("path/to/JSON-Schema-Test-Suite/repository", "draft7")]
fn no_op_test(_server_address: &str, _test_case: TestCase) {}
