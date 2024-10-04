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
        "draft2019-09::optional::ref_of_unknown_keyword::reference_internals_of_known_non_applicator",
        "draft2019-09::r#ref::ref_with_recursive_anchor",
        "draft2019-09::unevaluated_items",
        "draft2019-09::unevaluated_properties::unevaluated_properties_with_recursive_ref",
        "draft2019-09::vocabulary::schema_that_uses_custom_metaschema_with_with_no_validation_vocabulary",
        "draft2020-12::optional::cross_draft::refs_to_historic_drafts_are_processed_as_historic_drafts",
        "draft2020-12::optional::ref_of_unknown_keyword::reference_internals_of_known_non_applicator",
        "draft2020-12::unevaluated_properties::unevaluated_properties_with_dynamic_ref",
        "draft2020-12::unevaluated_items",
        "draft2020-12::vocabulary",
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
    let result = validator.validate(&test.data);

    if test.valid {
        if let Err(mut errors_iterator) = result {
            let first_error = errors_iterator.next();
            assert!(
                first_error.is_none(),
                "Test case should not have validation errors:\nGroup: {}\nTest case: {}\nSchema: {}\nInstance: {}\nError: {:?}",
                test.case,
                test.description,
                test.schema,
                test.data,
                first_error,
            );
        }
        assert!(
            validator.is_valid(&test.data),
            "Test case should be valid:\nCase: {}\nTest: {}\nSchema: {}\nInstance: {}",
            test.case,
            test.description,
            test.schema,
            test.data,
        );
        let output = validator.apply(&test.data).basic();
        assert!(
            output.is_valid(),
            "Test case should be valid via basic output:\nCase: {}\nTest: {}\nSchema: {}\nInstance: {}\nError: {:?}",
            test.case,
            test.description,
            test.schema,
            test.data,
            output
        );
    } else {
        assert!(
            result.is_err(),
            "Test case should have validation errors:\nCase: {}\nTest: {}\nSchema: {}\nInstance: {}",
            test.case,
            test.description,
            test.schema,
            test.data,
        );
        let errors = result.unwrap_err();
        for error in errors {
            let pointer = error.instance_path.to_string();
            assert_eq!(
                test.data.pointer(&pointer), Some(&*error.instance),
                "Expected error instance did not match actual error instance:\nCase: {}\nTest: {}\nSchema: {}\nInstance: {}\nExpected pointer: {:#?}\nActual pointer: {:#?}",
                test.case,
                test.description,
                test.schema,
                test.data,
                &*error.instance,
                &pointer,
            );
        }
        assert!(
            !validator.is_valid(&test.data),
            "Test case should be invalid:\nCase: {}\nTest: {}\nSchema: {}\nInstance: {}",
            test.case,
            test.description,
            test.schema,
            test.data,
        );
        let output = validator.apply(&test.data).basic();
        assert!(
            !output.is_valid(),
            "Test case should be invalid via basic output:\nCase: {}\nTest: {}\nSchema: {}\nInstance: {}",
            test.case,
            test.description,
            test.schema,
            test.data,
        );
    }
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
            let validator = jsonschema::validator_for(schema).unwrap_or_else(|_| {
                panic!(
                    "Valid schema. File: {}; Suite ID: {}; Schema: {}",
                    filename, suite_id, schema
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
                let error = validator
                    .validate(instance)
                    .expect_err(&format!(
                        "\nFile: {}\nSuite: {}\nTest: {}",
                        filename,
                        &data[suite_id]["description"],
                        &data[suite_id]["tests"][test_id]["description"],
                    ))
                    .next()
                    .expect("Validation error");
                assert_eq!(
                    error.instance_path.clone().into_vec(),
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
