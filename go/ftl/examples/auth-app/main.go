// FTL CDK Example with Authentication
// Shows how to create a private MCP platform with WorkOS SSO

package main

import (
	"fmt"

	"github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis"
)

func main() {
	// Create an enterprise MCP platform
	app := synthesis.NewApp("enterprise-platform").
		SetDescription("Enterprise MCP Tool Platform with SSO").
		SetVersion("2.0.0")

	// Add multiple tools with different configurations
	app.AddTool("code-analyzer").
		FromLocal("./analyzer.wasm").
		WithBuild("cargo build --release").
		WithWatch("src/**/*.rs").
		WithEnv("ANALYSIS_DEPTH", "deep").
		WithEnv("TIMEOUT_SECONDS", "60").
		Build()

	app.AddTool("database-admin").
		FromRegistry("ghcr.io", "enterprise/db-admin", "4.2.0").
		WithEnv("MAX_CONNECTIONS", "50").
		WithEnv("ENABLE_AUDIT", "true").
		Build()

	app.AddTool("security-scanner").
		FromLocal("./scanner.wasm").
		WithBuild("make build-wasm").
		WithEnv("SCAN_MODE", "comprehensive").
		Build()

	// Enable WorkOS authentication
	// This makes all tools private and adds an authorizer
	app.EnableWorkOSAuth("org_01HQMQB3Q4BJRWZ8Y6G2XYZABC")

	// Generate the manifest
	synth := synthesis.NewSynthesizer()
	manifest, _ := synth.SynthesizeApp(app)
	fmt.Println(manifest)
}
