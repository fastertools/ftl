package main

import (
	"fmt"
	"log"

	"github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis"
)

func main() {
	// Create the FTL application using the CDK
	cdk := synthesis.NewCDK()
	app := cdk.NewApp("demo-app").
		SetVersion("0.1.0")

	// Add fluid component from registry
	app.AddComponent("fluid").
		FromRegistry("ghcr.io", "bowlofarugula:fluid", "0.0.1").
		Build()

	// Add geo component from registry
	app.AddComponent("geo").
		FromRegistry("ghcr.io", "bowlofarugula:geo", "0.0.1").
		Build()

	// Build and synthesize to spin.toml
	builtCDK := app.Build()
	manifest, err := builtCDK.Synthesize()
	if err != nil {
		log.Fatalf("Failed to synthesize: %v", err)
	}

	// Output the manifest
	fmt.Print(manifest)
}
