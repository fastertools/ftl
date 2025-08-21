// +build ignore

package main

import (
    "fmt"
    "os"
    
    "github.com/fastertools/ftl/go/spindl/pkg/ftl"
)

func main() {
    // Create a new FTL application
    app := ftl.NewApp("calculator-platform").
        SetDescription("A platform for mathematical computations").
        SetVersion("2.0.0")
    
    // Add a calculator tool from local source
    app.AddTool("calculator").
        FromLocal("./components/calculator/calculator.wasm").
        WithBuild("cargo build --target wasm32-wasi --release").
        WithWatch("src/**/*.rs", "Cargo.toml").
        WithEnv("LOG_LEVEL", "debug").
        Build()
    
    // Add a statistics tool from registry
    app.AddTool("statistics").
        FromRegistry("ghcr.io", "mathtools/statistics", "1.5.0").
        WithEnv("PRECISION", "high").
        Build()
    
    // Add a graphing tool
    app.AddTool("grapher").
        FromLocal("./components/grapher/grapher.wasm").
        WithBuild("npm run build:wasm").
        WithWatch("src/**/*.ts", "package.json").
        Build()
    
    // Enable authentication for production
    if os.Getenv("ENVIRONMENT") == "production" {
        app.EnableWorkOSAuth(os.Getenv("WORKOS_ORG_ID"))
    } else {
        app.SetAccess(ftl.PublicAccess)
    }
    
    // Synthesize to spin.toml
    synthesizer := ftl.NewSynthesizer()
    manifest, err := synthesizer.SynthesizeApp(app)
    if err != nil {
        fmt.Fprintf(os.Stderr, "Error: %v\n", err)
        os.Exit(1)
    }
    
    // Output the manifest
    fmt.Println(manifest)
}