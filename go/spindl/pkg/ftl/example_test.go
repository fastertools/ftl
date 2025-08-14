package ftl_test

import (
    "fmt"
    "github.com/fastertools/ftl-cli/go/spindl/pkg/ftl"
)

func Example_simpleApp() {
    // Create a new FTL application with fluent API
    app := ftl.NewApp("my-mcp-platform").
        SetDescription("An MCP tool orchestration platform").
        SetVersion("1.0.0")
    
    // Add a local tool with build config
    app.AddTool("calculator").
        FromLocal("./tools/calc.wasm").
        WithBuild("cargo build --release").
        WithWatch("src/**/*.rs", "Cargo.toml").
        Build()
    
    // Add a tool from registry
    app.AddTool("weather").
        FromRegistry("ghcr.io", "example/weather", "2.0.0").
        WithEnv("API_KEY", "secret-key").
        Build()
    
    // Add another tool
    app.AddTool("translator").
        FromLocal("./tools/translator.wasm").
        Build()
    
    // Enable authentication
    app.EnableWorkOSAuth("org_123456")
    
    // Generate the CUE representation
    cue, _ := app.ToCUE()
    fmt.Println("Generated CUE:")
    fmt.Println(cue)
    
    // Synthesize to final spin.toml
    manifest, _ := app.Synthesize()
    fmt.Println("\nFinal spin.toml:")
    fmt.Println(manifest)
}

func Example_publicApp() {
    // Simple public app with no auth
    app := ftl.NewApp("public-tools").
        SetAccess(ftl.PublicAccess)
    
    app.AddTool("echo").
        FromLocal("./echo.wasm")
    
    manifest, _ := app.Synthesize()
    fmt.Println(manifest)
}

func Example_customAuth() {
    app := ftl.NewApp("enterprise-mcp")
    
    // Add tools
    app.AddTool("database-admin").
        FromRegistry("internal.company.com", "tools/db-admin", "1.0.0")
    
    // Enable custom auth
    app.EnableCustomAuth(
        "https://auth.company.com",
        "mcp-platform",
    )
    
    manifest, _ := app.Synthesize()
    fmt.Println(manifest)
}