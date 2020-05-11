use heck::SnakeCase;
use proc_macro::TokenStream;
use serde_json::{from_str, Value};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[proc_macro]
pub fn test_draft(input: TokenStream) -> TokenStream {
    let dir_name = input.to_string();
    let dir = Path::new(dir_name.trim_start_matches('"').trim_end_matches('"'));
    let draft = dir
        .to_str()
        .unwrap()
        .trim_end_matches('/')
        .split('/')
        .last()
        .unwrap()
        .to_string();
    let tests = load_tests(dir, format!("{}_", draft));
    let mut output = "".to_string();
    for (file_name, test) in tests {
        for (i, suite) in test.as_array().unwrap().iter().enumerate() {
            let schema = suite.get("schema").unwrap();
            let tests = suite.get("tests").unwrap().as_array().unwrap();
            for (j, test) in tests.iter().enumerate() {
                let description = test.get("description").unwrap().as_str().unwrap();
                let data = test.get("data").unwrap();
                let valid = test.get("valid").unwrap().as_bool().unwrap();
                output.push_str("\n#[test]\n");
                output.push_str(&format!("fn {}_{}_{}()", file_name, i, j));
                output.push_str(&make_fn_body(schema, data, &description, valid))
            }
        }
    }
    output.parse().unwrap()
}

fn load_tests(dir: &Path, prefix: String) -> Vec<(String, Value)> {
    let content = fs::read_dir(dir).unwrap();
    let mut tests = vec![];
    for entry in content {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            let mut prefix = prefix.clone();
            prefix.push_str(path.to_str().unwrap().split('/').last().unwrap());
            prefix.push('_');
            let more = load_tests(&path, prefix);
            tests.extend(more)
        } else {
            let mut file = File::open(&path).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).ok().unwrap();
            let data: Value = from_str(&content).unwrap();
            let filename = path.file_name().unwrap().to_str().unwrap();
            tests.push((
                format!(
                    "{}{}",
                    prefix,
                    filename[..filename.len() - 5].to_owned().to_snake_case()
                ),
                data,
            ))
        }
    }
    tests
}

fn make_fn_body(schema: &Value, data: &Value, description: &str, valid: bool) -> String {
    let mut output = "{".to_string();
    output.push_str(&format!(
        r###"
    let schema_str = r##"{}"##;
    let schema: serde_json::Value = serde_json::from_str(schema_str).unwrap();
    let data_str = r##"{}"##;
    let data: serde_json::Value = serde_json::from_str(data_str).unwrap();
    let description = r#"{}"#;
    println!("Description: {{}}", description);
    let compiled = jsonschema::JSONSchema::compile(&schema, None).unwrap();
    let result = compiled.validate(&data);
    assert_eq!(result.is_ok(), compiled.is_valid(&data));
    "###,
        schema.to_string(),
        data.to_string(),
        description
    ));
    if valid {
        output.push_str(
            r#"
        let err = result.err();
        let errors = err.iter().collect::<Vec<_>>();
        if !errors.is_empty() {
            let message = format!(
                "Schema: {}\nInstance: {}\nError: {:?}",
                schema, data, 1
            );
            assert!(false, message)
        }
            "#,
        )
    } else {
        output.push_str(r#"assert!(result.is_err(), "It should be INVALID!");"#)
    }
    output.push_str("}");
    output
}
