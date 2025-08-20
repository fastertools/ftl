package deploy

import (
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"time"

	v4 "github.com/aws/aws-sdk-go-v2/aws/signer/v4"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/credentials"
	"github.com/fastertools/ftl-cli/internal/api"
)

// StreamEvent represents a deployment progress event from the streaming response
type StreamEvent struct {
	Type         string            `json:"type"`      // "progress", "complete", "error"
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

// Deploy performs a deployment using the streaming Lambda Function URL
func (d *StreamingDeployer) Deploy(
	ctx context.Context,
	ftlConfig []byte,
	creds *api.CreateDeployCredentialsResponse,
	environment string,
	progressCallback func(event StreamEvent),
) error {
	// Create AWS config with temporary credentials
	cfg, err := config.LoadDefaultConfig(ctx,
		config.WithRegion(creds.Registry.Region),
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

	// Sign the request with AWS SigV4
	signer := v4.NewSigner()

	credsProvider := cfg.Credentials
	awsCreds, err := credsProvider.Retrieve(ctx)
	if err != nil {
		return fmt.Errorf("retrieve AWS credentials: %w", err)
	}

	// Sign with unsigned payload (streaming)
	err = signer.SignHTTP(ctx, awsCreds, req, "UNSIGNED-PAYLOAD", "lambda", creds.Registry.Region, time.Now())
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