package synthesis

import (
	"bytes"
	_ "embed"
	"fmt"
	"sort"
	"strings"

	"cuelang.org/go/cue"
	"cuelang.org/go/cue/cuecontext"
)

//go:embed patterns.cue
var ftlPatterns string

// Synthesizer handles the multi-stage synthesis pipeline
type Synthesizer struct {
	ctx *cue.Context
}

// NewSynthesizer creates a new synthesizer
func NewSynthesizer() *Synthesizer {
	return &Synthesizer{
		ctx: cuecontext.New(),
	}
}

// SynthesizeApp takes an App and produces spin.toml
func (s *Synthesizer) SynthesizeApp(app *App) (string, error) {
	// Stage 1: Generate CUE from Go SDK
	cueStr, err := app.ToCUE()
	if err != nil {
		return "", fmt.Errorf("failed to generate CUE: %w", err)
	}

	// Stage 2: Transform through CUE stages (FTL → SpinDL → Manifest)
	manifest, err := s.transformThroughCUE(cueStr)
	if err != nil {
		return "", fmt.Errorf("failed to transform through CUE: %w", err)
	}

	// Stage 3: Convert to TOML
	return s.toTOML(manifest)
}

// SynthesizeCUE takes raw CUE and produces spin.toml
func (s *Synthesizer) SynthesizeCUE(cueStr string) (string, error) {
	// Transform through CUE stages (FTL → SpinDL → Manifest)
	manifest, err := s.transformThroughCUE(cueStr)
	if err != nil {
		return "", fmt.Errorf("failed to transform through CUE: %w", err)
	}

	// Convert to TOML
	return s.toTOML(manifest)
}

// transformThroughCUE performs the complete transformation pipeline in CUE
func (s *Synthesizer) transformThroughCUE(cueStr string) (map[string]interface{}, error) {
	// Load the FTL patterns
	patternsContent := s.getFTLPatterns()

	// Build the complete CUE document with both transformations
	fullCUE := patternsContent + "\n\n" + cueStr + "\n\n" + `
// Stage 1: Transform FTL to SpinDL intermediate model
spindl: (#FTLToSpinDL & {input: app}).output

// Stage 2: Transform SpinDL to final manifest
manifest: (#SpinDLToManifest & {input: spindl}).output
`

	// Compile and evaluate the entire pipeline
	v := s.ctx.CompileString(fullCUE)
	if v.Err() != nil {
		return nil, fmt.Errorf("CUE compilation failed: %w", v.Err())
	}

	// Extract the final manifest
	manifest := v.LookupPath(cue.ParsePath("manifest"))
	if manifest.Err() != nil {
		return nil, fmt.Errorf("failed to extract manifest: %w", manifest.Err())
	}

	// Convert to Go map
	var manifestMap map[string]interface{}
	if err := manifest.Decode(&manifestMap); err != nil {
		return nil, fmt.Errorf("failed to decode manifest: %w", err)
	}

	// Post-process: Add component_names to gateway
	s.addComponentNames(manifestMap)

	return manifestMap, nil
}

// addComponentNames adds the component_names variable to the MCP gateway
func (s *Synthesizer) addComponentNames(manifest map[string]interface{}) {
	components, ok := manifest["component"].(map[string]interface{})
	if !ok {
		return
	}

	// Collect user component names (exclude MCP components)
	var userComponents []string
	for id := range components {
		if id != "ftl-mcp-gateway" && id != "mcp-authorizer" {
			userComponents = append(userComponents, id)
		}
	}
	sort.Strings(userComponents)

	// Add to gateway variables
	if gateway, ok := components["ftl-mcp-gateway"].(map[string]interface{}); ok {
		if len(userComponents) > 0 {
			if variables, ok := gateway["variables"].(map[string]interface{}); ok {
				variables["component_names"] = strings.Join(userComponents, ",")
			} else {
				gateway["variables"] = map[string]interface{}{
					"component_names": strings.Join(userComponents, ","),
				}
			}
		}
	}
}

// toTOML converts the manifest map to TOML format
func (s *Synthesizer) toTOML(manifest map[string]interface{}) (string, error) {
	var buf bytes.Buffer

	// Write manifest version and application
	buf.WriteString("spin_manifest_version = 2\n\n")

	if app, ok := manifest["application"].(map[string]interface{}); ok {
		buf.WriteString("[application]\n")
		if name, ok := app["name"].(string); ok {
			buf.WriteString(fmt.Sprintf("name = %q\n", name))
		}
		if version, ok := app["version"].(string); ok {
			buf.WriteString(fmt.Sprintf("version = %q\n", version))
		}
		if desc, ok := app["description"].(string); ok && desc != "" {
			buf.WriteString(fmt.Sprintf("description = %q\n", desc))
		}
		buf.WriteString("\n")
	}

	// Write components
	if components, ok := manifest["component"].(map[string]interface{}); ok {
		// Sort component keys for consistent output
		var componentKeys []string
		for k := range components {
			componentKeys = append(componentKeys, k)
		}
		sort.Strings(componentKeys)

		for _, id := range componentKeys {
			comp := components[id].(map[string]interface{})
			buf.WriteString(fmt.Sprintf("[component.%s]\n", id))

			// Write source
			if source := comp["source"]; source != nil {
				s.writeSource(&buf, source)
			}

			// Write other component properties
			if hosts, ok := comp["allowed_outbound_hosts"].([]interface{}); ok {
				buf.WriteString("allowed_outbound_hosts = [")
				for i, h := range hosts {
					if i > 0 {
						buf.WriteString(", ")
					}
					buf.WriteString(fmt.Sprintf("%q", h))
				}
				buf.WriteString("]\n")
			}

			// Write build config
			if build, ok := comp["build"].(map[string]interface{}); ok {
				buf.WriteString(fmt.Sprintf("\n[component.%s.build]\n", id))
				if cmd, ok := build["command"].(string); ok {
					buf.WriteString(fmt.Sprintf("command = %q\n", cmd))
				}
				if workdir, ok := build["workdir"].(string); ok {
					buf.WriteString(fmt.Sprintf("workdir = %q\n", workdir))
				}
				if watch, ok := build["watch"].([]interface{}); ok {
					buf.WriteString("watch = [")
					for i, w := range watch {
						if i > 0 {
							buf.WriteString(", ")
						}
						buf.WriteString(fmt.Sprintf("%q", w))
					}
					buf.WriteString("]\n")
				}
			}

			// Write variables
			if vars, ok := comp["variables"].(map[string]interface{}); ok {
				buf.WriteString(fmt.Sprintf("\n[component.%s.variables]\n", id))
				for k, v := range vars {
					buf.WriteString(fmt.Sprintf("%s = %q\n", k, v))
				}
			}

			buf.WriteString("\n")
		}
	}

	// Write triggers
	if triggers, ok := manifest["trigger"].(map[string]interface{}); ok {
		if httpTriggers, ok := triggers["http"].([]interface{}); ok {
			for _, t := range httpTriggers {
				trig := t.(map[string]interface{})
				buf.WriteString("[[trigger.http]]\n")

				// Write route
				if route := trig["route"]; route != nil {
					switch r := route.(type) {
					case string:
						buf.WriteString(fmt.Sprintf("route = %q\n", r))
					case map[string]interface{}:
						if private, ok := r["private"].(bool); ok && private {
							buf.WriteString("route = { private = true }\n")
						}
					}
				}

				// Write component
				if comp, ok := trig["component"].(string); ok {
					buf.WriteString(fmt.Sprintf("component = %q\n", comp))
				}

				buf.WriteString("\n")
			}
		}
	}

	return buf.String(), nil
}

// writeSource writes the source field in the appropriate format
func (s *Synthesizer) writeSource(buf *bytes.Buffer, source interface{}) {
	switch src := source.(type) {
	case string:
		buf.WriteString(fmt.Sprintf("source = %q\n", src))
	case map[string]interface{}:
		if registry, ok := src["registry"].(string); ok {
			// Registry source
			buf.WriteString("source = { ")
			buf.WriteString(fmt.Sprintf("registry = %q", registry))
			if pkg, ok := src["package"].(string); ok {
				buf.WriteString(fmt.Sprintf(", package = %q", pkg))
			}
			if version, ok := src["version"].(string); ok {
				buf.WriteString(fmt.Sprintf(", version = %q", version))
			}
			buf.WriteString(" }\n")
		} else if url, ok := src["url"].(string); ok {
			// URL source
			buf.WriteString("source = { ")
			buf.WriteString(fmt.Sprintf("url = %q", url))
			if digest, ok := src["digest"].(string); ok {
				buf.WriteString(fmt.Sprintf(", digest = %q", digest))
			}
			buf.WriteString(" }\n")
		}
	}
}

// getFTLPatterns returns the FTL CUE patterns
func (s *Synthesizer) getFTLPatterns() string {
	return ftlPatterns
}
