package platform

// Example showing how to handle org access mode with allowed subjects.
// This is used by the platform team when they need to restrict access
// to specific users within an organization.
func ExampleClient_ProcessDeployment_orgAccess() {
	// Platform team computes allowed subjects from org membership
	// filtered by allowed roles if specified
	allowedSubjects := computeAllowedSubjects("org_123", []string{"admin", "developer"})
	
	// Create the deployment request with org access configuration
	request := &DeploymentRequest{
		Application: &Application{
			Name:    "internal-tool",
			Version: "2.0.0",
			Access:  "org",
			Auth: &Auth{
				JWTIssuer:   "https://api.workos.com",
				JWTAudience: "internal-tool",
			},
			Components: []Component{
				{
					ID: "api",
					Source: map[string]interface{}{
						"registry": "ghcr.io",
						"package":  "myorg/internal-api",
						"version":  "2.0.0",
					},
				},
			},
		},
		// Inject the computed list of allowed user IDs
		AllowedSubjects: allowedSubjects,
		AllowedRoles:    []string{"admin", "developer"},
		Environment:     "production",
	}
	
	// Process the deployment
	config := DefaultConfig()
	client := NewClient(config)
	result, _ := client.ProcessDeployment(request)
	
	// The mcp-authorizer component will be configured with:
	// MCP_AUTH_ALLOWED_SUBJECTS = "user_01234,user_56789,user_abcde"
	// This restricts access to only these specific users
	_ = result
}

// computeAllowedSubjects would be implemented by the platform team
// to query their user management system and return user IDs
func computeAllowedSubjects(orgID string, roles []string) []string {
	// Platform team implementation:
	// 1. Query org membership from WorkOS or their user database
	// 2. Filter by roles if specified
	// 3. Return list of user IDs (subjects from JWT sub claim)
	
	// Example return value:
	return []string{
		"user_01234", // Admin user
		"user_56789", // Developer user  
		"user_abcde", // Another developer
	}
}