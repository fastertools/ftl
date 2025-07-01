use std::path::Path;

use anyhow::Result;
use handlebars::Handlebars;
use serde_json::json;

pub fn create_tool(name: &str, description: &str, target_dir: &Path) -> Result<()> {
    // Create directory structure
    std::fs::create_dir_all(target_dir)?;
    std::fs::create_dir_all(target_dir.join("src"))?;

    // Convert tool name to struct name (PascalCase)
    let struct_name = name
        .split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<String>();

    let handlebars = Handlebars::new();
    let data = json!({
        "tool_name": name,
        "struct_name": struct_name,
        "description": description,
        "version": "0.0.1",
    });

    // Create ftl.toml
    let ftl_toml_template = r#"[tool]
name = "{{tool_name}}"
version = "{{version}}"
description = "{{description}}"

[build]
profile = "release"
features = []

[optimization]
# See https://github.com/WebAssembly/binaryen/blob/version_123/test/lit/help/wasm-opt.test
flags = [
    "-O4",
    "-Oz",
]

[runtime]
# List of external hosts this tool is allowed to make HTTP requests to.
# Use exact hostnames or patterns with wildcards (e.g., "*.googleapis.com").
# Leave empty to deny all external requests.
allowed_hosts = []
"#;

    let ftl_toml = handlebars.render_template(ftl_toml_template, &data)?;
    std::fs::write(target_dir.join("ftl.toml"), ftl_toml)?;

    // Get the SDK version from compile-time constant
    let sdk_version = env!("FTL_SDK_RS_VERSION");
    let ftl_sdk_dep = format!("{{ version = \"^{sdk_version}\" }}");

    // Create Cargo.toml
    let cargo_toml_template = format!(
        r#"[package]
name = "{{{{tool_name}}}}"
version = "{{{{version}}}}"
edition = "2024"

[dependencies]
ftl-sdk-rs = {ftl_sdk_dep}
talc = {{ version = "4.4.3", default-features = false }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
anyhow = "1.0"
spin-sdk = "3.1.1"

[dev-dependencies]

[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = "symbols"

[profile.dev]
opt-level = 1
debug = true

[workspace]
"#
    );

    let cargo_toml = handlebars.render_template(&cargo_toml_template, &data)?;
    std::fs::write(target_dir.join("Cargo.toml"), cargo_toml)?;

    // Create src/lib.rs
    let lib_rs_template = r#"use ftl_sdk_rs::prelude::*;
use serde_json::json;

// --- Global Memory Allocator ---
// FTL tools are compiled to WebAssembly, which requires a global memory
// allocator to be defined. The `talc` allocator is used here with a
// statically-allocated arena.
//
// The size of this arena is a critical trade-off:
//   - Larger Arena: Supports more memory-intensive tools.
//   - Smaller Arena: Results in a smaller .wasm binary size, which can improve
//     cold start times.
//
// You can adjust the `ARENA_SIZE` constant below to fit your tool's specific
// memory requirements.
#[cfg(target_family = "wasm")]
#[global_allocator]
static ALLOC: talc::Talck<talc::locking::AssumeUnlockable, talc::ClaimOnOom> = {
    use talc::*;
    // Choose an arena size based on your tool's needs.
    // - 64 * 1024 (64KB): For minimal tools (e.g., simple text processing).
    // - 1 * 1024 * 1024 (1MB): A good default for most tools.
    // - 4 * 1024 * 1024 (4MB): For data-intensive tools (e.g., image processing).
    const ARENA_SIZE: usize = 1 * 1024 * 1024; // Default: 1MB
    static mut ARENA: [u8; ARENA_SIZE] = [0; ARENA_SIZE];
    Talc::new(unsafe {
        ClaimOnOom::new(Span::from_base_size(
            std::ptr::addr_of_mut!(ARENA).cast(),
            ARENA_SIZE,
        ))
    })
    .lock()
};

#[derive(Clone)]
pub struct {{struct_name}};

impl Tool for {{struct_name}} {
    fn name(&self) -> &'static str {
        "{{tool_name}}"
    }

    fn description(&self) -> &'static str {
        "{{description}}"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Input to process"
                }
            },
            "required": ["input"]
        })
    }

    fn call(&self, args: &serde_json::Value) -> Result<ToolResult, ToolError> {
        let input = args["input"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArguments("input is required".to_string()))?;

        // TODO: Implement your tool logic here
        let result = format!("Processed: {input}");

        Ok(ToolResult::text(result))
    }
}

ftl_sdk_rs::ftl_mcp_server!({{struct_name}});

#[cfg(test)]
mod tests;
"#;

    let lib_rs = handlebars.render_template(lib_rs_template, &data)?;
    std::fs::write(target_dir.join("src").join("lib.rs"), lib_rs)?;

    // Create src/tests.rs
    let tests_rs_template = r#"use ftl_sdk_rs::prelude::*;
use serde_json::json;

use super::{{struct_name}};

#[test]
fn test_tool_metadata() {
    let tool = {{struct_name}};
    assert_eq!(tool.name(), "{{tool_name}}");
    assert_eq!(tool.description(), "{{description}}");
}

#[test]
fn test_tool_call() {
    let tool = {{struct_name}};
    let args = json!({
        "input": "test input"
    });

    let result = tool.call(&args).unwrap();
    // Result is a text block
    assert!(!result.content.is_empty());
    let text = &result.content[0].text;
    assert!(text.contains("test input"));
}
"#;

    let tests_rs = handlebars.render_template(tests_rs_template, &data)?;
    std::fs::write(target_dir.join("src").join("tests.rs"), tests_rs)?;

    // Create README.md
    let readme_template = r#"# {{tool_name}}

{{description}}

## Usage

This tool is designed to be used with the Model Context Protocol (MCP).

### Development

```bash
# Serve locally for testing
ftl serve {{tool_name}}

# Build the tool
ftl build {{tool_name}}

# Run tests (requires spin test setup)
ftl test {{tool_name}}
```

### Input Schema

```json
{
  "type": "object",
  "properties": {
    "input": {
      "type": "string",
      "description": "Input to process"
    }
  },
  "required": ["input"]
}
```

### Example

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "{{tool_name}}",
      "arguments": {
        "input": "Hello, world!"
      }
    },
    "id": 1
  }'
```

## Configuration

Edit `ftl.toml` to configure build and runtime settings.

## License

Apache-2.0
"#;

    let readme = handlebars.render_template(readme_template, &data)?;
    std::fs::write(target_dir.join("README.md"), readme)?;

    // Create .gitignore
    let gitignore = r#"target/
Cargo.lock
.ftl/
*.wasm
"#;
    std::fs::write(target_dir.join(".gitignore"), gitignore)?;

    // Create rustfmt.toml
    let rustfmt_toml = r#"# Rust formatting configuration
edition = "2024"
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
reorder_imports = true
reorder_modules = true
remove_nested_parens = true
use_field_init_shorthand = true
use_try_shorthand = true
format_code_in_doc_comments = true
normalize_comments = true
normalize_doc_attributes = true
format_strings = true
format_macro_matchers = true
format_macro_bodies = true
empty_item_single_line = true
struct_lit_single_line = true
fn_single_line = false
where_single_line = false
imports_indent = "Block"
imports_layout = "Mixed"
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
reorder_impl_items = false
type_punctuation_density = "Wide"
space_before_colon = false
space_after_colon = true
spaces_around_ranges = false
binop_separator = "Front"
combine_control_expr = true
overflow_delimited_expr = false
struct_field_align_threshold = 0
enum_discrim_align_threshold = 0
match_arm_blocks = true
match_arm_leading_pipes = "Never"
force_multiline_blocks = false
fn_params_layout = "Tall"
brace_style = "SameLineWhere"
control_brace_style = "AlwaysSameLine"
trailing_semicolon = true
trailing_comma = "Vertical"
match_block_trailing_comma = false
blank_lines_upper_bound = 1
blank_lines_lower_bound = 0
merge_derives = true
wrap_comments = true
comment_width = 80
format_generated_files = false
skip_children = false
"#;
    std::fs::write(target_dir.join("rustfmt.toml"), rustfmt_toml)?;

    Ok(())
}
