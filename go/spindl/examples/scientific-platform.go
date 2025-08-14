// Scientific Computing Platform Example
// This example shows how to build an MCP tool platform with geo and fluid dynamics tools

package main

import (
	"fmt"
	"log"

	"github.com/fastertools/ftl-cli/go/spindl/pkg/ftl"
)

func main() {
	// Create a new FTL application for scientific computing
	app := ftl.NewApp("scientific-platform").
		SetDescription("Scientific computing platform with geo and fluid dynamics tools").
		SetVersion("1.0.0")

	// Add the geo tool from the registry
	// This tool might handle geological computations, mapping, GIS operations, etc.
	app.AddTool("geo").
		FromRegistry("ghcr.io", "bowlofarugula/geo", "0.0.1").
		WithEnv("LOG_LEVEL", "info").
		WithEnv("MAX_MEMORY", "2048").
		WithEnv("COMPUTE_THREADS", "4").
		Build()

	// Add the fluid dynamics tool
	// This tool might handle fluid simulations, CFD computations, etc.
	app.AddTool("fluid").
		FromRegistry("ghcr.io", "bowlofarugula/fluid", "0.0.1").
		WithEnv("PRECISION", "double").
		WithEnv("SOLVER_TYPE", "SIMPLE").
		WithEnv("MAX_ITERATIONS", "1000").
		Build()

	// You could also add a local tool that you're developing
	// For example, a visualization tool that works with both geo and fluid data
	app.AddTool("visualizer").
		FromLocal("./visualizer.wasm").
		WithBuild("cargo build --target wasm32-wasi --release").
		WithWatch("src/**/*.rs", "Cargo.toml").
		WithEnv("RENDER_ENGINE", "webgl").
		WithEnv("COLOR_SCHEME", "scientific").
		Build()

	// Enable authentication if you want to secure your platform
	// This uses WorkOS for enterprise SSO
	app.EnableWorkOSAuth("org_scientific_12345")

	// Alternative: Use custom JWT authentication
	// app.EnableCustomAuth("https://auth.yourcompany.com", "scientific-platform")

	// For public access (no authentication), you don't need to call any auth method
	// The default is public access

	// Generate the spin.toml manifest
	synth := ftl.NewSynthesizer()
	manifest, err := synth.SynthesizeApp(app)
	if err != nil {
		log.Fatalf("Failed to synthesize: %v", err)
	}

	fmt.Println("Generated spin.toml:")
	fmt.Println("==================================================")
	fmt.Println(manifest)
}