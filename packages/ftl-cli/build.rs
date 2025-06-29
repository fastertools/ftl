use std::{env, fs, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=../../Cargo.toml");
    println!("cargo:rerun-if-changed=../ftl-sdk-js/package.json");

    // Read ftl-sdk-rs version from workspace Cargo.toml
    let workspace_toml_path = Path::new("../../Cargo.toml");
    let ftl_sdk_rs_version = if workspace_toml_path.exists() {
        let content = fs::read_to_string(workspace_toml_path).unwrap();
        let workspace_toml: toml::Value = toml::from_str(&content).unwrap();

        workspace_toml
            .get("workspace")
            .and_then(|w| w.get("package"))
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.9")
            .to_string()
    } else {
        // Fallback for when building outside the workspace
        env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.9".to_string())
    };

    // Read @fastertools/ftl-sdk-js version from package.json
    let sdk_js_package_json = Path::new("../ftl-sdk-js/package.json");
    let ftl_sdk_js_version = if sdk_js_package_json.exists() {
        let content = fs::read_to_string(sdk_js_package_json).unwrap();
        let package_json: serde_json::Value = serde_json::from_str(&content).unwrap();

        package_json
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.9")
            .to_string()
    } else {
        // Use same version as Rust SDK as fallback
        ftl_sdk_rs_version.clone()
    };

    // Set environment variables that will be available at compile time
    println!("cargo:rustc-env=FTL_SDK_RS_VERSION={}", ftl_sdk_rs_version);
    println!("cargo:rustc-env=FTL_SDK_JS_VERSION={}", ftl_sdk_js_version);
}
