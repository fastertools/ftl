// Package validation provides CUE-based configuration validation for FTL
package validation

import (
	"bytes"
	"fmt"

	"cuelang.org/go/cue"
	"cuelang.org/go/cue/cuecontext"
	cuejson "cuelang.org/go/encoding/json"
	"cuelang.org/go/encoding/yaml"

	"github.com/fastertools/ftl-cli/pkg/synthesis"
)

// Validator provides CUE-based validation for FTL configurations
type Validator struct {
	ctx *cue.Context
}

// New creates a new validator instance
func New() *Validator {
	return &Validator{
		ctx: cuecontext.New(),
	}
}

// ValidateYAML validates YAML configuration against FTL schema
func (v *Validator) ValidateYAML(data []byte) (cue.Value, error) {
	// Extract YAML directly into CUE
	file, err := yaml.Extract("config.yaml", data)
	if err != nil {
		return cue.Value{}, fmt.Errorf("invalid YAML: %w", err)
	}

	value := v.ctx.BuildFile(file)
	if value.Err() != nil {
		return cue.Value{}, fmt.Errorf("failed to parse YAML: %w", value.Err())
	}

	return v.validate(value)
}

// ValidateJSON validates JSON configuration against FTL schema
func (v *Validator) ValidateJSON(data []byte) (cue.Value, error) {
	// Extract JSON directly into CUE
	decoder := cuejson.NewDecoder(nil, "config.json", bytes.NewReader(data))
	expr, err := decoder.Extract()
	if err != nil {
		return cue.Value{}, fmt.Errorf("invalid JSON: %w", err)
	}

	value := v.ctx.BuildExpr(expr)
	if value.Err() != nil {
		return cue.Value{}, fmt.Errorf("failed to parse JSON: %w", value.Err())
	}

	return v.validate(value)
}

// ValidateCUE validates CUE configuration against FTL schema
func (v *Validator) ValidateCUE(data []byte) (cue.Value, error) {
	value := v.ctx.CompileBytes(data, cue.Filename("config.cue"))
	if value.Err() != nil {
		return cue.Value{}, fmt.Errorf("invalid CUE: %w", value.Err())
	}

	return v.validate(value)
}

// validate applies FTL schema validation to a CUE value
func (v *Validator) validate(value cue.Value) (cue.Value, error) {
	// Compile the FTL patterns inline to avoid circular dependency
	// In production, this would be shared or embedded
	patterns := v.ctx.CompileString(synthesis.GetPatterns(), cue.Filename("patterns.cue"))
	if patterns.Err() != nil {
		return cue.Value{}, fmt.Errorf("failed to compile patterns: %w", patterns.Err())
	}

	// Get the FTLApplication schema
	schema := patterns.LookupPath(cue.ParsePath("#FTLApplication"))
	if !schema.Exists() {
		return cue.Value{}, fmt.Errorf("FTLApplication schema not found")
	}

	// Unify with the schema to validate
	unified := schema.Unify(value)
	if err := unified.Validate(); err != nil {
		return cue.Value{}, fmt.Errorf("validation failed: %w", err)
	}

	return unified, nil
}

// ExtractApplication extracts validated application data from CUE value
func ExtractApplication(v cue.Value) (*Application, error) {
	app := &Application{}

	// Extract basic fields
	if name, err := v.LookupPath(cue.ParsePath("name")).String(); err == nil {
		app.Name = name
	} else {
		return nil, fmt.Errorf("missing required field 'name'")
	}

	if version, err := v.LookupPath(cue.ParsePath("version")).String(); err == nil {
		app.Version = version
	}

	if desc, err := v.LookupPath(cue.ParsePath("description")).String(); err == nil {
		app.Description = desc
	}

	if access, err := v.LookupPath(cue.ParsePath("access")).String(); err == nil {
		app.Access = access
	}

	// Extract components
	componentsIter, err := v.LookupPath(cue.ParsePath("components")).List()
	if err == nil {
		for componentsIter.Next() {
			comp, err := extractComponent(componentsIter.Value())
			if err != nil {
				return nil, fmt.Errorf("invalid component: %w", err)
			}
			app.Components = append(app.Components, comp)
		}
	}

	// Extract auth if present
	authValue := v.LookupPath(cue.ParsePath("auth"))
	if authValue.Exists() {
		auth := &AuthConfig{}
		if issuer, err := authValue.LookupPath(cue.ParsePath("jwt_issuer")).String(); err == nil {
			auth.JWTIssuer = issuer
		}
		if audience, err := authValue.LookupPath(cue.ParsePath("jwt_audience")).String(); err == nil {
			auth.JWTAudience = audience
		}
		app.Auth = auth
	}

	// Extract variables
	varsValue := v.LookupPath(cue.ParsePath("variables"))
	if varsValue.Exists() {
		app.Variables = make(map[string]string)
		iter, _ := varsValue.Fields()
		for iter.Next() {
			if val, err := iter.Value().String(); err == nil {
				app.Variables[iter.Selector().Unquoted()] = val
			}
		}
	}

	return app, nil
}

func extractComponent(v cue.Value) (*Component, error) {
	comp := &Component{}

	if id, err := v.LookupPath(cue.ParsePath("id")).String(); err == nil {
		comp.ID = id
	} else {
		return nil, fmt.Errorf("component missing required field 'id'")
	}

	// Extract source (can be string or struct)
	sourceValue := v.LookupPath(cue.ParsePath("source"))
	if sourceStr, err := sourceValue.String(); err == nil {
		comp.Source = &LocalSource{Path: sourceStr}
	} else {
		// Try as registry source
		reg := &RegistrySource{}
		if r, err := sourceValue.LookupPath(cue.ParsePath("registry")).String(); err == nil {
			reg.Registry = r
		}
		if p, err := sourceValue.LookupPath(cue.ParsePath("package")).String(); err == nil {
			reg.Package = p
		}
		if ver, err := sourceValue.LookupPath(cue.ParsePath("version")).String(); err == nil {
			reg.Version = ver
		}
		comp.Source = reg
	}

	// Extract build config
	buildValue := v.LookupPath(cue.ParsePath("build"))
	if buildValue.Exists() {
		build := &BuildConfig{}
		if cmd, err := buildValue.LookupPath(cue.ParsePath("command")).String(); err == nil {
			build.Command = cmd
		}
		if wd, err := buildValue.LookupPath(cue.ParsePath("workdir")).String(); err == nil {
			build.Workdir = wd
		}
		watchIter, _ := buildValue.LookupPath(cue.ParsePath("watch")).List()
		for watchIter.Next() {
			if pattern, err := watchIter.Value().String(); err == nil {
				build.Watch = append(build.Watch, pattern)
			}
		}
		comp.Build = build
	}

	// Extract variables
	varsValue := v.LookupPath(cue.ParsePath("variables"))
	if varsValue.Exists() {
		comp.Variables = make(map[string]string)
		iter, _ := varsValue.Fields()
		for iter.Next() {
			if val, err := iter.Value().String(); err == nil {
				comp.Variables[iter.Selector().Unquoted()] = val
			}
		}
	}

	return comp, nil
}

// Application represents a validated FTL application
// These are strongly-typed, validated structures derived from CUE
type Application struct {
	Name        string
	Version     string
	Description string
	Access      string
	Auth        *AuthConfig
	Components  []*Component
	Variables   map[string]string
}

// Component represents a validated component
type Component struct {
	ID        string
	Source    ComponentSource
	Build     *BuildConfig
	Variables map[string]string
}

// ComponentSource is either local or registry
type ComponentSource interface {
	isComponentSource()
}

// LocalSource represents a local component source
type LocalSource struct {
	Path string
}

func (LocalSource) isComponentSource() {}

// RegistrySource represents a registry component source
type RegistrySource struct {
	Registry string
	Package  string
	Version  string
}

func (RegistrySource) isComponentSource() {}

// BuildConfig represents build configuration
type BuildConfig struct {
	Command string
	Workdir string
	Watch   []string
}

// AuthConfig represents authentication configuration
type AuthConfig struct {
	JWTIssuer   string
	JWTAudience string
}
