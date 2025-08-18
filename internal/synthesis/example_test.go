package synthesis

import (
	"fmt"
)

func Example_simpleApp() {
	// Create a new FTL application using the CDK
	cdk := NewCDK()
	app := cdk.NewApp("my-mcp-platform").
		SetDescription("An MCP tool orchestration platform").
		SetVersion("1.0.0")

	// Add a local component with build config
	app.AddComponent("calculator").
		FromLocal("./tools/calc.wasm").
		WithBuild("cargo build --release").
		WithWatch("src/**/*.rs", "Cargo.toml").
		Build()

	// Add a component from registry
	app.AddComponent("weather").
		FromRegistry("ghcr.io", "example/weather", "2.0.0").
		WithEnv("API_KEY", "secret-key").
		Build()

	// Add another component
	app.AddComponent("translator").
		FromLocal("./tools/translator.wasm").
		Build()

	// Enable organization-level authentication
	app.SetOrgAccess()

	// Build the application
	builtCDK := app.Build()

	// Generate the CUE representation
	cue, _ := builtCDK.ToCUE()
	fmt.Println("Generated CUE:")
	fmt.Println(cue)

	// Synthesize to final spin.toml
	manifest, _ := builtCDK.Synthesize()
	fmt.Println("\nFinal spin.toml:")
	fmt.Println(manifest)
}

func Example_publicApp() {
	// Simple public app with no auth
	cdk := NewCDK()
	app := cdk.NewApp("public-tools").
		SetAccess("public")

	app.AddComponent("echo").
		FromLocal("./echo.wasm").
		Build()

	builtCDK := app.Build()
	manifest, _ := builtCDK.Synthesize()
	fmt.Println(manifest)
}

func Example_customAuth() {
	cdk := NewCDK()
	app := cdk.NewApp("enterprise-mcp")

	// Add components
	app.AddComponent("database-admin").
		FromRegistry("internal.company.com", "tools/db-admin", "1.0.0").
		Build()

	// Enable custom auth
	app.SetCustomAuth(
		"https://auth.company.com",
		"mcp-platform",
	)

	builtCDK := app.Build()
	manifest, _ := builtCDK.Synthesize()
	fmt.Println(manifest)
}

func Example_directYAML() {
	// You can also synthesize directly from YAML
	yamlConfig := `
application:
  name: yaml-app
  version: 1.0.0
components:
  - id: component1
    source: ./component1.wasm
`

	synth := NewSynthesizer()
	manifest, _ := synth.SynthesizeYAML([]byte(yamlConfig))
	fmt.Println(manifest)
}

func Example_directCUE() {
	// Or directly from CUE source
	cueSource := `
app: {
	name: "cue-app"
	version: "1.0.0"
	components: [{
		id: "mycomponent"
		source: "./mycomponent.wasm"
	}]
}
`

	synth := NewSynthesizer()
	manifest, _ := synth.SynthesizeCUE(cueSource)
	fmt.Println(manifest)
}
