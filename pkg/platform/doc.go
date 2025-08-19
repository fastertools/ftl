// Package platform provides the official API for FTL platform integrations.
//
// This package is designed for cloud platforms that deploy FTL applications
// to WebAssembly runtimes like Fermyon Cloud. It provides a clean, explicit
// API for processing deployments with proper security controls.
//
// # Basic Usage
//
// Create a client with your platform configuration:
//
//	config := platform.DefaultConfig()
//	config.RequireRegistryComponents = true
//	config.AllowedRegistries = []string{"ghcr.io", "your-ecr.amazonaws.com"}
//
//	client := platform.NewClient(config)
//
// Process deployment requests:
//
//	result, err := client.ProcessDeployment(request)
//	if err != nil {
//	    return handleError(err)
//	}
//
//	deployToFermyon(result.SpinTOML)
//
// # Platform Components
//
// The platform automatically injects security components:
//
//   - mcp-gateway: Always injected for routing and request handling
//   - mcp-authorizer: Injected for non-public applications
//
// These components are configurable through the Config struct.
//
// # Security
//
// The platform enforces several security policies:
//
//   - Component source validation (local vs registry)
//   - Registry whitelist enforcement
//   - Component count limits
//   - Automatic auth component injection
package platform
