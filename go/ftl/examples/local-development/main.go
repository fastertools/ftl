package main

import (
	"fmt"
	"log"

	"github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis"
)

func main() {
	// Create a development workspace with mixed local and registry components
	cdk := synthesis.NewCDK()

	app := cdk.NewApp("dev-workspace").
		SetVersion("0.1.0").
		SetDescription("Local development workspace with build configurations").
		SetAccess("public") // Public for local development

	// Add local Rust component with build configuration
	app.AddComponent("my-rust-tool").
		FromLocal("./rust-tool/target/wasm32-wasi/release/rust_tool.wasm").
		WithBuild("cargo build --target wasm32-wasi --release").
		WithWatch("src/**/*.rs", "Cargo.toml").
		WithEnv("DEBUG_MODE", "true").
		WithEnv("LOG_LEVEL", "debug").
		Build()

	// Add local TypeScript component
	app.AddComponent("my-ts-tool").
		FromLocal("./ts-tool/dist/component.wasm").
		WithBuild("npm run build").
		WithWatch("src/**/*.ts", "package.json").
		WithEnv("API_ENDPOINT", "http://localhost:8080").
		Build()

	// Mix with a registry component
	app.AddComponent("calculator").
		FromRegistry("ghcr.io", "example:calculator", "1.0.0").
		Build()

	// Build and synthesize
	builtCDK := app.Build()
	manifest, err := builtCDK.Synthesize()
	if err != nil {
		log.Fatalf("Failed to synthesize: %v", err)
	}

	// Output the manifest
	fmt.Print(manifest)
}
