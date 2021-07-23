# json_schema_test_suite_test_case

This crate is supposed to support [`json_schema_test_suite`](https://crates.io/crates/json_schema_test_suite)
by exporting the base struct to allow users of `json_schema_test_suite` to have a single entity representing the test information.

This crate is needed because currently, at the time of writing, we are not allowed to export structs from a proc-macro library
Please refer to [`json-schema-test-suite` docs](https://docs.rs/json-schema-test-suite) for more informaton.

License: MIT
