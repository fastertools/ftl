// Simple Public Platform Example
// Minimal example showing just the geo and fluid tools with public access

package main

import (
	"fmt"
	"log"

	"github.com/fastertools/ftl-cli/go/spindl/pkg/ftl"
)

func main() {
	// Create a simple public platform
	app := ftl.NewApp("compute-tools").
		SetDescription("Public geo and fluid computation tools").
		SetVersion("0.1.0")

	// Add your two tools
	app.AddTool("geo").
		FromRegistry("ghcr.io", "bowlofarugula/geo", "0.0.1").
		Build()

	app.AddTool("fluid").
		FromRegistry("ghcr.io", "bowlofarugula/fluid", "0.0.1").
		Build()

	// That's it! Generate the manifest
	synth := ftl.NewSynthesizer()
	manifest, err := synth.SynthesizeApp(app)
	if err != nil {
		log.Fatalf("Failed to synthesize: %v", err)
	}

	fmt.Println(manifest)
}