package cmd

import (
	"bytes"
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	"github.com/AlecAivazis/survey/v2"
	"github.com/briandowns/spinner"
	"github.com/fatih/color"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"

	"github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis"
	"github.com/fastertools/ftl-cli/go/shared/api"
	"github.com/fastertools/ftl-cli/go/shared/auth"
	"github.com/fastertools/ftl-cli/go/shared/config"
)

// DeployOptions holds options for the deploy command
type DeployOptions struct {
	Environment    string
	ConfigFile     string
	DryRun         bool
	Yes            bool
	AccessControl  string
	JWTIssuer      string
	JWTAudience    string
	AllowedRoles   []string
	Variables      map[string]string
}

func newDeployCmd() *cobra.Command {
	opts := &DeployOptions{
		Variables: make(map[string]string),
	}

	cmd := &cobra.Command{
		Use:   "deploy [flags]",
		Short: "Deploy the FTL application to the platform",
		Long: `Deploy the FTL application to the platform.

This command:
1. Reads your FTL configuration (ftl.yaml, ftl.json, or app.cue)
2. Builds local components
3. Creates/updates the app on FTL platform
4. Pushes built components to the registry
5. Sends the FTL config to the platform for deployment
6. Platform synthesizes Spin manifest and deploys

Example:
  ftl deploy
  ftl deploy --access-control private
  ftl deploy --jwt-issuer https://auth.example.com --jwt-audience api.example.com
  ftl deploy --dry-run`,
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := context.Background()
			return runDeploy(ctx, opts)
		},
	}

	cmd.Flags().StringVarP(&opts.Environment, "environment", "e", "production", "Deployment environment")
	cmd.Flags().StringVarP(&opts.ConfigFile, "file", "f", "", "FTL configuration file (auto-detects if not specified)")
	cmd.Flags().BoolVar(&opts.DryRun, "dry-run", false, "Validate configuration without deploying")
	cmd.Flags().BoolVarP(&opts.Yes, "yes", "y", false, "Skip confirmation prompt")
	cmd.Flags().StringVar(&opts.AccessControl, "access-control", "", "Access control mode (public, private, org, custom)")
	cmd.Flags().StringVar(&opts.JWTIssuer, "jwt-issuer", "", "JWT issuer URL for authentication")
	cmd.Flags().StringVar(&opts.JWTAudience, "jwt-audience", "", "JWT audience for authentication")
	cmd.Flags().StringSliceVar(&opts.AllowedRoles, "allowed-roles", nil, "Allowed roles for org mode")
	cmd.Flags().StringToStringVar(&opts.Variables, "var", nil, "Set variable (can be used multiple times)")

	return cmd
}

func runDeploy(ctx context.Context, opts *DeployOptions) error {
	// Auto-detect config file if not specified
	if opts.ConfigFile == "" {
		for _, file := range []string{"ftl.yaml", "ftl.yml", "ftl.json", "app.cue"} {
			if _, err := os.Stat(file); err == nil {
				opts.ConfigFile = file
				break
			}
		}
		if opts.ConfigFile == "" {
			return fmt.Errorf("no FTL configuration file found (ftl.yaml, ftl.json, or app.cue)")
		}
	}

	// Load and parse configuration
	Info("Loading configuration from %s", opts.ConfigFile)
	ftlApp, err := loadFTLApplication(opts.ConfigFile)
	if err != nil {
		return fmt.Errorf("failed to load configuration: %w", err)
	}

	// Apply command-line overrides
	if opts.AccessControl != "" {
		ftlApp.Access = opts.AccessControl
	}
	if opts.JWTIssuer != "" {
		ftlApp.Auth.Provider = "custom"
		ftlApp.Auth.JWTIssuer = opts.JWTIssuer
		if opts.JWTAudience != "" {
			ftlApp.Auth.JWTAudience = opts.JWTAudience
		}
	}

	// Find local components that need building
	localComponents := findLocalComponents(ftlApp)
	
	if len(localComponents) > 0 && !opts.DryRun {
		Info("Building %d local component(s)", len(localComponents))
		if err := buildLocalComponents(ctx, localComponents); err != nil {
			return fmt.Errorf("failed to build components: %w", err)
		}
		Success("All components built successfully")
		fmt.Println()
	}

	// Initialize auth manager
	authManager := auth.NewManager()
	
	// Check authentication
	if _, err := authManager.GetToken(ctx); err != nil {
		return fmt.Errorf("not logged in to FTL. Run 'ftl auth login' first")
	}

	// Create API client
	apiClient, err := api.NewFTLClient(authManager, "")
	if err != nil {
		return fmt.Errorf("failed to create API client: %w", err)
	}

	// Check if app exists
	appName := ftlApp.Name
	apps, err := apiClient.ListApps(ctx, &api.ListAppsParams{
		AppName: &appName,
	})
	if err != nil {
		return fmt.Errorf("failed to check existing apps: %w", err)
	}

	var appID string
	appExists := len(apps.Apps) > 0
	
	if appExists {
		appID = apps.Apps[0].AppId.String()
		if !opts.Yes && !opts.DryRun {
			Info("Found existing app '%s'", appName)
			if !promptConfirm("Update existing app?", true) {
				return fmt.Errorf("deployment cancelled")
			}
		}
	} else {
		if !opts.Yes && !opts.DryRun {
			Info("Creating new app '%s'", appName)
			if !promptConfirm("Continue?", true) {
				return fmt.Errorf("deployment cancelled")
			}
		}
	}

	if opts.DryRun {
		displayDryRunSummary(ftlApp, localComponents, appExists)
		return nil
	}

	// Create app if it doesn't exist
	if !appExists {
		Info("Creating app on FTL platform...")
		
		accessControl := api.CreateAppRequestAccessControlPublic
		switch ftlApp.Access {
		case "private":
			accessControl = api.CreateAppRequestAccessControlPrivate
		case "org":
			accessControl = api.CreateAppRequestAccessControlOrg
		case "custom":
			accessControl = api.CreateAppRequestAccessControlCustom
		}
		
		createReq := api.CreateAppRequest{
			AppName:       appName,
			AccessControl: &accessControl,
		}
		
		createResp, err := apiClient.CreateApp(ctx, createReq)
		if err != nil {
			return fmt.Errorf("failed to create app: %w", err)
		}
		appID = createResp.AppId.String()
		Success("App created with ID: %s", appID)
	}

	// Get ECR credentials
	Info("Getting registry credentials...")
	ecrToken, err := apiClient.CreateECRToken(ctx, appID)
	if err != nil {
		return fmt.Errorf("failed to get ECR token: %w", err)
	}

	// Docker login to ECR
	if err := dockerLoginToECR(ctx, ecrToken); err != nil {
		return fmt.Errorf("failed to login to registry: %w", err)
	}
	Success("Logged in to registry")

	// Push local components to registry
	if len(localComponents) > 0 {
		Info("Pushing %d component(s) to registry", len(localComponents))
		pushedComponents, err := pushComponentsToRegistry(ctx, ftlApp, localComponents, ecrToken.RegistryUri, appID)
		if err != nil {
			return fmt.Errorf("failed to push components: %w", err)
		}
		
		// Update ftlApp with registry references for pushed components
		for _, pushed := range pushedComponents {
			for i, comp := range ftlApp.Components {
				if comp.ID == pushed.ID {
					ftlApp.Components[i].Source = pushed.RegistrySource
					break
				}
			}
		}
		Success("All components pushed to registry")
		fmt.Println()
	}

	// Create deployment request with the FTL configuration
	Info("Creating deployment...")
	
	deploymentReq := createDeploymentRequest(ftlApp, opts)
	
	// Send deployment request
	deployment, err := apiClient.CreateDeployment(ctx, appID, deploymentReq)
	if err != nil {
		return fmt.Errorf("failed to create deployment: %w", err)
	}

	// Poll for deployment status
	sp := spinner.New(spinner.CharSets[14], 100*time.Millisecond)
	sp.Suffix = " Waiting for deployment to complete..."
	sp.Start()
	
	deployed, err := waitForDeployment(ctx, apiClient, appID, deployment.DeploymentId, sp)
	if err != nil {
		sp.Stop()
		return fmt.Errorf("deployment failed: %w", err)
	}
	
	sp.Stop()
	Success("Deployment completed successfully!")
	
	if deployed.ProviderUrl != nil && *deployed.ProviderUrl != "" {
		fmt.Println()
		fmt.Printf("  MCP URL: %s\n", *deployed.ProviderUrl)
		fmt.Println()
	}

	return nil
}

// loadFTLApplication loads the FTL application configuration from various formats
func loadFTLApplication(configFile string) (*FTLApplication, error) {
	ext := filepath.Ext(configFile)
	
	switch ext {
	case ".yaml", ".yml":
		return loadYAMLConfig(configFile)
	case ".json":
		return loadJSONConfig(configFile)
	case ".cue":
		return loadCUEConfig(configFile)
	default:
		return nil, fmt.Errorf("unsupported config format: %s", ext)
	}
}

// FTLApplication represents the FTL application configuration
// This matches the CUE schema in patterns.cue
type FTLApplication struct {
	Name        string          `json:"name" yaml:"name"`
	Version     string          `json:"version" yaml:"version"`
	Description string          `json:"description" yaml:"description"`
	Components  []FTLComponent  `json:"components" yaml:"components"`
	Access      string          `json:"access" yaml:"access"`
	Auth        FTLAuthConfig   `json:"auth" yaml:"auth"`
}

type FTLComponent struct {
	ID       string                 `json:"id" yaml:"id"`
	Source   interface{}            `json:"source" yaml:"source"`
	Build    *FTLBuildConfig        `json:"build,omitempty" yaml:"build,omitempty"`
	Variables map[string]string     `json:"variables,omitempty" yaml:"variables,omitempty"`
	
	// Internal field for tracking registry source after push
	RegistrySource interface{} `json:"-" yaml:"-"`
}

type FTLBuildConfig struct {
	Command string   `json:"command" yaml:"command"`
	Workdir string   `json:"workdir,omitempty" yaml:"workdir,omitempty"`
	Watch   []string `json:"watch,omitempty" yaml:"watch,omitempty"`
}

type FTLAuthConfig struct {
	Provider     string `json:"provider" yaml:"provider"`
	OrgID        string `json:"org_id,omitempty" yaml:"org_id,omitempty"`
	JWTIssuer    string `json:"jwt_issuer,omitempty" yaml:"jwt_issuer,omitempty"`
	JWTAudience  string `json:"jwt_audience,omitempty" yaml:"jwt_audience,omitempty"`
}

func loadYAMLConfig(configFile string) (*FTLApplication, error) {
	data, err := os.ReadFile(configFile)
	if err != nil {
		return nil, err
	}
	
	// First parse as generic config to extract application info
	var cfg config.FTLConfig
	if err := yaml.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse YAML: %w", err)
	}
	
	// Convert to FTLApplication format
	app := &FTLApplication{
		Name:        cfg.Application.Name,
		Version:     cfg.Application.Version,
		Description: cfg.Application.Description,
		Components:  make([]FTLComponent, 0, len(cfg.Components)),
		Access:      "public",
		Auth: FTLAuthConfig{
			Provider: "workos",
		},
	}
	
	// Convert MCP config to access/auth settings
	if cfg.MCP != nil && cfg.MCP.Authorizer != nil {
		if cfg.MCP.Authorizer.AccessControl != "" {
			app.Access = cfg.MCP.Authorizer.AccessControl
		}
		if cfg.MCP.Authorizer.JWTIssuer != "" {
			app.Auth.JWTIssuer = cfg.MCP.Authorizer.JWTIssuer
		}
		if cfg.MCP.Authorizer.JWTAudience != "" {
			app.Auth.JWTAudience = cfg.MCP.Authorizer.JWTAudience
		}
		if cfg.MCP.Authorizer.OrgID != "" {
			app.Auth.OrgID = cfg.MCP.Authorizer.OrgID
		}
	}
	
	// Convert components
	for _, comp := range cfg.Components {
		ftlComp := FTLComponent{
			ID:        comp.ID,
			Source:    comp.Source,
			Variables: comp.Variables,
		}
		
		if comp.Build != nil {
			ftlComp.Build = &FTLBuildConfig{
				Command: comp.Build.Command,
				Workdir: comp.Build.Workdir,
				Watch:   comp.Build.Watch,
			}
		}
		
		app.Components = append(app.Components, ftlComp)
	}
	
	// Set defaults
	if app.Version == "" {
		app.Version = "0.1.0"
	}
	
	return app, nil
}

func loadJSONConfig(configFile string) (*FTLApplication, error) {
	data, err := os.ReadFile(configFile)
	if err != nil {
		return nil, err
	}
	
	var app FTLApplication
	if err := json.Unmarshal(data, &app); err != nil {
		return nil, fmt.Errorf("failed to parse JSON: %w", err)
	}
	
	// Set defaults
	if app.Version == "" {
		app.Version = "0.1.0"
	}
	if app.Access == "" {
		app.Access = "public"
	}
	
	return &app, nil
}

func loadCUEConfig(configFile string) (*FTLApplication, error) {
	// Use the synthesizer to extract the FTL application from CUE
	synth := synthesis.NewSynthesizer()
	
	cueData, err := os.ReadFile(configFile)
	if err != nil {
		return nil, err
	}
	
	// For now, we need to synthesize and then extract back
	// In the future, we should extract the FTL app directly from CUE
	_, err = synth.SynthesizeCUE(string(cueData))
	if err != nil {
		return nil, fmt.Errorf("failed to parse CUE config: %w", err)
	}
	
	// TODO: Extract FTLApplication directly from CUE value
	// For now, return a minimal config
	return &FTLApplication{
		Name:    "app",
		Version: "0.1.0",
		Access:  "public",
		Auth: FTLAuthConfig{
			Provider: "workos",
		},
		Components: []FTLComponent{},
	}, nil
}

func findLocalComponents(app *FTLApplication) []FTLComponent {
	var local []FTLComponent
	
	for _, comp := range app.Components {
		// Check if source is a string (local path) vs map (registry reference)
		if _, ok := comp.Source.(string); ok {
			local = append(local, comp)
		}
	}
	
	return local
}

func buildLocalComponents(ctx context.Context, components []FTLComponent) error {
	for _, comp := range components {
		if comp.Build == nil || comp.Build.Command == "" {
			continue
		}
		
		Info("Building component '%s'", comp.ID)
		
		// Determine working directory
		workdir := "."
		if comp.Build.Workdir != "" {
			workdir = comp.Build.Workdir
		}
		
		// Execute build command
		cmd := exec.CommandContext(ctx, "sh", "-c", comp.Build.Command)
		cmd.Dir = workdir
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		
		if err := cmd.Run(); err != nil {
			return fmt.Errorf("failed to build component %s: %w", comp.ID, err)
		}
	}
	
	return nil
}

func dockerLoginToECR(ctx context.Context, ecrToken *api.CreateEcrTokenResponseBody) error {
	// Decode the authorization token
	decoded, err := base64.StdEncoding.DecodeString(ecrToken.AuthorizationToken)
	if err != nil {
		return fmt.Errorf("failed to decode ECR token: %w", err)
	}
	
	// Extract password (format is "AWS:password")
	parts := strings.SplitN(string(decoded), ":", 2)
	if len(parts) != 2 || parts[0] != "AWS" {
		return fmt.Errorf("invalid ECR token format")
	}
	password := parts[1]
	
	// Run docker login
	cmd := exec.CommandContext(ctx, "docker", "login", 
		"--username", "AWS",
		"--password-stdin",
		ecrToken.RegistryUri)
	
	cmd.Stdin = strings.NewReader(password)
	
	var stderr bytes.Buffer
	cmd.Stderr = &stderr
	
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("docker login failed: %s", stderr.String())
	}
	
	return nil
}

type PushedComponent struct {
	ID             string
	RegistrySource map[string]interface{}
}

func pushComponentsToRegistry(ctx context.Context, app *FTLApplication, components []FTLComponent, registryURI string, appID string) ([]PushedComponent, error) {
	var pushed []PushedComponent
	
	for _, comp := range components {
		sourcePath, ok := comp.Source.(string)
		if !ok {
			continue
		}
		
		// Determine version (default to app version)
		version := app.Version
		
		// Construct the registry reference
		// Format: registry/namespace:package@version
		registryRef := fmt.Sprintf("%s/%s:%s@%s", 
			registryURI,
			appID,
			comp.ID,
			version)
		
		Info("Pushing component '%s' to %s", comp.ID, registryRef)
		
		// Use spin deps publish to push the component
		cmd := exec.CommandContext(ctx, "spin", "deps", "publish",
			"--registry", registryURI,
			"--package", fmt.Sprintf("%s:%s@%s", appID, comp.ID, version),
			sourcePath)
		
		var stderr bytes.Buffer
		cmd.Stderr = &stderr
		
		if err := cmd.Run(); err != nil {
			return nil, fmt.Errorf("failed to push component %s: %s", comp.ID, stderr.String())
		}
		
		// Create registry source reference
		pushed = append(pushed, PushedComponent{
			ID: comp.ID,
			RegistrySource: map[string]interface{}{
				"registry": registryURI,
				"package":  fmt.Sprintf("%s:%s", appID, comp.ID),
				"version":  version,
			},
		})
	}
	
	return pushed, nil
}

func createDeploymentRequest(app *FTLApplication, opts *DeployOptions) api.CreateDeploymentRequest {
	req := api.CreateDeploymentRequest{
		Variables: opts.Variables,
	}
	
	// Set access control
	switch app.Access {
	case "private":
		ac := api.CreateDeploymentRequestAccessControlPrivate
		req.AccessControl = &ac
	case "org":
		ac := api.CreateDeploymentRequestAccessControlOrg
		req.AccessControl = &ac
	case "custom":
		ac := api.CreateDeploymentRequestAccessControlCustom
		req.AccessControl = &ac
	default:
		ac := api.CreateDeploymentRequestAccessControlPublic
		req.AccessControl = &ac
	}
	
	// Set allowed roles for org mode
	if app.Access == "org" && len(opts.AllowedRoles) > 0 {
		req.AllowedRoles = &opts.AllowedRoles
	}
	
	// Convert FTLApplication to a format the API expects
	// The API should accept the FTL config schema directly
	configData := map[string]interface{}{
		"name":        app.Name,
		"version":     app.Version,
		"description": app.Description,
		"components":  app.Components,
		"access":      app.Access,
		"auth":        app.Auth,
	}
	
	// Marshal to JSON and back to ensure proper types
	jsonData, _ := json.Marshal(configData)
	var components []api.CreateDeploymentRequestComponentsItem
	
	// For now, convert to the expected format
	// TODO: Update API to accept FTL config directly
	for _, comp := range app.Components {
		item := api.CreateDeploymentRequestComponentsItem{
			ComponentName: comp.ID,
		}
		
		// Add registry info if available
		if regSource, ok := comp.RegistrySource.(map[string]interface{}); ok {
			if version, ok := regSource["version"].(string); ok {
				item.Tag = &version
			}
		}
		
		components = append(components, item)
	}
	
	req.Components = components
	
	// Add custom auth if needed
	if app.Access == "custom" && app.Auth.Provider == "custom" {
		req.CustomAuth = &api.CreateDeploymentRequestCustomAuth{
			Issuer:   app.Auth.JWTIssuer,
			Audience: []string{app.Auth.JWTAudience},
		}
	}
	
	return req
}

func waitForDeployment(ctx context.Context, client *api.FTLClient, appID string, deploymentID string, sp *spinner.Spinner) (*api.App, error) {
	maxAttempts := 60 // 5 minutes with 5-second intervals
	
	for i := 0; i < maxAttempts; i++ {
		app, err := client.GetApp(ctx, appID)
		if err != nil {
			return nil, fmt.Errorf("failed to get app status: %w", err)
		}
		
		switch app.Status {
		case api.AppStatusActive:
			return app, nil
		case api.AppStatusFailed:
			errMsg := "deployment failed"
			if app.ProviderError != nil {
				errMsg = *app.ProviderError
			}
			return nil, fmt.Errorf(errMsg)
		case api.AppStatusDeleted, api.AppStatusDeleting:
			return nil, fmt.Errorf("app was deleted during deployment")
		default:
			// Still pending or creating
			sp.Suffix = fmt.Sprintf(" Deployment in progress... (%s)", app.Status)
		}
		
		time.Sleep(5 * time.Second)
	}
	
	return nil, fmt.Errorf("deployment timeout after 5 minutes")
}

func displayDryRunSummary(app *FTLApplication, localComponents []FTLComponent, appExists bool) {
	fmt.Println()
	fmt.Println("🔍 DRY RUN MODE - No changes will be made")
	fmt.Println()
	
	color.Cyan("Application Configuration:")
	fmt.Printf("  Name: %s\n", app.Name)
	fmt.Printf("  Version: %s\n", app.Version)
	if app.Description != "" {
		fmt.Printf("  Description: %s\n", app.Description)
	}
	fmt.Printf("  Access Control: %s\n", app.Access)
	
	if app.Auth.Provider == "custom" {
		fmt.Printf("  Auth Provider: Custom\n")
		fmt.Printf("  JWT Issuer: %s\n", app.Auth.JWTIssuer)
		if app.Auth.JWTAudience != "" {
			fmt.Printf("  JWT Audience: %s\n", app.Auth.JWTAudience)
		}
	} else if app.Access == "private" || app.Access == "org" {
		fmt.Printf("  Auth Provider: WorkOS\n")
		if app.Auth.OrgID != "" {
			fmt.Printf("  Organization ID: %s\n", app.Auth.OrgID)
		}
	}
	
	fmt.Println()
	color.Cyan("Components:")
	for _, comp := range app.Components {
		fmt.Printf("  • %s\n", comp.ID)
		
		// Show source type
		if sourcePath, ok := comp.Source.(string); ok {
			fmt.Printf("    Source: %s (local)\n", sourcePath)
			if comp.Build != nil && comp.Build.Command != "" {
				fmt.Printf("    Build: %s\n", comp.Build.Command)
			}
		} else if sourceMap, ok := comp.Source.(map[string]interface{}); ok {
			if registry, ok := sourceMap["registry"].(string); ok {
				fmt.Printf("    Source: %s (registry)\n", registry)
			}
		}
	}
	
	fmt.Println()
	color.Cyan("Actions that would be performed:")
	
	if len(localComponents) > 0 {
		fmt.Printf("  ✓ Build %d local component(s)\n", len(localComponents))
	}
	
	if appExists {
		fmt.Printf("  ✓ Update existing app\n")
	} else {
		fmt.Printf("  ✓ Create new app\n")
	}
	
	if len(localComponents) > 0 {
		fmt.Printf("  ✓ Push %d component(s) to registry\n", len(localComponents))
	}
	
	fmt.Printf("  ✓ Create deployment with FTL configuration\n")
	fmt.Printf("  ✓ Platform will synthesize Spin manifest and deploy\n")
	
	fmt.Println()
	fmt.Println("To perform the actual deployment, run without --dry-run")
}

func promptConfirm(message string, defaultYes bool) bool {
	prompt := &survey.Confirm{
		Message: message,
		Default: defaultYes,
	}
	
	var result bool
	if err := survey.AskOne(prompt, &result); err != nil {
		return false
	}
	
	return result
}