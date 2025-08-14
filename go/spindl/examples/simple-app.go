// Simple FTL CDK Example
// Demonstrates the minimal setup for an MCP tool platform

package main

import (
	"fmt"

	"github.com/fastertools/ftl-cli/go/spindl/pkg/ftl"
)

func main() {
	// Create a simple MCP platform with two tools
	app := ftl.NewApp("simple-platform").
		SetDescription("A simple MCP tool platform").
		SetVersion("0.1.0")

	// Add a calculator tool
	app.AddTool("calculator").
		FromLocal("./calculator.wasm").
		WithEnv("LOG_LEVEL", "info").
		Build()

	// Add a weather tool from a registry
	app.AddTool("weather").
		FromRegistry("ghcr.io", "mcp-tools/weather", "1.0.0").
		Build()

	// Keep it public (no authentication)
	// This means the MCP gateway will be the public entry point

	// Generate the manifest
	synth := ftl.NewSynthesizer()
	manifest, _ := synth.SynthesizeApp(app)
	fmt.Println(manifest)
}