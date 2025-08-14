package cmd

import (
	"context"
	"fmt"
	"os"

	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"

	"github.com/fastertools/ftl-cli/go/shared/config"
	"github.com/fastertools/ftl-cli/go/shared/spin"
)

func newDeployCmd() *cobra.Command {
	var (
		environment string
		configFile  string
		dryRun      bool
	)

	cmd := &cobra.Command{
		Use:   "deploy [flags]",
		Short: "Deploy the FTL application to the platform",
		Long: `Deploy the FTL application to the platform.

This command:
1. Reads your ftl.yaml configuration
2. Validates the configuration
3. Pushes components to the registry
4. Deploys the application

Example:
  ftl deploy
  ftl deploy -e staging
  ftl deploy -f custom-config.yaml`,
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := context.Background()
			return runDeploy(ctx, configFile, environment, dryRun)
		},
	}

	cmd.Flags().StringVarP(&environment, "environment", "e", "production", "Deployment environment (production, staging, development)")
	cmd.Flags().StringVarP(&configFile, "file", "f", "ftl.yaml", "FTL configuration file")
	cmd.Flags().BoolVar(&dryRun, "dry-run", false, "Validate configuration without deploying")

	return cmd
}

func runDeploy(ctx context.Context, configFile, environment string, dryRun bool) error {
	// Load configuration
	cfg, err := loadConfig(configFile)
	if err != nil {
		return err
	}

	if dryRun {
		fmt.Println("🔍 Dry run mode - no changes will be made")
		fmt.Printf("Would deploy application: %s\n", cfg.Application.Name)
		fmt.Printf("Environment: %s\n", environment)
		return nil
	}

	// Run spin deploy
	fmt.Printf("🚀 Deploying %s to %s environment...\n", cfg.Application.Name, environment)
	
	deployArgs := []string{"deploy"}
	if environment != "" {
		deployArgs = append(deployArgs, "--environment-name", environment)
	}

	if err := spin.Deploy(ctx, deployArgs...); err != nil {
		return fmt.Errorf("deployment failed: %w", err)
	}

	fmt.Println("✅ Deployment successful!")
	return nil
}

// loadConfig loads and parses the FTL configuration file
func loadConfig(configFile string) (*config.FTLConfig, error) {
	data, err := os.ReadFile(configFile)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, fmt.Errorf("config file not found: %s", configFile)
		}
		return nil, err
	}

	var cfg config.FTLConfig
	if err := yaml.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse YAML: %w", err)
	}

	// Set defaults
	cfg.SetDefaults()

	return &cfg, nil
}
