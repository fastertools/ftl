//! Build script for generating API client code from `OpenAPI` specification
//!
//! This build script uses the `progenitor` crate to automatically generate
//! a strongly-typed Rust client from the FTL backend `OpenAPI` specification.
//! The generated client provides type-safe access to all FTL API endpoints.

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=backend-openapi.json");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
    let dest_path = Path::new(&out_dir).join("ftl_backend_client.rs");

    let spec_path = "backend-openapi.json";
    let spec_content = fs::read_to_string(spec_path).unwrap_or_else(|e| {
        panic!("Failed to read OpenAPI spec from '{}': {}\n\nEnsure the OpenAPI specification file exists in the project root.", spec_path, e)
    });
    
    let spec: openapiv3::OpenAPI = serde_json::from_str(&spec_content).unwrap_or_else(|e| {
        panic!("Failed to parse OpenAPI spec from '{}': {}\n\nEnsure the file contains valid JSON and follows the OpenAPI 3.x specification.", spec_path, e)
    });

    // Use builder interface style for better ergonomics
    let mut settings = progenitor::GenerationSettings::default();
    settings.with_interface(progenitor::InterfaceStyle::Builder);

    let mut generator = progenitor::Generator::new(&settings);

    let tokens = generator.generate_tokens(&spec).unwrap_or_else(|e| {
        panic!("Failed to generate client code from OpenAPI spec: {}\n\nThis may indicate an issue with the OpenAPI specification format or unsupported features.", e)
    });

    let ast = syn::parse2(tokens).unwrap_or_else(|e| {
        panic!("Failed to parse generated Rust tokens: {}\n\nThis indicates a bug in the code generation process.", e)
    });
    
    let content = prettyplease::unparse(&ast);

    std::fs::write(&dest_path, content).unwrap_or_else(|e| {
        panic!("Failed to write generated client to '{:?}': {}\n\nEnsure the output directory is writable.", dest_path, e)
    });
}
