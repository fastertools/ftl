package main

import (
	"fmt"
	"log"

	"github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis"
)

func main() {
	// Create FTL application with custom JWT authentication
	cdk := synthesis.NewCDK()

	app := cdk.NewApp("enterprise-tools").
		SetVersion("2.0.0").
		SetDescription("Enterprise MCP tools with custom JWT authentication").
		// Enable custom JWT authentication
		SetCustomAuth("https://auth.example.com", "enterprise-mcp-tools")

	// Add enterprise components
	app.AddComponent("compliance-scanner").
		FromRegistry("ghcr.io", "enterprise:compliance", "3.0.0").
		WithEnv("SCAN_DEPTH", "full").
		WithEnv("REPORT_FORMAT", "json").
		Build()

	app.AddComponent("audit-logger").
		FromRegistry("ghcr.io", "enterprise:audit", "2.5.0").
		WithEnv("LOG_RETENTION_DAYS", "90").
		WithEnv("ENCRYPTION_ENABLED", "true").
		Build()

	app.AddComponent("policy-engine").
		FromRegistry("ghcr.io", "enterprise:policy", "4.1.0").
		WithEnv("POLICY_SOURCE", "https://policies.example.com/v1").
		WithEnv("ENFORCEMENT_MODE", "strict").
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
