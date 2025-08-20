package validation

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestValidation_RequiredClaims(t *testing.T) {
	tests := []struct {
		name    string
		yaml    string
		wantErr bool
		check   func(t *testing.T, app *Application)
	}{
		{
			name: "org mode with required claims",
			yaml: `
name: test-app
access: org
required_claims:
  role: admin
  department: engineering
components: []
`,
			wantErr: false,
			check: func(t *testing.T, app *Application) {
				require.NotNil(t, app.RequiredClaims)
				assert.Equal(t, "admin", app.RequiredClaims["role"])
				assert.Equal(t, "engineering", app.RequiredClaims["department"])
			},
		},
		{
			name: "private mode with required claims",
			yaml: `
name: test-app
access: private
required_claims:
  clearance_level: secret
  team: platform
components: []
`,
			wantErr: false,
			check: func(t *testing.T, app *Application) {
				require.NotNil(t, app.RequiredClaims)
				assert.Equal(t, "secret", app.RequiredClaims["clearance_level"])
				assert.Equal(t, "platform", app.RequiredClaims["team"])
			},
		},
		{
			name: "required claims with array values",
			yaml: `
name: test-app
access: org
required_claims:
  role: admin
  permissions:
    - read
    - write
    - delete
components: []
`,
			wantErr: false,
			check: func(t *testing.T, app *Application) {
				require.NotNil(t, app.RequiredClaims)
				assert.Equal(t, "admin", app.RequiredClaims["role"])
				
				perms, ok := app.RequiredClaims["permissions"].([]interface{})
				require.True(t, ok, "permissions should be an array")
				assert.Len(t, perms, 3)
				assert.Contains(t, perms, "read")
				assert.Contains(t, perms, "write")
				assert.Contains(t, perms, "delete")
			},
		},
		{
			name: "required claims with nested objects",
			yaml: `
name: test-app
access: org
required_claims:
  user_type: admin
  metadata:
    department: engineering
    location: sf
    level: 5
components: []
`,
			wantErr: false,
			check: func(t *testing.T, app *Application) {
				require.NotNil(t, app.RequiredClaims)
				assert.Equal(t, "admin", app.RequiredClaims["user_type"])
				
				metadata, ok := app.RequiredClaims["metadata"].(map[string]interface{})
				require.True(t, ok, "metadata should be a map")
				assert.Equal(t, "engineering", metadata["department"])
				assert.Equal(t, "sf", metadata["location"])
				assert.EqualValues(t, 5, metadata["level"])
			},
		},
		{
			name: "public mode ignores required claims",
			yaml: `
name: test-app
access: public
required_claims:
  role: admin
components: []
`,
			wantErr: false,
			check: func(t *testing.T, app *Application) {
				// Required claims should still be parsed even in public mode
				// The synthesis layer will ignore them
				require.NotNil(t, app.RequiredClaims)
				assert.Equal(t, "admin", app.RequiredClaims["role"])
			},
		},
		{
			name: "no required claims",
			yaml: `
name: test-app
access: org
components: []
`,
			wantErr: false,
			check: func(t *testing.T, app *Application) {
				assert.Nil(t, app.RequiredClaims)
			},
		},
		{
			name: "required claims with boolean values",
			yaml: `
name: test-app
access: org
required_claims:
  verified: true
  beta_access: false
  admin: true
components: []
`,
			wantErr: false,
			check: func(t *testing.T, app *Application) {
				require.NotNil(t, app.RequiredClaims)
				assert.Equal(t, true, app.RequiredClaims["verified"])
				assert.Equal(t, false, app.RequiredClaims["beta_access"])
				assert.Equal(t, true, app.RequiredClaims["admin"])
			},
		},
		{
			name: "required claims with numeric values",
			yaml: `
name: test-app
access: org
required_claims:
  clearance_level: 5
  team_size: 10
  budget: 50000.50
components: []
`,
			wantErr: false,
			check: func(t *testing.T, app *Application) {
				require.NotNil(t, app.RequiredClaims)
				assert.EqualValues(t, 5, app.RequiredClaims["clearance_level"])
				assert.EqualValues(t, 10, app.RequiredClaims["team_size"])
				assert.InDelta(t, 50000.50, app.RequiredClaims["budget"], 0.01)
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			v := New()
			validated, err := v.ValidateYAML([]byte(tt.yaml))
			
			if tt.wantErr {
				require.Error(t, err)
				return
			}
			
			require.NoError(t, err)
			app, err := ExtractApplication(validated)
			require.NoError(t, err)
			
			if tt.check != nil {
				tt.check(t, app)
			}
		})
	}
}

// TestValidation_RequiredClaimsJSON tests JSON format support
func TestValidation_RequiredClaimsJSON(t *testing.T) {
	jsonConfig := `{
		"name": "test-app",
		"access": "org",
		"required_claims": {
			"role": "admin",
			"permissions": ["read", "write"],
			"metadata": {
				"department": "engineering",
				"level": 3
			}
		},
		"components": []
	}`

	v := New()
	validated, err := v.ValidateJSON([]byte(jsonConfig))
	require.NoError(t, err)
	
	app, err := ExtractApplication(validated)
	require.NoError(t, err)
	
	require.NotNil(t, app.RequiredClaims)
	assert.Equal(t, "admin", app.RequiredClaims["role"])
	
	perms, ok := app.RequiredClaims["permissions"].([]interface{})
	require.True(t, ok)
	assert.Len(t, perms, 2)
	
	metadata, ok := app.RequiredClaims["metadata"].(map[string]interface{})
	require.True(t, ok)
	assert.Equal(t, "engineering", metadata["department"])
	assert.EqualValues(t, 3, metadata["level"])
}