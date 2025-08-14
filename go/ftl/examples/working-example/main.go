// Complete Working Example
// This CDK app creates a real MCP platform you can run

package main

import (
	"fmt"
	"log"

	"github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis"
)

func main() {
	// Create a working MCP platform
	app := synthesis.NewApp("demo-platform").
		SetDescription("Demo MCP Platform - Ready to Run").
		SetVersion("1.0.0")

	// Add a simple HTTP responder tool
	// You'll need to create a simple WASM file or use an existing one
	app.AddTool("hello").
		FromLocal("./hello.wasm"). // You'll need this file
		WithEnv("MESSAGE", "Hello from MCP!").
		Build()

	// Add another tool from a registry (if available)
	// Or comment this out if you don't have access to the registry
	app.AddTool("echo").
		FromRegistry("ghcr.io", "fastertools/echo", "latest").
		WithEnv("LOG_LEVEL", "debug").
		Build()

	// Keep it public for easy testing
	// (No authentication required)

	// Generate the manifest
	synth := synthesis.NewSynthesizer()

	// Generate spin.toml (validation happens during synthesis)
	manifest, err := synth.SynthesizeApp(app)
	if err != nil {
		log.Fatalf("Synthesis failed: %v", err)
	}

	fmt.Println(manifest)
}
