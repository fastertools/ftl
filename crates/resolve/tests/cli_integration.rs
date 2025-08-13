//! CLI integration tests

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_spin_from_file() {
    let mut file = NamedTempFile::new().unwrap();
    write!(
        file,
        r#"
[project]
name = "test-app"
version = "1.0.0"

[mcp]
gateway = "ghcr.io/fastertools/mcp-gateway:0.0.11"
authorizer = "ghcr.io/fastertools/mcp-authorizer:0.0.14"
"#
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("ftl-resolve").unwrap();
    cmd.arg("spin")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("spin_manifest_version = 2"))
        .stdout(predicate::str::contains("[application]"))
        .stdout(predicate::str::contains("name = \"test-app\""));
}

#[test]
fn test_spin_from_stdin() {
    let input = r#"
[project]
name = "stdin-app"
version = "2.0.0"

[mcp]
gateway = "gateway.wasm"
authorizer = "authorizer.wasm"
"#;

    let mut cmd = Command::cargo_bin("ftl-resolve").unwrap();
    cmd.arg("spin")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("name = \"stdin-app\""))
        .stdout(predicate::str::contains("version = \"2.0.0\""));
}

#[test]
fn test_spin_json_input() {
    let json = r#"{
        "project": {
            "name": "json-app",
            "version": "1.0.0"
        },
        "mcp": {
            "gateway": "gateway.wasm",
            "authorizer": "authorizer.wasm"
        },
        "oauth": null,
        "component": {},
        "variables": {}
    }"#;

    let mut cmd = Command::cargo_bin("ftl-resolve").unwrap();
    cmd.arg("spin")
        .arg("-")
        .arg("-f")
        .arg("json")
        .write_stdin(json)
        .assert()
        .success()
        .stdout(predicate::str::contains("name = \"json-app\""));
}

#[test]
fn test_schema_generation() {
    let mut cmd = Command::cargo_bin("ftl-resolve").unwrap();
    cmd.arg("schema")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"$schema\""))
        .stdout(predicate::str::contains("\"title\": \"FtlConfig\""))
        .stdout(predicate::str::contains("\"properties\""));
}

#[test]
fn test_schema_to_file() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("ftl-resolve").unwrap();
    cmd.arg("schema")
        .arg("-o")
        .arg(path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Successfully wrote schema"));

    // Verify file was created and contains valid JSON
    let content = std::fs::read_to_string(path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(json["title"], "FtlConfig");
}

#[test]
fn test_validate_valid_config() {
    let mut file = NamedTempFile::new().unwrap();
    write!(
        file,
        r#"
[project]
name = "valid-app"
version = "1.0.0"

[mcp]
gateway = "gateway.wasm"
authorizer = "authorizer.wasm"
"#
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("ftl-resolve").unwrap();
    cmd.arg("validate")
        .arg(file.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Configuration is valid"));
}

#[test]
fn test_validate_invalid_config() {
    let mut file = NamedTempFile::new().unwrap();
    write!(
        file,
        r#"
[project]
# Missing required 'name' field
version = "1.0.0"
"#
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("ftl-resolve").unwrap();
    cmd.arg("validate")
        .arg(file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to parse TOML"));
}

#[test]
fn test_public_mode_spin() {
    let input = r#"
[project]
name = "public-app"

[mcp]
gateway = "gateway.wasm"
authorizer = "authorizer.wasm"
"#;

    let mut cmd = Command::cargo_bin("ftl-resolve").unwrap();
    cmd.arg("spin")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        // In public mode, gateway should be named "mcp"
        .stdout(predicate::str::contains("[component.mcp]"))
        // Should not have ftl-mcp-gateway
        .stdout(predicate::str::contains("[component.ftl-mcp-gateway]").not());
}

#[test]
fn test_auth_mode_spin() {
    let input = r#"
[project]
name = "auth-app"

[oauth]
issuer = "https://auth.example.com"
audience = ["api-1", "api-2"]

[mcp]
gateway = "gateway.wasm"
authorizer = "authorizer.wasm"
"#;

    let mut cmd = Command::cargo_bin("ftl-resolve").unwrap();
    cmd.arg("spin")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        // In auth mode, should have both components
        .stdout(predicate::str::contains("[component.mcp]"))
        .stdout(predicate::str::contains("[component.ftl-mcp-gateway]"))
        // Should have auth variables
        .stdout(predicate::str::contains("mcp_jwt_issuer"))
        .stdout(predicate::str::contains("mcp_jwt_audience = { default = \"api-1,api-2\" }"));
}

#[test]
fn test_component_with_build() {
    let input = r#"
[project]
name = "build-app"

[mcp]
gateway = "gateway.wasm"
authorizer = "authorizer.wasm"

[component.my-tool]
wasm = "my-tool.wasm"

[component.my-tool.build]
command = "cargo component build --release"
watch = ["src/**/*.rs", "Cargo.toml"]
"#;

    let mut cmd = Command::cargo_bin("ftl-resolve").unwrap();
    cmd.arg("spin")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("[component.my-tool]"))
        .stdout(predicate::str::contains("[component.my-tool.build]"))
        .stdout(predicate::str::contains(
            "command = \"cargo component build --release\"",
        ))
        .stdout(predicate::str::contains(
            "watch = [\"src/**/*.rs\", \"Cargo.toml\"]",
        ));
}
