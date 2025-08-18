package ftl

import (
	"encoding/base64"
	"fmt"
	"strings"
)

// ECRAuth represents parsed ECR authentication credentials
type ECRAuth struct {
	Registry string
	Username string
	Password string
}

// ParseECRToken decodes an ECR authorization token into usable credentials
// This is useful for Docker login and other ECR authentication needs
func ParseECRToken(registryURI string, authToken string) (*ECRAuth, error) {
	// Decode the base64 authorization token
	decoded, err := base64.StdEncoding.DecodeString(authToken)
	if err != nil {
		return nil, fmt.Errorf("failed to decode ECR token: %w", err)
	}

	// Extract username and password (format is "AWS:password")
	parts := strings.SplitN(string(decoded), ":", 2)
	if len(parts) != 2 || parts[0] != "AWS" {
		return nil, fmt.Errorf("invalid ECR token format")
	}

	// Clean up registry URI (remove protocol if present)
	registry := strings.TrimPrefix(registryURI, "https://")
	registry = strings.TrimPrefix(registry, "http://")

	return &ECRAuth{
		Registry: registry,
		Username: parts[0],
		Password: parts[1],
	}, nil
}
