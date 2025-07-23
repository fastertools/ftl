//! Build script for generating API client code from `OpenAPI` specification
//!
//! This build script uses the `progenitor` crate to automatically generate
//! a strongly-typed Rust client from the FTL backend `OpenAPI` specification.
//! The generated client provides type-safe access to all FTL API endpoints.

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=../../backend-openapi.json");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("ftl_backend_client.rs");

    let spec_path = "../../backend-openapi.json";
    let spec_content = fs::read_to_string(spec_path).expect("Failed to read OpenAPI spec");
    let spec: openapiv3::OpenAPI =
        serde_json::from_str(&spec_content).expect("Failed to parse OpenAPI spec");

    // Use builder interface style for better ergonomics
    let mut settings = progenitor::GenerationSettings::default();
    settings.with_interface(progenitor::InterfaceStyle::Builder);

    let mut generator = progenitor::Generator::new(&settings);

    let tokens = generator
        .generate_tokens(&spec)
        .expect("Failed to generate client code");

    let ast = syn::parse2(tokens).expect("Failed to parse generated tokens");
    let content = prettyplease::unparse(&ast);

    std::fs::write(dest_path, content).expect("Failed to write generated client");
}
