package cli

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/AlecAivazis/survey/v2"
	"github.com/briandowns/spinner"
	"github.com/fatih/color"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"

	"github.com/fastertools/ftl-cli/internal/api"
	"github.com/fastertools/ftl-cli/internal/auth"
	"github.com/fastertools/ftl-cli/pkg/oci"
	ftltypes "github.com/fastertools/ftl-cli/pkg/types"
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

	// First synthesize spin.toml from the FTL configuration
	Info("Synthesizing Spin manifest from %s", opts.ConfigFile)
	if err := runSynth(ctx, opts.ConfigFile); err != nil {
		return fmt.Errorf("failed to synthesize spin.toml: %w", err)
	}
	Success("Generated spin.toml")

	// Load and parse configuration
	manifest, err := loadDeployManifest(opts.ConfigFile)
	if err != nil {
		return fmt.Errorf("failed to load configuration: %w", err)
	}

	// Apply command-line overrides
	if opts.AccessControl != "" {
		manifest.Access = opts.AccessControl
	}
	if opts.JWTIssuer != "" {
		if manifest.Auth == nil {
			manifest.Auth = &ftltypes.Auth{}
		}
		manifest.Auth.JWTIssuer = opts.JWTIssuer
		if opts.JWTAudience != "" {
			manifest.Auth.JWTAudience = opts.JWTAudience
		}
	}

	// Run spin build to build all local components
	if !opts.DryRun {
		Info("Building local components with 'spin build'")
		cmd := ExecCommand("spin", "build")
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		if err := cmd.Run(); err != nil {
			return fmt.Errorf("failed to build components: %w", err)
		}
		Success("All local components built successfully")
		fmt.Println()
	}

	// Dry-run mode: validate configuration without authentication
	if opts.DryRun {
		displayDryRunSummary(manifest, false)
		return nil
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
	appName := manifest.Application.Name
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
		if !opts.Yes {
			Info("Found existing app '%s'", appName)
			if !promptConfirm("Update existing app?", true) {
				return fmt.Errorf("deployment cancelled")
			}
		}
	} else {
		if !opts.Yes {
			Info("Creating new app '%s'", appName)
			if !promptConfirm("Continue?", true) {
				return fmt.Errorf("deployment cancelled")
			}
		}
	}

	// Create app if it doesn't exist
	if !appExists {
		Info("Creating app on FTL platform...")

		accessControl := api.CreateAppRequestAccessControlPublic
		switch manifest.Access {
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
	componentNames := make([]string, 0, len(manifest.Components))
	for _, comp := range manifest.Components {
		componentNames = append(componentNames, comp.ID)
	}
	ecrToken, err := apiClient.CreateECRToken(ctx, appID, componentNames)
	if err != nil {
		return fmt.Errorf("failed to get ECR token: %w", err)
	}

	// Parse ECR credentials
	ecrAuth, err := oci.ParseECRToken(ecrToken.RegistryUri, ecrToken.AuthorizationToken)
	if err != nil {
		return fmt.Errorf("failed to parse ECR token: %w", err)
	}

	// Process components: pull registry components and push everything to ECR
	if ecrToken.PackageNamespace == nil || *ecrToken.PackageNamespace == "" {
		return fmt.Errorf("ECR token response missing required packageNamespace field")
	}
	namespace := *ecrToken.PackageNamespace

	Info("Processing components...")
	processedManifest, err := processComponents(ctx, manifest, ecrAuth, namespace)
	if err != nil {
		return fmt.Errorf("failed to process components: %w", err)
	}
	Success("All components processed and pushed to ECR")
	fmt.Println()

	// Create deployment request with the processed manifest
	Info("Creating deployment...")

	deploymentReq := createDeploymentRequest(processedManifest, opts)

	// Send deployment request
	deployment, err := apiClient.CreateDeployment(ctx, appID, deploymentReq)
	if err != nil {
		return fmt.Errorf("failed to create deployment: %w", err)
	}

	// Poll for deployment status
	sp := spinner.New(spinner.CharSets[14], 100*time.Millisecond)
	sp.Suffix = " Waiting for deployment to start..."
	sp.Start()

	deployed, err := waitForDeployment(ctx, apiClient, appID, deployment.DeploymentId, sp)
	if err != nil {
		sp.Stop()
		return fmt.Errorf("deployment failed: %w", err)
	}

	sp.Stop()
	Success("Deployment completed successfully!")

	if deployed.ProviderUrl != nil && *deployed.ProviderUrl != "" {
		// Display MCP URLs for the deployed application
		displayMCPUrls(*deployed.ProviderUrl, processedManifest.Components)
	}

	return nil
}

// loadDeployManifest loads the FTL manifest configuration for deployment
func loadDeployManifest(configFile string) (*ftltypes.Manifest, error) {
	// Clean the path to prevent directory traversal
	configFile = filepath.Clean(configFile)
	data, err := os.ReadFile(configFile)
	if err != nil {
		return nil, err
	}

	var manifest ftltypes.Manifest
	if err := yaml.Unmarshal(data, &manifest); err != nil {
		return nil, fmt.Errorf("failed to parse manifest: %w", err)
	}

	return &manifest, nil
}

// runSynth runs the synth command to generate spin.toml
func runSynth(ctx context.Context, configFile string) error {
	cmd := ExecCommand("ftl", "synth", "-o", "spin.toml", configFile)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}


// processComponents handles pulling registry components and pushing everything to ECR
func processComponents(ctx context.Context, manifest *ftltypes.Manifest, ecrAuth *oci.ECRAuth, namespace string) (*ftltypes.Manifest, error) {
	// Create output manifest with ECR references
	processedManifest := &ftltypes.Manifest{
		Application: manifest.Application,
		Access:      manifest.Access,
		Auth:        manifest.Auth,
		Variables:   manifest.Variables,
		Components:  make([]ftltypes.Component, 0, len(manifest.Components)),
	}

	// Create a WASMPuller for pulling registry components
	puller := oci.NewWASMPuller()

	// Create a WASMPusher for pushing to ECR
	pusher := oci.NewWASMPusher(ecrAuth)

	// Process each component
	for _, comp := range manifest.Components {
		var wasmPath string
		var err error

		// Check if it's a local or registry source
		if localPath, registrySource := ftltypes.ParseComponentSource(comp.Source); localPath != "" {
			// Local component - find the built WASM file
			wasmPath, err = findBuiltWASM(localPath, comp.ID)
			if err != nil {
				return nil, fmt.Errorf("failed to find built WASM for %s: %w", comp.ID, err)
			}
			Info("Found local component %s at %s", comp.ID, wasmPath)
		} else if registrySource != nil {
			// Registry component - pull it
			Info("Pulling component %s from %s", comp.ID, registrySource.Registry)
			wasmPath, err = puller.Pull(ctx, registrySource)
			if err != nil {
				return nil, fmt.Errorf("failed to pull component %s: %w", comp.ID, err)
			}
			Success("Pulled %s", comp.ID)
		} else {
			return nil, fmt.Errorf("invalid source for component %s", comp.ID)
		}

		// Push to ECR
		// Package name should use / not : for the repository path
		packageName := fmt.Sprintf("%s/%s", namespace, comp.ID)
		version := manifest.Application.Version

		Info("Pushing %s to ECR", comp.ID)
		if err := pusher.Push(ctx, wasmPath, packageName, version); err != nil {
			return nil, fmt.Errorf("failed to push component %s: %w", comp.ID, err)
		}
		Success("Pushed %s", comp.ID)

		// Create processed component with ECR reference
		// Convert package name from namespace/component to namespace:component for Spin compatibility
		spinPackageName := strings.Replace(packageName, "/", ":", 1)
		processedComp := ftltypes.Component{
			ID: comp.ID,
			Source: map[string]interface{}{
				"registry": ecrAuth.Registry,
				"package":  spinPackageName,
				"version":  version,
			},
			Build:     comp.Build,
			Variables: comp.Variables,
		}
		processedManifest.Components = append(processedManifest.Components, processedComp)
	}

	return processedManifest, nil
}

// findBuiltWASM locates the built WASM file for a local component
func findBuiltWASM(sourcePath, componentID string) (string, error) {
	// Check if sourcePath is already a .wasm file
	if strings.HasSuffix(sourcePath, ".wasm") {
		if _, err := os.Stat(sourcePath); err == nil {
			return sourcePath, nil
		}
	}

	// Look in common build output locations
	possiblePaths := []string{
		filepath.Join(sourcePath, componentID+".wasm"),
		filepath.Join(sourcePath, "target", "wasm32-wasip2", "release", componentID+".wasm"),
		filepath.Join(sourcePath, "target", "wasm32-wasi", "release", componentID+".wasm"),
		filepath.Join(sourcePath, "build", componentID+".wasm"),
		filepath.Join(sourcePath, "dist", componentID+".wasm"),
		componentID + ".wasm",
	}

	for _, path := range possiblePaths {
		if _, err := os.Stat(path); err == nil {
			return path, nil
		}
	}

	return "", fmt.Errorf("could not find built WASM file for component %s", componentID)
}


func createDeploymentRequest(manifest *ftltypes.Manifest, opts *DeployOptions) api.CreateDeploymentRequest {
	req := api.CreateDeploymentRequest{
		Variables: &opts.Variables,
	}

	// Set the application details
	req.Application.Name = manifest.Application.Name
	req.Application.Version = &manifest.Application.Version
	if manifest.Application.Description != "" {
		req.Application.Description = &manifest.Application.Description
	}

	// Set access control
	switch manifest.Access {
	case "private":
		ac := api.CreateDeploymentRequestApplicationAccessPrivate
		req.Application.Access = &ac
	case "org":
		ac := api.CreateDeploymentRequestApplicationAccessOrg
		req.Application.Access = &ac
	case "custom":
		ac := api.CreateDeploymentRequestApplicationAccessCustom
		req.Application.Access = &ac
	default:
		ac := api.CreateDeploymentRequestApplicationAccessPublic
		req.Application.Access = &ac
	}

	// Set auth configuration if needed
	if manifest.Auth != nil && (manifest.Access == "org" || manifest.Access == "custom") {
		req.Application.Auth = &struct {
			JwtAudience *string                                             `json:"jwt_audience,omitempty"`
			JwtIssuer   *string                                             `json:"jwt_issuer,omitempty"`
			OrgId       *string                                             `json:"org_id,omitempty"`
			Provider    *api.CreateDeploymentRequestApplicationAuthProvider `json:"provider,omitempty"`
		}{}

		// Default to custom provider when JWT settings are present
		if manifest.Auth.JWTIssuer != "" {
			provider := api.CreateDeploymentRequestApplicationAuthProviderCustom
			req.Application.Auth.Provider = &provider
			req.Application.Auth.JwtIssuer = &manifest.Auth.JWTIssuer
		}
		if manifest.Auth.JWTAudience != "" {
			req.Application.Auth.JwtAudience = &manifest.Auth.JWTAudience
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
	}, 0, len(manifest.Components))

	for _, comp := range manifest.Components {
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

		// Parse component source - should be registry at this point
		if _, registrySource := ftltypes.ParseComponentSource(comp.Source); registrySource != nil {
			deployComp.Source.Registry = registrySource.Registry
			deployComp.Source.Package = registrySource.Package
			deployComp.Source.Version = registrySource.Version
		} else {
			// This shouldn't happen after processing
			Error("Component %s has non-registry source after processing", comp.ID)
			continue
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
	if len(manifest.Variables) > 0 {
		req.Application.Variables = &manifest.Variables
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
				sp.Suffix = fmt.Sprintf(" Waiting for deployment to start... (%s, %s, %s)", app.LatestDeployment.DeploymentId, deploymentID, app.LatestDeployment.Status)
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

func displayDryRunSummary(manifest *ftltypes.Manifest, appExists bool) {
	fmt.Println()
	fmt.Println("🔍 DRY RUN MODE - No changes will be made")
	fmt.Println()

	color.Cyan("Application Configuration:")
	fmt.Printf("  Name: %s\n", manifest.Application.Name)
	fmt.Printf("  Version: %s\n", manifest.Application.Version)
	if manifest.Application.Description != "" {
		fmt.Printf("  Description: %s\n", manifest.Application.Description)
	}
	fmt.Printf("  Access Control: %s\n", manifest.Access)

	if manifest.Auth != nil {
		if manifest.Auth.JWTIssuer != "" {
			fmt.Printf("  Auth Provider: Custom\n")
			fmt.Printf("  JWT Issuer: %s\n", manifest.Auth.JWTIssuer)
			if manifest.Auth.JWTAudience != "" {
				fmt.Printf("  JWT Audience: %s\n", manifest.Auth.JWTAudience)
			}
		}
	}

	fmt.Println()
	color.Cyan("Components:")
	for _, comp := range manifest.Components {
		fmt.Printf("  • %s\n", comp.ID)

		// Show source type
		if localPath, registrySource := ftltypes.ParseComponentSource(comp.Source); localPath != "" {
			fmt.Printf("    Source: %s (local)\n", localPath)
			if comp.Build != nil && comp.Build.Command != "" {
				fmt.Printf("    Build: %s\n", comp.Build.Command)
			}
		} else if registrySource != nil {
			fmt.Printf("    Source: %s (registry)\n", registrySource.Registry)
			if registrySource.Package != "" {
				fmt.Printf("    Package: %s\n", registrySource.Package)
			}
			if registrySource.Version != "" {
				fmt.Printf("    Version: %s\n", registrySource.Version)
			}
		}
	}

	fmt.Println()
	color.Cyan("Actions that would be performed:")

	fmt.Printf("  ✓ Synthesize spin.toml from configuration\n")
	fmt.Printf("  ✓ Build local components with 'spin build'\n")

	if appExists {
		fmt.Printf("  ✓ Update existing app\n")
	} else {
		fmt.Printf("  ✓ Create new app\n")
	}

	fmt.Printf("  ✓ Pull registry components and push all to ECR\n")
	fmt.Printf("  ✓ Create deployment with processed manifest\n")
	fmt.Printf("  ✓ Platform will deploy from ECR\n")

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

// displayMCPUrls displays a table showing MCP URLs for the application and its components
func displayMCPUrls(baseURL string, components []ftltypes.Component) {
	// Ensure the base URL ends with /mcp
	mcpBaseURL := strings.TrimRight(baseURL, "/") + "/mcp"

	// Create data writer for table output
	dw := NewDataWriter(colorOutput, "table")

	// Build table with headers
	tb := NewTableBuilder("COMPONENT", "URL")

	// Add the main application MCP URL
	tb.AddRow("*all", mcpBaseURL)

	// Add component-specific MCP URLs
	for _, comp := range components {
		componentURL := fmt.Sprintf("%s/x/%s", mcpBaseURL, comp.ID)
		tb.AddRow(comp.ID, componentURL)
	}

	// Write the table (with empty line before it)
	fmt.Println()
	if err := tb.Write(dw); err != nil {
		// Fallback to simple display if table fails
		fmt.Printf("URL: %s\n", mcpBaseURL)
		for _, comp := range components {
			fmt.Printf("%s: %s/x/%s\n", comp.ID, mcpBaseURL, comp.ID)
		}
	}

	// Add summary line after table
	_, _ = fmt.Fprintf(colorOutput, "Connect to MCP clients with the URLs above.\n")
}
