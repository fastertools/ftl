package main

import (
	"fmt"
	"log"

	"github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis"
)

func main() {
	// Create your FTL application
	app := synthesis.NewApp("demo-app").
		SetDescription("Demo MCP application with geo and fluid tools").
		SetVersion("0.1.0")

	// Add geo and fluid components from registry
	app.AddTool("geo").
		FromRegistry("ghcr.io", "bowlofarugula:geo", "0.0.1").
		Build()
	
	app.AddTool("fluid").
		FromRegistry("ghcr.io", "bowlofarugula:fluid", "0.0.1").
		Build()

	// Enable authentication (optional)
	// app.EnableWorkOSAuth("org_123456")

	// Synthesize to spin.toml
	synth := synthesis.NewSynthesizer()
	manifest, err := synth.SynthesizeApp(app)
	if err != nil {
		log.Fatalf("Failed to synthesize: %v", err)
	}

	// Output the manifest
	fmt.Print(manifest)
}
