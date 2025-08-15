// +build example

package ftl

// This file demonstrates how the backend Lambda functions would use the shared FTL package
// to process deployment requests and maintain consistency with the CLI

import (
	"context"
	"encoding/json"
	"fmt"
	"log"

	"github.com/aws/aws-lambda-go/events"
)

// ExampleDeploymentHandler shows how a Lambda function would handle deployment requests
func ExampleDeploymentHandler(ctx context.Context, request events.APIGatewayProxyRequest) (events.APIGatewayProxyResponse, error) {
	// Parse the deployment request
	var deployReq DeploymentRequest
	if err := json.Unmarshal([]byte(request.Body), &deployReq); err != nil {
		return errorResponse(400, "Invalid deployment request"), nil
	}
	
	// Process the deployment request to get the Spin manifest
	manifest, err := ProcessDeploymentRequest(&deployReq)
	if err != nil {
		log.Printf("Failed to process deployment: %v", err)
		return errorResponse(400, fmt.Sprintf("Failed to process deployment: %v", err)), nil
	}
	
	// Convert manifest to TOML for Spin deployment
	synth := NewSynthesizer()
	tomlManifest, err := synth.SynthesizeToTOML(deployReq.Application)
	if err != nil {
		log.Printf("Failed to synthesize TOML: %v", err)
		return errorResponse(500, "Failed to generate manifest"), nil
	}
	
	// At this point, the backend would:
	// 1. Store the manifest in S3 or similar
	// 2. Trigger the actual Spin deployment
	// 3. Update deployment status in the database
	
	// Example deployment ID generation
	deploymentID := generateDeploymentID()
	
	// Store deployment metadata
	if err := storeDeployment(deploymentID, deployReq, manifest); err != nil {
		return errorResponse(500, "Failed to store deployment"), nil
	}
	
	// Trigger async deployment (e.g., via SQS/Step Functions)
	if err := triggerSpinDeployment(deploymentID, tomlManifest); err != nil {
		return errorResponse(500, "Failed to trigger deployment"), nil
	}
	
	// Return deployment response
	response := DeploymentResponse{
		DeploymentID: deploymentID,
		AppID:        generateAppID(deployReq.Application.Name),
		AppName:      deployReq.Application.Name,
		Status:       "pending",
		Message:      "Deployment initiated",
	}
	
	respBody, _ := json.Marshal(response)
	return events.APIGatewayProxyResponse{
		StatusCode: 202,
		Headers: map[string]string{
			"Content-Type": "application/json",
		},
		Body: string(respBody),
	}, nil
}

// ExampleSynthesisHandler shows how to provide a synthesis endpoint for testing
func ExampleSynthesisHandler(ctx context.Context, request events.APIGatewayProxyRequest) (events.APIGatewayProxyResponse, error) {
	// Parse the FTL application
	var app Application
	if err := json.Unmarshal([]byte(request.Body), &app); err != nil {
		return errorResponse(400, "Invalid application configuration"), nil
	}
	
	// Synthesize to Spin manifest
	synth := NewSynthesizer()
	manifest, err := synth.SynthesizeToSpin(&app)
	if err != nil {
		return errorResponse(400, fmt.Sprintf("Failed to synthesize: %v", err)), nil
	}
	
	// Return the manifest as JSON for inspection
	respBody, _ := json.Marshal(manifest)
	return events.APIGatewayProxyResponse{
		StatusCode: 200,
		Headers: map[string]string{
			"Content-Type": "application/json",
		},
		Body: string(respBody),
	}, nil
}

// Helper functions (these would be implemented in the actual backend)

func errorResponse(code int, message string) events.APIGatewayProxyResponse {
	return events.APIGatewayProxyResponse{
		StatusCode: code,
		Headers: map[string]string{
			"Content-Type": "application/json",
		},
		Body: fmt.Sprintf(`{"error": "%s"}`, message),
	}
}

func generateDeploymentID() string {
	// In reality, use UUID or similar
	return "dep_" + generateRandomString(16)
}

func generateAppID(appName string) string {
	// In reality, this would look up or create the app ID
	return "app_" + generateRandomString(16)
}

func generateRandomString(length int) string {
	// Placeholder for actual implementation
	return "random123456789"
}

func storeDeployment(deploymentID string, req DeploymentRequest, manifest *SpinManifest) error {
	// Store in DynamoDB or similar
	log.Printf("Storing deployment %s", deploymentID)
	return nil
}

func triggerSpinDeployment(deploymentID string, tomlManifest string) error {
	// Trigger via SQS, Step Functions, or direct API call
	log.Printf("Triggering deployment %s", deploymentID)
	return nil
}