package oci

import (
	"encoding/base64"
	"fmt"
	"strings"
)

// ECRAuth holds parsed ECR authentication details
type ECRAuth struct {
	Registry string
	Username string
	Password string
}

// ParseECRToken parses an ECR authorization token from AWS
// The token is base64 encoded in the format "AWS:password"
func ParseECRToken(registryURI, authToken string) (*ECRAuth, error) {
	decoded, err := base64.StdEncoding.DecodeString(authToken)
	if err != nil {
		return nil, fmt.Errorf("failed to decode ECR token: %w", err)
	}

	parts := strings.SplitN(string(decoded), ":", 2)
	if len(parts) != 2 || parts[0] != "AWS" {
		return nil, fmt.Errorf("invalid ECR token format")
	}

	return &ECRAuth{
		Registry: registryURI,
		Username: parts[0],
		Password: parts[1],
	}, nil
}
