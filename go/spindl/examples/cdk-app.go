// Example FTL CDK Application
// This demonstrates how to use the FTL CDK to define an MCP tool platform

package main

import (
	"fmt"
	"log"

	"github.com/fastertools/ftl-cli/go/spindl/pkg/ftl"
)

func main() {
	// Create a new FTL application
	app := ftl.NewApp("my-mcp-platform").
		SetDescription("A powerful MCP tool orchestration platform").
		SetVersion("1.0.0")

	// Add a calculator tool from a local WebAssembly module
	app.AddTool("calculator").
		FromLocal("./tools/calculator.wasm").
		WithBuild("cargo build --target wasm32-wasi --release").
		WithWatch("src/**/*.rs", "Cargo.toml").
		WithEnv("PRECISION", "high").
		WithEnv("MAX_OPERATIONS", "1000").
		Build()

	// Add a weather service from a registry
	app.AddTool("weather").
		FromRegistry("ghcr.io", "mcp-tools/weather", "2.1.0").
		WithEnv("API_KEY", "${WEATHER_API_KEY}").
		WithEnv("CACHE_TTL", "300").
		Build()

	// Add a database connector with build configuration
	app.AddTool("database").
		FromLocal("./tools/database.wasm").
		WithBuild("npm run build:wasm").
		WithWatch("src/**/*.ts", "package.json", "tsconfig.json").
		WithEnv("CONNECTION_POOL_SIZE", "10").
		WithEnv("QUERY_TIMEOUT", "30000").
		Build()

	// Add an AI assistant tool
	app.AddTool("ai-assistant").
		FromRegistry("ghcr.io", "mcp-tools/ai-assistant", "3.0.0").
		WithEnv("MODEL", "gpt-4").
		WithEnv("MAX_TOKENS", "2000").
		WithEnv("TEMPERATURE", "0.7").
		Build()

	// Enable WorkOS authentication for enterprise SSO
	// This will make all tools private and require authentication
	app.EnableWorkOSAuth("org_01HQMQB3Q4BJRWZ8Y6G2XYZABC")

	// Alternative: Enable custom JWT authentication
	// app.EnableCustomAuth("https://auth.example.com", "my-platform")

	// For public access (no authentication), just don't call EnableWorkOSAuth or EnableCustomAuth
	// The default is public access

	// Generate the spin.toml manifest
	synthesizer := ftl.NewSynthesizer()
	
	// Validate the configuration
	if err := synthesizer.Validate(app); err != nil {
		log.Fatalf("Validation failed: %v", err)
	}

	// Synthesize to spin.toml format
	manifest, err := synthesizer.Synthesize(app)
	if err != nil {
		log.Fatalf("Failed to synthesize: %v", err)
	}

	// Output the generated manifest
	fmt.Println(manifest)
}