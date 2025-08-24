package config

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// ConfigFile represents a detected configuration file
type ConfigFile struct {
	Path   string
	Format string
}

// DetectionOptions controls config file detection behavior
type DetectionOptions struct {
	PreferredFormats []string // Override default priority
	IncludeGo        bool     // Include main.go/platform.go
	IncludeCUE       bool     // Include .cue files
}

// DefaultOptions returns the standard detection options used by most commands
func DefaultOptions() *DetectionOptions {
	return &DetectionOptions{
		PreferredFormats: []string{"ftl.yaml", "ftl.yml", "ftl.json", "app.cue"},
		IncludeGo:        true,
		IncludeCUE:       true,
	}
}

// BuildOptions returns options optimized for build/up commands
func BuildOptions() *DetectionOptions {
	return &DetectionOptions{
		PreferredFormats: []string{"ftl.yaml", "ftl.json", "app.cue", "main.go"},
		IncludeGo:        true,
		IncludeCUE:       true,
	}
}

// FindConfigFile detects configuration files with given options
func FindConfigFile(opts *DetectionOptions) (*ConfigFile, error) {
	if opts == nil {
		opts = DefaultOptions()
	}
	
	candidates := buildCandidateList(opts)
	
	for _, candidate := range candidates {
		if _, err := os.Stat(candidate.Path); err == nil {
			return candidate, nil
		}
	}
	
	return nil, fmt.Errorf("no FTL configuration file found. Looked for: %v", 
		extractPaths(candidates))
}

// AutoDetectConfigFile uses default detection options
func AutoDetectConfigFile() (*ConfigFile, error) {
	return FindConfigFile(DefaultOptions())
}

// AutoDetectForBuild uses build-optimized detection options
func AutoDetectForBuild() (*ConfigFile, error) {
	return FindConfigFile(BuildOptions())
}

func buildCandidateList(opts *DetectionOptions) []*ConfigFile {
	var candidates []*ConfigFile
	
	// Add preferred formats first
	for _, path := range opts.PreferredFormats {
		candidates = append(candidates, &ConfigFile{
			Path:   path,
			Format: detectFormat(path),
		})
	}
	
	// Add Go files if enabled and not already included
	if opts.IncludeGo {
		goFiles := []string{"main.go", "platform.go"}
		for _, goFile := range goFiles {
			if !containsPath(candidates, goFile) {
				candidates = append(candidates, &ConfigFile{
					Path:   goFile,
					Format: "go",
				})
			}
		}
	}
	
	// Add CUE files if enabled and not already included
	if opts.IncludeCUE {
		cueFiles := []string{"ftl.cue", "app.cue"}
		for _, cueFile := range cueFiles {
			if !containsPath(candidates, cueFile) {
				candidates = append(candidates, &ConfigFile{
					Path:   cueFile,
					Format: "cue",
				})
			}
		}
	}
	
	return candidates
}

func detectFormat(path string) string {
	ext := filepath.Ext(path)
	switch ext {
	case ".yaml", ".yml":
		return "yaml"
	case ".json":
		return "json"
	case ".cue":
		return "cue"
	case ".go":
		return "go"
	default:
		// Handle files without extensions or special cases
		if strings.Contains(path, "yaml") {
			return "yaml"
		}
		if strings.Contains(path, "json") {
			return "json"
		}
		return "unknown"
	}
}

func containsPath(candidates []*ConfigFile, path string) bool {
	for _, candidate := range candidates {
		if candidate.Path == path {
			return true
		}
	}
	return false
}

func extractPaths(candidates []*ConfigFile) []string {
	var paths []string
	for _, candidate := range candidates {
		paths = append(paths, candidate.Path)
	}
	return paths
}