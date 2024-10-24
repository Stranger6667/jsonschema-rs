#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use jsonschema::Draft;
    use std::fs;
    use testsuite::{suite, Test};

    #[suite(
    path = "crates/jsonschema/tests/suite",
    drafts = [
        "draft4",
        "draft6",
        "draft7",
        "draft2019-09",
        "draft2020-12",
    ],
    xfail = [
        "draft4::optional::bignum::integer::a_bignum_is_an_integer",
        "draft4::optional::bignum::integer::a_negative_bignum_is_an_integer",
    ]
)]
    fn test_suite(test: Test) {
        let mut options = jsonschema::options();
        match test.draft {
            "draft4" => {
                options.with_draft(Draft::Draft4);
            }
            "draft6" => {
                options.with_draft(Draft::Draft6);
            }
            "draft7" => {
                options.with_draft(Draft::Draft7);
            }
            "draft2019-09" | "draft2020-12" => {}
            _ => panic!("Unsupported draft"),
        };
        if test.is_optional {
            options.should_validate_formats(true);
        }
        let validator = options
            .build(&test.schema)
            .expect("Failed to build a schema");

        if test.valid {
            if let Some(first) = validator.iter_errors(&test.data).next() {
                panic!(
                    "Test case should not have validation errors:\nGroup: {}\nTest case: {}\nSchema: {}\nInstance: {}\nError: {first:?}",
                    test.case,
                    test.description,
                    pretty_json(&test.schema),
                    pretty_json(&test.data),
                );
            }
            assert!(
                validator.is_valid(&test.data),
                "Test case should be valid:\nCase: {}\nTest: {}\nSchema: {}\nInstance: {}",
                test.case,
                test.description,
                pretty_json(&test.schema),
                pretty_json(&test.data),
            );
            assert!(
                validator.validate(&test.data).is_ok(),
                "Test case should be valid:\nCase: {}\nTest: {}\nSchema: {}\nInstance: {}",
                test.case,
                test.description,
                pretty_json(&test.schema),
                pretty_json(&test.data),
            );
            let output = validator.apply(&test.data).basic();
            assert!(
                output.is_valid(),
                "Test case should be valid via basic output:\nCase: {}\nTest: {}\nSchema: {}\nInstance: {}\nError: {:?}",
                test.case,
                test.description,
                pretty_json(&test.schema),
                pretty_json(&test.data),
                output
            );
        } else {
            let errors = validator.iter_errors(&test.data).collect::<Vec<_>>();
            assert!(
                !errors.is_empty(),
                "Test case should have validation errors:\nCase: {}\nTest: {}\nSchema: {}\nInstance: {}",
                test.case,
                test.description,
                pretty_json(&test.schema),
                pretty_json(&test.data),
            );
            for error in errors {
                let pointer = error.instance_path.as_str();
                assert_eq!(
                    test.data.pointer(pointer), Some(&*error.instance),
                    "Expected error instance did not match actual error instance:\nCase: {}\nTest: {}\nSchema: {}\nInstance: {}\nExpected pointer: {:#?}\nActual pointer: {:#?}",
                    test.case,
                    test.description,
                    pretty_json(&test.schema),
                    pretty_json(&test.data),
                    &*error.instance,
                    &pointer,
                );
            }
            assert!(
                !validator.is_valid(&test.data),
                "Test case should be invalid:\nCase: {}\nTest: {}\nSchema: {}\nInstance: {}",
                test.case,
                test.description,
                pretty_json(&test.schema),
                pretty_json(&test.data),
            );
            let Some(error) = validator.validate(&test.data).err() else {
                panic!(
                    "Test case should be invalid:\nCase: {}\nTest: {}\nSchema: {}\nInstance: {}",
                    test.case,
                    test.description,
                    pretty_json(&test.schema),
                    pretty_json(&test.data),
                );
            };
            let pointer = error.instance_path.as_str();
            assert_eq!(
                test.data.pointer(pointer), Some(&*error.instance),
                "Expected error instance did not match actual error instance:\nCase: {}\nTest: {}\nSchema: {}\nInstance: {}\nExpected pointer: {:#?}\nActual pointer: {:#?}",
                test.case,
                test.description,
                pretty_json(&test.schema),
                pretty_json(&test.data),
                &*error.instance,
                &pointer,
            );
            let output = validator.apply(&test.data).basic();
            assert!(
                !output.is_valid(),
                "Test case should be invalid via basic output:\nCase: {}\nTest: {}\nSchema: {}\nInstance: {}",
                test.case,
                test.description,
                pretty_json(&test.schema),
                pretty_json(&test.data),
            );
        }
    }

    fn pretty_json(v: &serde_json::Value) -> String {
        serde_json::to_string_pretty(v).expect("Failed to format JSON")
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
                let schema = &data[suite_id]["schema"];
                let validator = jsonschema::options()
                    .with_draft(Draft::Draft7)
                    .build(schema)
                    .unwrap_or_else(|_| {
                        panic!(
                            "Valid schema. File: {}; Suite ID: {}; Schema: {}",
                            filename, suite_id, schema
                        )
                    });
                for test_data in item["tests"].as_array().expect("Valid array") {
                    let test_id = test_data["id"].as_u64().expect("Is integer") as usize;
                    let mut instance_path = String::new();

                    for segment in test_data["instance_path"]
                        .as_array()
                        .expect("Valid array")
                        .iter()
                    {
                        instance_path.push('/');
                        instance_path.push_str(segment.as_str().expect("A string"));
                    }
                    let instance = &data[suite_id]["tests"][test_id]["data"];
                    let error = validator.validate(instance).expect_err(&format!(
                        "\nFile: {}\nSuite: {}\nTest: {}",
                        filename,
                        &data[suite_id]["description"],
                        &data[suite_id]["tests"][test_id]["description"],
                    ));
                    assert_eq!(
                        error.instance_path.as_str(),
                        instance_path,
                        "\nFile: {}\nSuite: {}\nTest: {}\nError: {}",
                        filename,
                        &data[suite_id]["description"],
                        &data[suite_id]["tests"][test_id]["description"],
                        &error
                    )
                }
            }
        }
    }
}
