package cmd

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"time"

	"github.com/AlecAivazis/survey/v2"
	"github.com/briandowns/spinner"
	"github.com/fatih/color"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"

	"github.com/fastertools/ftl-cli/go/shared/api"
	"github.com/fastertools/ftl-cli/go/shared/auth"
	"github.com/fastertools/ftl-cli/go/shared/config"
	"github.com/fastertools/ftl-cli/go/shared/ftl"
)

// DeployOptions holds options for the deploy command
type DeployOptions struct {
	Environment   string
	ConfigFile    string
	DryRun        bool
	Yes           bool
	AccessControl string
	JWTIssuer     string
	JWTAudience   string
	AllowedRoles  []string
	Variables     map[string]string
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
		ftlApp.Access = ftl.AccessMode(opts.AccessControl)
	}
	if opts.JWTIssuer != "" {
		ftlApp.Auth.Provider = ftl.AuthProviderCustom
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
	store, err := auth.NewKeyringStore()
	if err != nil {
		return fmt.Errorf("failed to initialize credential store: %w", err)
	}
	authManager := auth.NewManager(store, nil)

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
		Name: &appName,
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
	// Extract component names to pass to ECR token creation
	componentNames := make([]string, 0, len(localComponents))
	for _, comp := range localComponents {
		componentNames = append(componentNames, comp.ID)
	}
	ecrToken, err := apiClient.CreateECRToken(ctx, appID, componentNames)
	if err != nil {
		return fmt.Errorf("failed to get ECR token: %w", err)
	}

	// Update wkg config with ECR credentials for spin deps
	// Use the shared ftl package utility that the backend can also use
	if err := ftl.UpdateWkgAuthForECR("", ecrToken.RegistryUri, ecrToken.AuthorizationToken); err != nil {
		return fmt.Errorf("failed to configure wkg auth: %w", err)
	}
	Success("Configured registry authentication")

	// Push local components to registry
	if len(localComponents) > 0 {
		Info("Pushing %d component(s) to registry", len(localComponents))
		// Use the sanitized packageNamespace from the ECR token
		if ecrToken.PackageNamespace == nil || *ecrToken.PackageNamespace == "" {
			return fmt.Errorf("ECR token response missing required packageNamespace field - backend may need updating")
		}
		namespace := *ecrToken.PackageNamespace
		pushedComponents, err := pushComponentsToRegistry(ctx, ftlApp, localComponents, ecrToken.RegistryUri, namespace)
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

	// Optional: Clean up ECR auth from wkg config
	// (ECR tokens expire after 12 hours anyway, so this is optional)
	// removeWkgAuthForECR(ecrToken.RegistryUri)

	return nil
}

// loadFTLApplication loads the FTL application configuration from various formats
func loadFTLApplication(configFile string) (*ftl.Application, error) {
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

func loadYAMLConfig(configFile string) (*ftl.Application, error) {
	data, err := os.ReadFile(configFile)
	if err != nil {
		return nil, err
	}

	// First parse as generic config to extract application info
	var cfg config.FTLConfig
	if err := yaml.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse YAML: %w", err)
	}

	// Convert to ftl.Application format
	app := &ftl.Application{
		Name:        cfg.Application.Name,
		Version:     cfg.Application.Version,
		Description: cfg.Application.Description,
		Components:  make([]ftl.Component, 0, len(cfg.Components)),
		Access:      ftl.AccessPublic,
		Auth: ftl.AuthConfig{
			Provider: ftl.AuthProviderWorkOS,
		},
	}

	// Convert MCP config to access/auth settings
	if cfg.MCP != nil && cfg.MCP.Authorizer != nil {
		if cfg.MCP.Authorizer.AccessControl != "" {
			app.Access = ftl.AccessMode(cfg.MCP.Authorizer.AccessControl)
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
		// Determine source type
		var source ftl.ComponentSource
		if srcStr, ok := comp.Source.(string); ok {
			source = ftl.LocalSource(srcStr)
		} else if srcMap, ok := comp.Source.(map[string]interface{}); ok {
			// Registry source
			source = &ftl.RegistrySource{
				Registry: srcMap["registry"].(string),
				Package:  srcMap["package"].(string),
				Version:  srcMap["version"].(string),
			}
		}

		ftlComp := ftl.Component{
			ID:        comp.ID,
			Source:    source,
			Variables: comp.Variables,
		}

		if comp.Build != nil {
			ftlComp.Build = &ftl.BuildConfig{
				Command: comp.Build.Command,
				Workdir: comp.Build.Workdir,
				Watch:   comp.Build.Watch,
			}
		}

		app.Components = append(app.Components, ftlComp)
	}

	// Set defaults using the shared package method
	app.SetDefaults()

	return app, nil
}

func loadJSONConfig(configFile string) (*ftl.Application, error) {
	data, err := os.ReadFile(configFile)
	if err != nil {
		return nil, err
	}

	var app ftl.Application
	if err := json.Unmarshal(data, &app); err != nil {
		return nil, fmt.Errorf("failed to parse JSON: %w", err)
	}

	// Set defaults
	app.SetDefaults()

	return &app, nil
}

func loadCUEConfig(configFile string) (*ftl.Application, error) {
	// TODO: Properly extract from CUE
	// For now, return a minimal config
	return &ftl.Application{
		Name:    "app",
		Version: "0.1.0",
		Access:  ftl.AccessPublic,
		Auth: ftl.AuthConfig{
			Provider: ftl.AuthProviderWorkOS,
		},
		Components: []ftl.Component{},
	}, nil
}

func findLocalComponents(app *ftl.Application) []ftl.Component {
	var local []ftl.Component

	for _, comp := range app.Components {
		// Use the helper to check if it's local
		if comp.Source != nil && comp.Source.IsLocal() {
			local = append(local, comp)
		}
	}

	return local
}

func buildLocalComponents(ctx context.Context, components []ftl.Component) error {
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

// dockerLoginToECR is no longer used - we update wkg config instead
// Keeping for reference in case we need Docker operations in the future
/*
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
*/

type PushedComponent struct {
	ID             string
	RegistrySource ftl.ComponentSource
}

func pushComponentsToRegistry(ctx context.Context, app *ftl.Application, components []ftl.Component, registryURI string, namespace string) ([]PushedComponent, error) {
	var pushed []PushedComponent

	for _, comp := range components {
		// Get the local source path
		sourcePath, ok := ftl.AsLocal(comp.Source)
		if !ok {
			continue
		}

		// Determine version (default to app version)
		version := app.Version

		// Construct the registry reference
		// Format: registry/namespace:package@version
		registryRef := fmt.Sprintf("%s/%s:%s@%s",
			registryURI,
			namespace,
			comp.ID,
			version)

		Info("Pushing component '%s' to %s", comp.ID, registryRef)

		// Use spin deps publish to push the component
		cmd := exec.CommandContext(ctx, "spin", "deps", "publish",
			"--registry", registryURI,
			"--package", fmt.Sprintf("%s:%s@%s", namespace, comp.ID, version),
			sourcePath)

		var stderr bytes.Buffer
		cmd.Stderr = &stderr

		if err := cmd.Run(); err != nil {
			return nil, fmt.Errorf("failed to push component %s: %s", comp.ID, stderr.String())
		}

		// Create registry source reference
		pushed = append(pushed, PushedComponent{
			ID: comp.ID,
			RegistrySource: &ftl.RegistrySource{
				Registry: registryURI,
				Package:  fmt.Sprintf("%s:%s", namespace, comp.ID),
				Version:  version,
			},
		})
	}

	return pushed, nil
}

func createDeploymentRequest(app *ftl.Application, opts *DeployOptions) api.CreateDeploymentRequest {
	req := api.CreateDeploymentRequest{
		Variables: &opts.Variables,
	}

	// Set the application details
	req.Application.Name = app.Name
	req.Application.Version = &app.Version
	if app.Description != "" {
		req.Application.Description = &app.Description
	}

	// Set access control
	switch app.Access {
	case ftl.AccessPrivate:
		ac := api.CreateDeploymentRequestApplicationAccessPrivate
		req.Application.Access = &ac
	case ftl.AccessOrg:
		ac := api.CreateDeploymentRequestApplicationAccessOrg
		req.Application.Access = &ac
	case ftl.AccessCustom:
		ac := api.CreateDeploymentRequestApplicationAccessCustom
		req.Application.Access = &ac
	default:
		ac := api.CreateDeploymentRequestApplicationAccessPublic
		req.Application.Access = &ac
	}

	// Set auth configuration if needed
	if app.Access == ftl.AccessOrg || app.Access == ftl.AccessCustom {
		req.Application.Auth = &struct {
			JwtAudience *string                                         `json:"jwt_audience,omitempty"`
			JwtIssuer   *string                                         `json:"jwt_issuer,omitempty"`
			OrgId       *string                                         `json:"org_id,omitempty"`
			Provider    *api.CreateDeploymentRequestApplicationAuthProvider `json:"provider,omitempty"`
		}{}

		if app.Auth.Provider == ftl.AuthProviderWorkOS {
			provider := api.CreateDeploymentRequestApplicationAuthProviderWorkos
			req.Application.Auth.Provider = &provider
		} else if app.Auth.Provider == ftl.AuthProviderCustom {
			provider := api.CreateDeploymentRequestApplicationAuthProviderCustom
			req.Application.Auth.Provider = &provider
		}

		if app.Auth.OrgID != "" {
			req.Application.Auth.OrgId = &app.Auth.OrgID
		}
		if app.Auth.JWTIssuer != "" {
			req.Application.Auth.JwtIssuer = &app.Auth.JWTIssuer
		}
		if app.Auth.JWTAudience != "" {
			req.Application.Auth.JwtAudience = &app.Auth.JWTAudience
		}
	}

	// Add components
	components := make([]struct {
		Id     string `json:"id"`
		Source struct {
			Package  string `json:"package"`
			Registry string `json:"registry"`
			Version  string `json:"version"`
		} `json:"source"`
		Variables *map[string]string `json:"variables,omitempty"`
	}, 0, len(app.Components))

	for _, comp := range app.Components {
		deployComp := struct {
			Id     string `json:"id"`
			Source struct {
				Package  string `json:"package"`
				Registry string `json:"registry"`
				Version  string `json:"version"`
			} `json:"source"`
			Variables *map[string]string `json:"variables,omitempty"`
		}{
			Id: comp.ID,
		}

		// Convert component source to API format - only registry sources are allowed
		if regSource, ok := ftl.AsRegistry(comp.Source); ok {
			// Set the structured registry reference
			deployComp.Source.Registry = regSource.Registry
			deployComp.Source.Package = regSource.Package
			deployComp.Source.Version = regSource.Version
		} else if _, ok := ftl.AsLocal(comp.Source); ok {
			// Local sources should have been replaced with registry sources after pushing
			// This shouldn't happen at deployment time
			return api.CreateDeploymentRequest{} // Return empty request to fail early
		}

		// Add component variables if any
		if len(comp.Variables) > 0 {
			deployComp.Variables = &comp.Variables
		}

		components = append(components, deployComp)
	}

	if len(components) > 0 {
		req.Application.Components = &components
	}

	// Add application variables if any
	if len(app.Variables) > 0 {
		req.Application.Variables = &app.Variables
	}

	// Set environment based on options
	switch opts.Environment {
	case "development":
		env := api.Development
		req.Environment = &env
	case "staging":
		env := api.Staging
		req.Environment = &env
	case "production":
		env := api.Production
		req.Environment = &env
	default:
		// Default to production if not specified
		env := api.Production
		req.Environment = &env
	}

	return req
}

func waitForDeployment(ctx context.Context, client *api.FTLClient, appID string, deploymentID string, sp *spinner.Spinner) (*api.App, error) {
	maxAttempts := 36 // 3 minutes with 5-second intervals

	for i := 0; i < maxAttempts; i++ {
		app, err := client.GetApp(ctx, appID)
		if err != nil {
			return nil, fmt.Errorf("failed to get app status: %w", err)
		}

		// Check the latest deployment status if available
		if app.LatestDeployment != nil {
			// Check if this is our deployment
			if app.LatestDeployment.DeploymentId == deploymentID {
				switch app.LatestDeployment.Status {
				case api.AppLatestDeploymentStatusDeployed:
					// Deployment succeeded
					return app, nil
				case api.AppLatestDeploymentStatusFailed:
					// Deployment failed
					errMsg := "deployment failed"
					if app.LatestDeployment.StatusMessage != nil && *app.LatestDeployment.StatusMessage != "" {
						errMsg = *app.LatestDeployment.StatusMessage
					}
					return nil, fmt.Errorf("%s", errMsg)
				case api.AppLatestDeploymentStatusPending, api.AppLatestDeploymentStatusDeploying:
					// Still in progress
					sp.Suffix = fmt.Sprintf(" Deployment in progress... (%s)", app.LatestDeployment.Status)
				default:
					// Unknown status, continue polling
					sp.Suffix = fmt.Sprintf(" Deployment in progress... (%s)", app.LatestDeployment.Status)
				}
			} else {
				// This is a different deployment, might be a race condition
				// Continue polling to see if our deployment shows up
				sp.Suffix = " Waiting for deployment to start..."
			}
		} else {
			// No deployment info yet, check app status as fallback
			switch app.Status {
			case api.AppStatusACTIVE:
				// App is active but no deployment info, consider it success
				return app, nil
			case api.AppStatusFAILED:
				errMsg := "app failed"
				if app.ProviderError != nil {
					errMsg = *app.ProviderError
				}
				return nil, fmt.Errorf("%s", errMsg)
			case api.AppStatusDELETED, api.AppStatusDELETING:
				return nil, fmt.Errorf("app was deleted during deployment")
			default:
				// Still pending or creating
				sp.Suffix = fmt.Sprintf(" Waiting for deployment... (app: %s)", app.Status)
			}
		}

		time.Sleep(5 * time.Second)
	}

	return nil, fmt.Errorf("deployment timeout after 3 minutes")
}

func displayDryRunSummary(app *ftl.Application, localComponents []ftl.Component, appExists bool) {
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
		if sourcePath, ok := ftl.AsLocal(comp.Source); ok {
			fmt.Printf("    Source: %s (local)\n", sourcePath)
			if comp.Build != nil && comp.Build.Command != "" {
				fmt.Printf("    Build: %s\n", comp.Build.Command)
			}
		} else if regSource, ok := ftl.AsRegistry(comp.Source); ok {
			fmt.Printf("    Source: %s (registry)\n", regSource.Registry)
			if regSource.Package != "" {
				fmt.Printf("    Package: %s\n", regSource.Package)
			}
			if regSource.Version != "" {
				fmt.Printf("    Version: %s\n", regSource.Version)
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
