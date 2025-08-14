package construct

// Construct represents a high-level application pattern
type Construct struct {
	Name        string   `json:"name"`
	Description string   `json:"description"`
	Status      string   `json:"status"` // stable, preview, planned
	Features    []string `json:"features"`
	Examples    []string `json:"examples"`
}

// GetAvailableConstructs returns all available constructs
func GetAvailableConstructs() []Construct {
	return []Construct{
		{
			Name:        "mcp",
			Description: "Model Context Protocol application with authentication and tool gateway",
			Status:      "stable",
			Features: []string{
				"JWT-based authentication with configurable providers",
				"MCP gateway for tool orchestration",
				"Automatic component discovery and routing",
				"Built-in security and validation",
				"Scalable multi-tool architecture",
			},
			Examples: []string{
				"spindl init ai-assistant --template mcp",
				"spindl init tool-platform --template mcp",
			},
		},
		{
			Name:        "wordpress",
			Description: "WordPress site with database and caching",
			Status:      "planned",
			Features: []string{
				"WordPress core with PHP runtime",
				"MySQL database integration",
				"Redis caching layer",
				"File storage and media handling",
				"SSL/TLS termination",
			},
			Examples: []string{
				"spindl init my-blog --template wordpress",
			},
		},
		{
			Name:        "microservices",
			Description: "Microservices mesh with service discovery",
			Status:      "planned",
			Features: []string{
				"Service discovery and registration",
				"Load balancing and circuit breaking",
				"Distributed tracing and metrics",
				"API gateway with rate limiting",
				"Inter-service authentication",
			},
			Examples: []string{
				"spindl init api-platform --template microservices",
			},
		},
		{
			Name:        "ai-pipeline",
			Description: "AI/ML pipeline with model serving and data processing",
			Status:      "planned",
			Features: []string{
				"Model serving with versioning",
				"Data preprocessing pipelines",
				"Batch and streaming inference",
				"Model monitoring and drift detection",
				"A/B testing for model deployment",
			},
			Examples: []string{
				"spindl init ml-platform --template ai-pipeline",
			},
		},
	}
}

// GetConstruct returns a specific construct by name
func GetConstruct(name string) *Construct {
	constructs := GetAvailableConstructs()
	for _, c := range constructs {
		if c.Name == name {
			return &c
		}
	}
	return nil
}