use json_schema_test_suite::{json_schema_test_suite, TestCase};
use jsonschema::{Draft, JSONSchema};
use std::fs;

#[json_schema_test_suite("tests/suite", "draft4", {"optional_bignum_0_0", "optional_bignum_2_0", r"optional_format_email_0_\d+"})]
#[json_schema_test_suite("tests/suite", "draft6", {r"optional_format_email_0_\d+"})]
#[json_schema_test_suite("tests/suite", "draft7", {
    r"optional_format_idn_hostname_0_\d+",  // https://github.com/Stranger6667/jsonschema-rs/issues/101
    r"optional_format_email_0_\d+"
})]
fn test_draft(_server_address: &str, test_case: TestCase) {
    let draft_version = match test_case.draft_version.as_ref() {
        "draft4" => Draft::Draft4,
        "draft6" => Draft::Draft6,
        "draft7" => Draft::Draft7,
        _ => panic!("Unsupported draft"),
    };

    let compiled = JSONSchema::options()
        .with_draft(draft_version)
        .compile(&test_case.schema)
        .unwrap();

    let result = compiled.validate(&test_case.instance);

    if test_case.is_valid {
        if let Err(mut errors_iterator) = result {
            let first_error = errors_iterator.next();
            assert!(
                first_error.is_none(),
                "Schema: {}\nInstance: {}\nError: {:?}",
                test_case.schema,
                test_case.instance,
                first_error,
            );
        }
    } else {
        assert!(
            result.is_err(),
            "Schema: {}\nInstance: {}\nError: It is supposed to be INVALID!",
            test_case.schema,
            test_case.instance,
        );
        let errors: Vec<_> = result.expect_err("Errors").collect();
        for error in errors {
            let pointer = error.instance_path.to_string();
            assert_eq!(test_case.instance.pointer(&pointer), Some(&*error.instance))
        }
    }

    // Ensure that `JSONSchema::is_valid` is in sync with the validity expectation
    assert_eq!(compiled.is_valid(&test_case.instance), test_case.is_valid);
}

#[test]
fn test_instance_path() {
    let expectations: serde_json::Value =
        serde_json::from_str(include_str!("draft7_instance_paths.json")).expect("Valid JSON");
    for (filename, expected) in expectations.as_object().expect("Is object") {
        let test_file = fs::read_to_string(format!("tests/suite/tests/draft7/{}", filename))
            .unwrap_or_else(|_| panic!("Valid file: {}", filename));
        let data: serde_json::Value = serde_json::from_str(&test_file).expect("Valid JSON");
        for item in expected.as_array().expect("Is array") {
            let suite_id = item["suite_id"].as_u64().expect("Is integer") as usize;
            let raw_schema = &data[suite_id]["schema"];
            let schema = JSONSchema::compile(raw_schema).unwrap_or_else(|_| {
                panic!(
                    "Valid schema. File: {}; Suite ID: {}; Schema: {}",
                    filename, suite_id, raw_schema
                )
            });
            for test_data in item["tests"].as_array().expect("Valid array") {
                let test_id = test_data["id"].as_u64().expect("Is integer") as usize;
                let instance_path: Vec<&str> = test_data["instance_path"]
                    .as_array()
                    .expect("Valid array")
                    .iter()
                    .map(|value| value.as_str().expect("A string"))
                    .collect();
                let instance = &data[suite_id]["tests"][test_id]["data"];
                let error = schema
                    .validate(instance)
                    .expect_err(&format!(
                        "File: {}; Suite ID: {}; Test ID: {}",
                        filename, suite_id, test_id
                    ))
                    .next()
                    .expect("Validation error");
                assert_eq!(
                    error.instance_path.into_vec(),
                    instance_path,
                    "File: {}; Suite ID: {}; Test ID: {}",
                    filename,
                    suite_id,
                    test_id
                )
            }
        }
    }
}
