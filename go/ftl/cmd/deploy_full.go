package cmd

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/fatih/color"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"

	"github.com/fastertools/ftl-cli/go/shared/auth"
	"github.com/fastertools/ftl-cli/go/shared/config"
	"github.com/fastertools/ftl-cli/go/shared/spin"
	spindl "github.com/fastertools/ftl-cli/go/spindl/pkg/spindl"
)

// DeployArgs contains all deployment arguments
type DeployArgs struct {
	Environment   string
	ConfigFile    string
	DryRun        bool
	Variables     []string
	AccessControl string
	JWTIssuer     string
	JWTAudience   string
	AuthProvider  string
	Build         bool
	NoBuild       bool
}

// ComponentInfo contains component deployment information
type ComponentInfo struct {
	ID              string
	Name            string
	Source          string
	Version         string
	DeployName      string
	IsRegistry      bool
	IsUserComponent bool
}

// newDeployFullCmd creates the full deploy command
func newDeployFullCmd() *cobra.Command {
	args := &DeployArgs{}

	cmd := &cobra.Command{
		Use:   "deploy-full [flags]",
		Short: "Deploy the FTL application to the platform (full implementation)",
		Long: `Deploy the FTL application to the FTL platform.

This command:
1. Builds your application (unless --no-build is specified)
2. Authenticates with the FTL platform
3. Pushes user components to the registry
4. Creates or updates the deployment
5. Configures access control and authentication

Examples:
  # Deploy with default settings
  ftl deploy-full

  # Deploy to staging environment
  ftl deploy-full -e staging

  # Deploy with public access
  ftl deploy-full --access-control public

  # Deploy with custom authentication
  ftl deploy-full --access-control custom --jwt-issuer https://auth.example.com

  # Deploy with variables
  ftl deploy-full --var API_KEY=secret --var LOG_LEVEL=debug

  # Dry run to validate without deploying
  ftl deploy-full --dry-run`,
		RunE: func(cmd *cobra.Command, _ []string) error {
			ctx := context.Background()
			return runFullDeploy(ctx, args)
		},
	}

	// Add flags
	cmd.Flags().StringVarP(&args.Environment, "environment", "e", "production", "Deployment environment")
	cmd.Flags().StringVarP(&args.ConfigFile, "file", "f", "spindl.yml", "Configuration file")
	cmd.Flags().BoolVar(&args.DryRun, "dry-run", false, "Validate without deploying")
	cmd.Flags().StringArrayVar(&args.Variables, "var", nil, "Set variables (KEY=value)")
	cmd.Flags().StringVar(&args.AccessControl, "access-control", "", "Access control mode (public, private, custom)")
	cmd.Flags().StringVar(&args.JWTIssuer, "jwt-issuer", "", "JWT issuer URL for custom auth")
	cmd.Flags().StringVar(&args.JWTAudience, "jwt-audience", "", "JWT audience for custom auth")
	cmd.Flags().StringVar(&args.AuthProvider, "auth-provider", "", "Authentication provider type")
	cmd.Flags().BoolVar(&args.Build, "build", false, "Force build before deploy")
	cmd.Flags().BoolVar(&args.NoBuild, "no-build", false, "Skip building")

	return cmd
}

func runFullDeploy(ctx context.Context, args *DeployArgs) error {
	// Color helpers
	blue := color.New(color.FgBlue).SprintFunc()
	green := color.New(color.FgGreen).SprintFunc()
	yellow := color.New(color.FgYellow).SprintFunc()
	_ = color.New(color.FgRed).SprintFunc() // red - for errors

	// Header
	if args.DryRun {
		fmt.Printf("%s Deploying to FTL Platform (DRY RUN)\n", blue("▶"))
	} else {
		fmt.Printf("%s Deploying to FTL Platform\n", blue("▶"))
	}
	fmt.Println()

	// Step 1: Check dependencies
	fmt.Printf("%s Checking dependencies...\n", blue("→"))
	if err := spin.EnsureInstalled(); err != nil {
		return fmt.Errorf("spin not installed: %w", err)
	}
	fmt.Printf("%s Dependencies verified\n", green("✓"))

	// Step 2: Load and validate configuration
	fmt.Printf("\n%s Loading configuration...\n", blue("→"))
	ftlConfig, err := loadConfig(args.ConfigFile)
	if err != nil {
		return fmt.Errorf("failed to load config: %w", err)
	}

	if err := ftlConfig.Validate(); err != nil {
		return fmt.Errorf("invalid configuration: %w", err)
	}

	fmt.Printf("%s Application: %s (v%s)\n", green("✓"), ftlConfig.Application.Name, ftlConfig.Application.Version)
	fmt.Printf("  Components: %d\n", len(ftlConfig.Components))

	// Step 3: Synthesize spin.toml
	fmt.Printf("\n%s Synthesizing manifest...\n", blue("→"))
	engine := spindl.NewEngine()

	configData, err := yaml.Marshal(ftlConfig)
	if err != nil {
		return fmt.Errorf("failed to marshal config: %w", err)
	}

	spinManifest, err := engine.SynthesizeConfig(configData, "yaml")
	if err != nil {
		return fmt.Errorf("failed to synthesize manifest: %w", err)
	}

	// Write temporary spin.toml
	tempDir, err := os.MkdirTemp("", "ftl-deploy-")
	if err != nil {
		return fmt.Errorf("failed to create temp dir: %w", err)
	}
	defer os.RemoveAll(tempDir)

	spinTomlPath := filepath.Join(tempDir, "spin.toml")
	if err := os.WriteFile(spinTomlPath, spinManifest, 0644); err != nil {
		return fmt.Errorf("failed to write spin.toml: %w", err)
	}
	fmt.Printf("%s Manifest synthesized\n", green("✓"))

	// Step 4: Build if needed
	if !args.NoBuild {
		fmt.Printf("\n%s Building application...\n", blue("→"))
		buildCmd := exec.CommandContext(ctx, "spin", "build", "-f", spinTomlPath)
		buildCmd.Stdout = os.Stdout
		buildCmd.Stderr = os.Stderr
		if err := buildCmd.Run(); err != nil {
			return fmt.Errorf("build failed: %w", err)
		}
		fmt.Printf("%s Build completed\n", green("✓"))
	}

	// Step 5: Authenticate
	fmt.Printf("\n%s Authenticating...\n", blue("→"))
	store, err := auth.NewKeyringStore()
	if err != nil {
		return fmt.Errorf("failed to create auth store: %w", err)
	}
	if _, err := store.Load(); err != nil {
		return fmt.Errorf("not authenticated. Run 'ftl login' first: %w", err)
	}
	fmt.Printf("%s Authenticated\n", green("✓"))

	if args.DryRun {
		fmt.Printf("\n%s Dry run complete. Configuration is valid.\n", green("✓"))
		fmt.Printf("\nDeployment would include:\n")
		fmt.Printf("  • Application: %s\n", ftlConfig.Application.Name)
		fmt.Printf("  • Components: %d\n", len(ftlConfig.Components))
		fmt.Printf("  • Environment: %s\n", args.Environment)
		return nil
	}

	// Step 6: Identify user components
	fmt.Printf("\n%s Identifying components to deploy...\n", blue("→"))
	userComponents := identifyUserComponents(ftlConfig)
	fmt.Printf("  Found %d user components\n", len(userComponents))

	// For now, since the backend API is still being developed,
	// we'll use spin deploy directly
	fmt.Printf("\n%s Using spin deploy for deployment...\n", blue("→"))

	// Build spin deploy command
	deployCmd := exec.CommandContext(ctx, "spin", "deploy", "-f", spinTomlPath)

	// Add environment variables for auth config if specified
	if args.AccessControl == "public" {
		deployCmd.Env = append(os.Environ(), "SPIN_VARIABLE_auth_enabled=false")
	} else if args.AccessControl == "private" {
		deployCmd.Env = append(os.Environ(), "SPIN_VARIABLE_auth_enabled=true")
		if args.JWTIssuer != "" {
			deployCmd.Env = append(deployCmd.Env, fmt.Sprintf("SPIN_VARIABLE_mcp_jwt_issuer=%s", args.JWTIssuer))
		}
	}

	// Add custom variables
	for _, v := range args.Variables {
		parts := strings.SplitN(v, "=", 2)
		if len(parts) == 2 {
			deployCmd.Env = append(deployCmd.Env, fmt.Sprintf("SPIN_VARIABLE_%s=%s", parts[0], parts[1]))
		}
	}

	deployCmd.Stdout = os.Stdout
	deployCmd.Stderr = os.Stderr

	if err := deployCmd.Run(); err != nil {
		return fmt.Errorf("deployment failed: %w", err)
	}

	// Success!
	fmt.Printf("\n%s Deployment successful!\n", green("✓"))

	// Show access control info
	if args.AccessControl == "public" {
		fmt.Printf("\n%s Access: Public (no authentication required)\n", yellow("⚠"))
	} else {
		fmt.Printf("\n%s Access: Private (authentication required)\n", blue("ℹ"))
		if args.JWTIssuer != "" {
			fmt.Printf("  JWT Issuer: %s\n", args.JWTIssuer)
		}
	}

	return nil
}

// identifyUserComponents identifies which components are user components (not MCP infrastructure)
func identifyUserComponents(cfg *config.FTLConfig) []ComponentInfo {
	components := []ComponentInfo{}

	for _, comp := range cfg.Components {
		// Skip MCP infrastructure components
		if strings.HasPrefix(comp.ID, "mcp-") {
			continue
		}

		// Convert source to string representation
		var sourceStr string
		switch src := comp.Source.(type) {
		case string:
			sourceStr = src
		default:
			sourceStr = fmt.Sprintf("%v", src)
		}

		info := ComponentInfo{
			ID:              comp.ID,
			Name:            comp.ID,
			Source:          sourceStr,
			Version:         "latest",
			IsRegistry:      comp.IsRegistrySource(),
			IsUserComponent: true,
		}

		components = append(components, info)
	}

	return components
}

// These functions will be implemented when the backend API is ready
// For now, we use spin deploy directly
