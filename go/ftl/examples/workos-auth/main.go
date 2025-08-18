package main

import (
	"fmt"
	"log"

	"github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis"
)

func main() {
	// Create FTL application with WorkOS authentication
	cdk := synthesis.NewCDK()

	app := cdk.NewApp("secure-tools").
		SetVersion("1.0.0").
		SetDescription("Secure MCP tools with WorkOS authentication").
		// Enable org-level access - FTL platform handles authentication
		SetOrgAccess()

	// Add secure components
	app.AddComponent("database-admin").
		FromRegistry("ghcr.io", "example:db-admin", "2.0.0").
		WithEnv("LOG_LEVEL", "info").
		Build()

	app.AddComponent("secrets-manager").
		FromRegistry("ghcr.io", "example:secrets", "1.5.0").
		WithEnv("VAULT_ENDPOINT", "https://vault.internal").
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
