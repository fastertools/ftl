package synth

import (
	"bytes"
	"fmt"
	"strings"
	"text/template"

	"github.com/BurntSushi/toml"
	"github.com/fastertools/ftl-cli/go/shared/config"
	"gopkg.in/yaml.v3"
)

// MCP Component versions
const (
	MCPGatewayVersion    = "0.0.13-alpha.0"
	MCPAuthorizerVersion = "0.0.15-alpha.0"
	MCPRegistry          = "ghcr.io"
)

// SynthesizeFromConfig generates a spin.toml from an FTLConfig struct
func (e *Engine) SynthesizeFromConfig(cfg *config.FTLConfig) (string, error) {
	// Create the Spin manifest structure
	manifest := map[string]interface{}{
		"spin_manifest_version": 2,
		"application": map[string]interface{}{
			"name":    cfg.Application.Name,
			"version": cfg.Application.Version,
		},
	}

	// Add description and authors if present
	if cfg.Application.Description != "" {
		manifest["application"].(map[string]interface{})["description"] = cfg.Application.Description
	}
	if len(cfg.Application.Authors) > 0 {
		manifest["application"].(map[string]interface{})["authors"] = cfg.Application.Authors
	}

	// Create a components map to handle separately
	components := make(map[string]map[string]interface{})
	
	// Convert components (Spin v3 format uses [component.<id>] sections)
	for _, comp := range cfg.Components {
		spinComp := map[string]interface{}{}

		// Pass through the source directly - let Spin handle resolution
		// Spin supports:
		// - String paths for local files
		// - Registry references like "registry.io/namespace/package:version"
		// - Tables with { url = "...", digest = "..." }
		// - Tables with { registry = "...", package = "...", version = "..." }
		
		// Convert source from interface{} to proper type
		switch src := comp.Source.(type) {
		case string:
			spinComp["source"] = src
		case map[interface{}]interface{}:
			// Convert map[interface{}]interface{} to map[string]interface{}
			sourceMap := make(map[string]interface{})
			for k, v := range src {
				if key, ok := k.(string); ok {
					sourceMap[key] = v
				}
			}
			spinComp["source"] = sourceMap
		case map[string]interface{}:
			spinComp["source"] = src
		}
		
		if comp.Description != "" {
			spinComp["description"] = comp.Description
		}

		// Add build config if present
		if comp.Build != nil {
			build := map[string]interface{}{}
			if comp.Build.Command != "" {
				build["command"] = comp.Build.Command
			}
			if comp.Build.Workdir != "" {
				build["workdir"] = comp.Build.Workdir
			}
			if len(comp.Build.Watch) > 0 {
				build["watch"] = comp.Build.Watch
			}
			if len(build) > 0 {
				spinComp["build"] = build
			}
		}

		// Add environment variables
		if len(comp.Environment) > 0 {
			spinComp["environment"] = comp.Environment
		}

		// Add files
		if len(comp.Files) > 0 {
			var files []map[string]string
			for _, f := range comp.Files {
				files = append(files, map[string]string{
					"source":      f.Source,
					"destination": f.Destination,
				})
			}
			spinComp["files"] = files
		}

		// Add allowed hosts
		if len(comp.AllowedOutboundHosts) > 0 {
			spinComp["allowed_outbound_hosts"] = comp.AllowedOutboundHosts
		}
		if len(comp.AllowedHTTPHosts) > 0 {
			spinComp["allowed_http_hosts"] = comp.AllowedHTTPHosts
		}

		// Add key-value stores
		if len(comp.KeyValueStores) > 0 {
			spinComp["key_value_stores"] = comp.KeyValueStores
		}

		// Add SQLite databases
		if len(comp.SQLiteDatabases) > 0 {
			spinComp["sqlite_databases"] = comp.SQLiteDatabases
		}

		// Add AI models
		if len(comp.AIModels) > 0 {
			spinComp["ai_models"] = comp.AIModels
		}

		// Don't add triggers inline - they'll be added separately for Spin v3

		// Store component in components map
		components[comp.ID] = spinComp
	}

	// Add variables
	if len(cfg.Variables) > 0 {
		var variables []map[string]interface{}
		for k, v := range cfg.Variables {
			variables = append(variables, map[string]interface{}{
				"name":    k,
				"default": v,
			})
		}
		manifest["variables"] = variables
	}

	// Convert to TOML with special handling for components
	var buf bytes.Buffer
	
	// First encode the main manifest without components
	encoder := toml.NewEncoder(&buf)
	encoder.Indent = ""
	if err := encoder.Encode(manifest); err != nil {
		return "", fmt.Errorf("failed to encode to TOML: %w", err)
	}
	
	// Now manually add components with proper section headers
	for id, component := range components {
		buf.WriteString(fmt.Sprintf("\n[component.%s]\n", id))
		// Write each component property manually to avoid nested encoding issues
		// Handle different source types
		if source := component["source"]; source != nil {
			switch src := source.(type) {
			case string:
				// Local path - simple string
				buf.WriteString(fmt.Sprintf("source = %q\n", src))
			case map[string]interface{}:
				// Registry or URL source - inline table format
				if registry, ok := src["registry"]; ok {
					// Registry source: { registry = "...", package = "...", version = "..." }
					buf.WriteString("source = { ")
					buf.WriteString(fmt.Sprintf("registry = %q", registry))
					if pkg, ok := src["package"]; ok {
						buf.WriteString(fmt.Sprintf(", package = %q", pkg))
					}
					if version, ok := src["version"]; ok {
						buf.WriteString(fmt.Sprintf(", version = %q", version))
					}
					buf.WriteString(" }\n")
				} else if url, ok := src["url"]; ok {
					// URL source: { url = "...", digest = "..." }
					buf.WriteString("source = { ")
					buf.WriteString(fmt.Sprintf("url = %q", url))
					if digest, ok := src["digest"]; ok {
						buf.WriteString(fmt.Sprintf(", digest = %q", digest))
					}
					buf.WriteString(" }\n")
				}
			}
		}
		if desc, ok := component["description"].(string); ok {
			buf.WriteString(fmt.Sprintf("description = %q\n", desc))
		}
		// Handle other properties like environment, allowed_hosts, etc.
		if env, ok := component["environment"].(map[string]string); ok && len(env) > 0 {
			buf.WriteString(fmt.Sprintf("\n[component.%s.environment]\n", id))
			for k, v := range env {
				buf.WriteString(fmt.Sprintf("%s = %q\n", k, v))
			}
		}
		if hosts, ok := component["allowed_outbound_hosts"].([]string); ok && len(hosts) > 0 {
			buf.WriteString("allowed_outbound_hosts = [")
			for i, host := range hosts {
				if i > 0 {
					buf.WriteString(", ")
				}
				buf.WriteString(fmt.Sprintf("%q", host))
			}
			buf.WriteString("]\n")
		}
		// Add build command if present
		if build, ok := component["build"].(map[string]interface{}); ok {
			if cmd, ok := build["command"].(string); ok {
				buf.WriteString(fmt.Sprintf("\n[component.%s.build]\n", id))
				buf.WriteString(fmt.Sprintf("command = %q\n", cmd))
				if workdir, ok := build["workdir"].(string); ok {
					buf.WriteString(fmt.Sprintf("workdir = %q\n", workdir))
				}
			}
		}
	}
	
	// Add MCP components and their triggers based on access control
	accessControl := config.AccessControlPublic // default
	if cfg.MCP != nil && cfg.MCP.Authorizer != nil && cfg.MCP.Authorizer.AccessControl != "" {
		accessControl = cfg.MCP.Authorizer.AccessControl
	}
	
	// Collect component names for the gateway
	componentNames := []string{}
	for _, comp := range cfg.Components {
		componentNames = append(componentNames, comp.ID)
	}
	
	// Add MCP Gateway component (always needed)
	buf.WriteString("\n[component.ftl-mcp-gateway]\n")
	buf.WriteString(fmt.Sprintf("source = { registry = %q, package = \"fastertools:mcp-gateway\", version = %q }\n", MCPRegistry, MCPGatewayVersion))
	buf.WriteString("allowed_outbound_hosts = [\"http://*.spin.internal\"]\n")
	buf.WriteString("\n[component.ftl-mcp-gateway.variables]\n")
	// Write component_names as a comma-separated string (as expected by the MCP gateway)
	buf.WriteString(fmt.Sprintf("component_names = %q\n", strings.Join(componentNames, ",")))
	
	// Add MCP Authorizer component only for non-public access
	if accessControl != config.AccessControlPublic {
		buf.WriteString("\n[component.mcp-authorizer]\n")
		buf.WriteString(fmt.Sprintf("source = { registry = %q, package = \"fastertools:mcp-authorizer\", version = %q }\n", MCPRegistry, MCPAuthorizerVersion))
		buf.WriteString("allowed_outbound_hosts = [\"http://*.spin.internal\", \"https://*.authkit.app\", \"https://*.workos.com\"]\n")
		buf.WriteString("key_value_stores = [\"default\"]\n")
		
		// Add auth variables if configured
		if cfg.MCP != nil && cfg.MCP.Authorizer != nil {
			buf.WriteString("\n[component.mcp-authorizer.variables]\n")
			if cfg.MCP.Authorizer.JWTIssuer != "" {
				buf.WriteString(fmt.Sprintf("mcp_jwt_issuer = %q\n", cfg.MCP.Authorizer.JWTIssuer))
			}
			if cfg.MCP.Authorizer.JWTAudience != "" {
				buf.WriteString(fmt.Sprintf("mcp_jwt_audience = %q\n", cfg.MCP.Authorizer.JWTAudience))
			}
			buf.WriteString("mcp_gateway_url = \"http://ftl-mcp-gateway.spin.internal\"\n")
		}
	}
	
	// Add triggers in separate trigger sections for Spin v3
	// Group triggers by type
	httpTriggers := []config.TriggerConfig{}
	redisTriggers := []config.TriggerConfig{}
	
	for _, trig := range cfg.Triggers {
		switch trig.Type {
		case config.TriggerTypeHTTP:
			httpTriggers = append(httpTriggers, trig)
		case config.TriggerTypeRedis:
			redisTriggers = append(redisTriggers, trig)
		}
	}
	
	// Add MCP component triggers based on access control
	if accessControl == config.AccessControlPublic {
		// Public mode: gateway handles all public routes (catch-all)
		buf.WriteString("\n[[trigger.http]]\n")
		buf.WriteString("route = \"/...\"\n")
		buf.WriteString("component = \"ftl-mcp-gateway\"\n")
	} else {
		// Private/org/custom mode: authorizer handles all public routes (catch-all)
		buf.WriteString("\n[[trigger.http]]\n")
		buf.WriteString("route = \"/...\"\n")
		buf.WriteString("component = \"mcp-authorizer\"\n")
		
		// Gateway is private (only accessible internally)
		buf.WriteString("\n[[trigger.http]]\n")
		buf.WriteString("route = { private = true }\n")
		buf.WriteString("component = \"ftl-mcp-gateway\"\n")
	}
	
	// Add user component HTTP triggers (all private)
	for _, trig := range httpTriggers {
		buf.WriteString(fmt.Sprintf("\n[[trigger.http]]\n"))
		
		// Handle special route values for FTL
		if trig.Route == "private" {
			// Private routes in Spin are represented as { private = true }
			buf.WriteString("route = { private = true }\n")
		} else if trig.Route != "" {
			buf.WriteString(fmt.Sprintf("route = %q\n", trig.Route))
		}
		
		buf.WriteString(fmt.Sprintf("component = %q\n", trig.Component))
	}
	
	// Add Redis triggers
	if len(redisTriggers) > 0 {
		for i, trig := range redisTriggers {
			buf.WriteString(fmt.Sprintf("\n[[trigger.redis]]\n"))
			buf.WriteString(fmt.Sprintf("component = %q\n", trig.Component))
			if trig.Channel != "" {
				buf.WriteString(fmt.Sprintf("channel = %q\n", trig.Channel))
			}
			buf.WriteString(fmt.Sprintf("id = \"redis-trigger-%d\"\n", i))
		}
	}

	return buf.String(), nil
}

// substituteTemplates replaces template variables in the config
func substituteTemplates(value string, vars map[string]string) (string, error) {
	tmpl, err := template.New("config").Parse(value)
	if err != nil {
		return value, nil // If it's not a valid template, return as-is
	}

	var buf bytes.Buffer
	if err := tmpl.Execute(&buf, vars); err != nil {
		return value, nil // If template execution fails, return as-is
	}

	return buf.String(), nil
}

// SynthesizeFromYAML generates a spin.toml from YAML data
func SynthesizeFromYAML(yamlData []byte) ([]byte, error) {
	// Parse YAML into FTLConfig
	var cfg config.FTLConfig
	if err := yaml.Unmarshal(yamlData, &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse YAML: %w", err)
	}
	
	// Create a temporary engine
	engine := &Engine{}
	
	// Use the existing SynthesizeFromConfig method
	tomlStr, err := engine.SynthesizeFromConfig(&cfg)
	if err != nil {
		return nil, err
	}
	
	return []byte(tomlStr), nil
}