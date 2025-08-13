use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("spinc").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Infrastructure as Code for WebAssembly"));
}

#[test]
fn test_init_command() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("spinc").unwrap();
    
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .arg("test-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initializing"));
    
    // Check that spinc.yaml was created
    assert!(temp_dir.path().join("spinc.yaml").exists());
    
    let content = fs::read_to_string(temp_dir.path().join("spinc.yaml")).unwrap();
    assert!(content.contains("name: test-app"));
    assert!(content.contains("template: mcp"));
}

#[test]
fn test_validate_valid_config() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a valid config
    let config = r#"
name: test-app
template: mcp
auth:
  enabled: false
"#;
    
    let config_path = temp_dir.path().join("spinc.yaml");
    fs::write(&config_path, config).unwrap();
    
    let mut cmd = Command::cargo_bin("spinc").unwrap();
    cmd.arg("validate")
        .arg(&config_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn test_list_constructs() {
    let mut cmd = Command::cargo_bin("spinc").unwrap();
    cmd.arg("construct")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("mcp"))
        .stdout(predicate::str::contains("L3 - Solutions"));
}