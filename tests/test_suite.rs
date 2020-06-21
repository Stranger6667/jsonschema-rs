use json_schema_test_suite::{json_schema_test_suite, TestCase};
use jsonschema::{Draft, JSONSchema};

#[json_schema_test_suite("tests/suite", "draft4", {"optional_bignum_0_0", "optional_bignum_2_0"})]
#[json_schema_test_suite("tests/suite", "draft6")]
#[json_schema_test_suite("tests/suite", "draft7", {
    "optional_format_idn_hostname_0_11", // https://github.com/Stranger6667/jsonschema-rs/issues/101
    "optional_format_idn_hostname_0_6",  // https://github.com/Stranger6667/jsonschema-rs/issues/101
    "optional_format_idn_hostname_0_7",  // https://github.com/Stranger6667/jsonschema-rs/issues/101
})]
fn test_draft(_server_address: &str, test_case: TestCase) {
    let draft_version = match test_case.draft_version.as_ref() {
        "draft4" => Draft::Draft4,
        "draft6" => Draft::Draft6,
        "draft7" => Draft::Draft7,
        _ => panic!("Unsupported draft"),
    };

    let compiled = JSONSchema::compile(&test_case.schema, Some(draft_version)).unwrap();

    let result = compiled.validate(&test_case.instance);

    if test_case.is_valid {
        if let Err(mut errors_iterator) = result {
            let first_error = errors_iterator.next();
            assert!(
                first_error.is_none(),
                format!(
                    "Schema: {}\nInstance: {}\nError: {:?}",
                    test_case.schema, test_case.instance, first_error,
                )
            )
        }
    } else {
        assert!(
            result.is_err(),
            format!(
                "Schema: {}\nInstance: {}\nError: It is supposed to be INVALID!",
                test_case.schema, test_case.instance,
            )
        );
    }
}
