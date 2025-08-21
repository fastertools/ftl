// Package validation provides CUE-based configuration validation for FTL
package validation

import (
	"bytes"
	"encoding/json"
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
		if jwksURI, err := authValue.LookupPath(cue.ParsePath("jwt_jwks_uri")).String(); err == nil {
			auth.JWTJwksURI = jwksURI
		}
		if policy, err := authValue.LookupPath(cue.ParsePath("policy")).String(); err == nil {
			auth.Policy = policy
		}
		// Policy data can be string or object
		policyDataValue := authValue.LookupPath(cue.ParsePath("policy_data"))
		if policyDataValue.Exists() {
			// Try as string first
			if str, err := policyDataValue.String(); err == nil {
				auth.PolicyData = str
			} else {
				// Try to decode as interface{}
				var data interface{}
				if err := policyDataValue.Decode(&data); err == nil {
					auth.PolicyData = data
				}
			}
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
	Name            string            `json:"name,omitempty"`
	Version         string            `json:"version,omitempty"`
	Description     string            `json:"description,omitempty"`
	Access          string            `json:"access,omitempty"`
	Auth            *AuthConfig       `json:"auth,omitempty"`
	Components      []*Component      `json:"components,omitempty"`
	Variables  map[string]string `json:"variables,omitempty"`
}

// Component represents a validated component
type Component struct {
	ID        string            `json:"id"`
	Source    ComponentSource   `json:"-"` // Exclude from automatic JSON marshaling
	Build     *BuildConfig      `json:"build,omitempty"`
	Variables map[string]string `json:"variables,omitempty"`
}

// MarshalJSON implements custom JSON marshaling for Component to handle the Source interface
func (c Component) MarshalJSON() ([]byte, error) {
	type Alias Component // prevent recursion
	aux := struct {
		Source interface{} `json:"source"`
		Alias
	}{
		Alias: Alias(c),
	}

	// Handle the source field based on its concrete type
	switch src := c.Source.(type) {
	case *LocalSource:
		aux.Source = src.Path // Local source is just a string path
	case *RegistrySource:
		aux.Source = src // Registry source is a struct
	}

	return json.Marshal(aux)
}

// ComponentSource is either local or registry
type ComponentSource interface {
	isComponentSource()
}

// LocalSource represents a local component source
type LocalSource struct {
	Path string `json:"path"`
}

func (LocalSource) isComponentSource() {}

// RegistrySource represents a registry component source
type RegistrySource struct {
	Registry string `json:"registry"`
	Package  string `json:"package"`
	Version  string `json:"version"`
}

func (RegistrySource) isComponentSource() {}

// BuildConfig represents build configuration
type BuildConfig struct {
	Command string   `json:"command"`
	Workdir string   `json:"workdir,omitempty"`
	Watch   []string `json:"watch,omitempty"`
}

// AuthConfig represents authentication configuration
type AuthConfig struct {
	JWTIssuer   string      `json:"jwt_issuer,omitempty"`
	JWTAudience string      `json:"jwt_audience,omitempty"`
	JWTJwksURI  string      `json:"jwt_jwks_uri,omitempty"`
	Policy      string      `json:"policy,omitempty"`
	PolicyData  interface{} `json:"policy_data,omitempty"` // Can be string or map
}
