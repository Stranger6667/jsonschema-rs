use assert_cmd::Command;
use insta::assert_snapshot;
use std::fs;
use tempfile::tempdir;

fn cli() -> Command {
    Command::cargo_bin("jsonschema-cli").unwrap()
}

fn create_temp_file(dir: &tempfile::TempDir, name: &str, content: &str) -> String {
    let file_path = dir.path().join(name);
    fs::write(&file_path, content).unwrap();
    file_path.to_str().unwrap().to_string()
}

fn sanitize_output(output: String, file_names: &[&str]) -> String {
    let mut sanitized = output;
    for (i, name) in file_names.iter().enumerate() {
        sanitized = sanitized.replace(name, &format!("{{FILE_{}}}", i + 1));
    }
    sanitized
}

#[test]
fn test_version() {
    let mut cmd = cli();
    cmd.arg("--version");
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

#[test]
fn test_valid_instance() {
    let dir = tempdir().unwrap();
    let schema = create_temp_file(
        &dir,
        "schema.json",
        r#"{"type": "object", "properties": {"name": {"type": "string"}}}"#,
    );
    let instance = create_temp_file(&dir, "instance.json", r#"{"name": "John Doe"}"#);

    let mut cmd = cli();
    cmd.arg(&schema).arg("--instance").arg(&instance);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let sanitized = sanitize_output(
        String::from_utf8_lossy(&output.stdout).to_string(),
        &[&instance],
    );
    assert_snapshot!(sanitized);
}

#[test]
fn test_invalid_instance() {
    let dir = tempdir().unwrap();
    let schema = create_temp_file(
        &dir,
        "schema.json",
        r#"{"type": "object", "properties": {"name": {"type": "string"}}}"#,
    );
    let instance = create_temp_file(&dir, "instance.json", r#"{"name": 123}"#);

    let mut cmd = cli();
    cmd.arg(&schema).arg("--instance").arg(&instance);
    let output = cmd.output().unwrap();
    assert!(!output.status.success());
    let sanitized = sanitize_output(
        String::from_utf8_lossy(&output.stdout).to_string(),
        &[&instance],
    );
    assert_snapshot!(sanitized);
}

#[test]
fn test_invalid_schema() {
    let dir = tempdir().unwrap();
    let schema = create_temp_file(&dir, "schema.json", r#"{"type": "invalid"}"#);
    let instance = create_temp_file(&dir, "instance.json", r#"{}"#);

    let mut cmd = cli();
    cmd.arg(&schema).arg("--instance").arg(&instance);
    let output = cmd.output().unwrap();
    assert!(!output.status.success());
    let sanitized = sanitize_output(
        String::from_utf8_lossy(&output.stdout).to_string(),
        &[&instance],
    );
    assert_snapshot!(sanitized);
}

#[test]
fn test_multiple_instances() {
    let dir = tempdir().unwrap();
    let schema = create_temp_file(
        &dir,
        "schema.json",
        r#"{"type": "object", "properties": {"name": {"type": "string"}}}"#,
    );
    let instance1 = create_temp_file(&dir, "instance1.json", r#"{"name": "John Doe"}"#);
    let instance2 = create_temp_file(&dir, "instance2.json", r#"{"name": 123}"#);

    let mut cmd = cli();
    cmd.arg(&schema)
        .arg("--instance")
        .arg(&instance1)
        .arg("--instance")
        .arg(&instance2);
    let output = cmd.output().unwrap();
    assert!(!output.status.success());
    let sanitized = sanitize_output(
        String::from_utf8_lossy(&output.stdout).to_string(),
        &[&instance1, &instance2],
    );
    assert_snapshot!(sanitized);
}

#[test]
fn test_no_instances() {
    let dir = tempdir().unwrap();
    let schema = create_temp_file(&dir, "schema.json", r#"{"type": "object"}"#);

    let mut cmd = cli();
    cmd.arg(&schema);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}
