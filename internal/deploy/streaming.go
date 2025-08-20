package deploy

import (
	"bufio"
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"

	v4 "github.com/aws/aws-sdk-go-v2/aws/signer/v4"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/credentials"
	"github.com/fastertools/ftl-cli/internal/api"
)

// StreamEvent represents a deployment progress event from the streaming response
type StreamEvent struct {
	Type         string            `json:"type"` // "progress", "complete", "error"
	Message      string            `json:"message"`
	DeploymentID string            `json:"deploymentId,omitempty"`
	URL          string            `json:"url,omitempty"`
	Data         map[string]string `json:"data,omitempty"`
	Timestamp    int64             `json:"timestamp"`
}

// StreamingDeployer handles deployments via Lambda Function URLs with streaming responses
type StreamingDeployer struct {
	httpClient *http.Client
}

// NewStreamingDeployer creates a new streaming deployer
func NewStreamingDeployer() *StreamingDeployer {
	return &StreamingDeployer{
		httpClient: &http.Client{
			Timeout: 5 * time.Minute, // Lambda can run up to 5 minutes
		},
	}
}

// extractRegion extracts the AWS region from various AWS URLs
func extractRegion(functionURL, registryURI string) string {
	// Try Lambda function URL first (format: https://xxx.lambda-url.REGION.on.aws/)
	if u, err := url.Parse(functionURL); err == nil {
		parts := strings.Split(u.Host, ".")
		for i, part := range parts {
			if part == "lambda-url" && i+1 < len(parts) {
				return parts[i+1]
			}
		}
	}

	// Try ECR registry URI (format: 123456789.dkr.ecr.REGION.amazonaws.com)
	if registryURI != "" {
		parts := strings.Split(registryURI, ".")
		for i, part := range parts {
			if part == "ecr" && i+1 < len(parts) {
				return parts[i+1]
			}
		}
	}

	// Default to us-west-2 if we can't extract
	return "us-west-2"
}

// Deploy performs a deployment using the streaming Lambda Function URL
func (d *StreamingDeployer) Deploy(
	ctx context.Context,
	ftlConfig []byte,
	creds *api.CreateDeployCredentialsResponse,
	environment string,
	progressCallback func(event StreamEvent),
) error {
	// Extract region from the URLs we have
	region := extractRegion(creds.Deployment.FunctionUrl, creds.Registry.RegistryUri)

	// Create AWS config with temporary credentials
	cfg, err := config.LoadDefaultConfig(ctx,
		config.WithRegion(region),
		config.WithCredentialsProvider(
			credentials.NewStaticCredentialsProvider(
				creds.Deployment.Credentials.AccessKeyId,
				creds.Deployment.Credentials.SecretAccessKey,
				creds.Deployment.Credentials.SessionToken,
			),
		),
	)
	if err != nil {
		return fmt.Errorf("create AWS config: %w", err)
	}

	// Build the request URL
	reqURL, err := url.Parse(creds.Deployment.FunctionUrl)
	if err != nil {
		return fmt.Errorf("parse function URL: %w", err)
	}

	// Add environment parameter if not production
	if environment != "" && environment != "production" {
		q := reqURL.Query()
		q.Set("environment", environment)
		reqURL.RawQuery = q.Encode()
	}

	// Create the request
	req, err := http.NewRequestWithContext(ctx, "POST", reqURL.String(), bytes.NewReader(ftlConfig))
	if err != nil {
		return fmt.Errorf("create request: %w", err)
	}

	// Set headers
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Accept", "application/x-ndjson")

	// Calculate payload hash for signing
	hash := sha256.Sum256(ftlConfig)
	payloadHash := hex.EncodeToString(hash[:])

	// Sign the request with AWS SigV4
	signer := v4.NewSigner()

	// Get credentials from the config
	awsCreds, err := cfg.Credentials.Retrieve(ctx)
	if err != nil {
		return fmt.Errorf("retrieve AWS credentials: %w", err)
	}

	// Sign the request
	err = signer.SignHTTP(ctx, awsCreds, req, payloadHash, "lambda", region, time.Now())
	if err != nil {
		return fmt.Errorf("sign request: %w", err)
	}

	// Make the request
	resp, err := d.httpClient.Do(req)
	if err != nil {
		return fmt.Errorf("send deployment request: %w", err)
	}
	defer resp.Body.Close()

	// Check for non-200 status
	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return fmt.Errorf("deployment failed with status %d: %s", resp.StatusCode, string(body))
	}

	// Process the NDJSON stream
	scanner := bufio.NewScanner(resp.Body)
	for scanner.Scan() {
		line := scanner.Bytes()
		if len(line) == 0 {
			continue // Skip empty lines
		}

		var event StreamEvent
		if err := json.Unmarshal(line, &event); err != nil {
			// Log the error but continue processing
			fmt.Printf("Warning: failed to parse stream event: %v\n", err)
			continue
		}

		// Call the progress callback if provided
		if progressCallback != nil {
			progressCallback(event)
		}

		// Handle event types
		switch event.Type {
		case "complete":
			return nil // Deployment successful
		case "error":
			return fmt.Errorf("deployment failed: %s", event.Message)
		case "progress":
			// Continue processing
		default:
			// Unknown event type, log and continue
			fmt.Printf("Unknown event type: %s\n", event.Type)
		}
	}

	if err := scanner.Err(); err != nil {
		return fmt.Errorf("error reading stream: %w", err)
	}

	// If we got here without a complete event, something went wrong
	return fmt.Errorf("deployment stream ended without completion")
}
