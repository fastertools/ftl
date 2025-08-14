package cmd

import (
	"context"
	"fmt"
	"os"

	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"

	"github.com/fastertools/ftl-cli/go/shared/config"
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
1. Reads your spindl.yml configuration
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
	cmd.Flags().StringVarP(&configFile, "file", "f", "spindl.yml", "FTL configuration file")
	cmd.Flags().BoolVar(&dryRun, "dry-run", false, "Validate configuration without deploying")

	return cmd
}

func runDeploy(ctx context.Context, configFile, environment string, dryRun bool) error {
	// Use the full deploy implementation
	args := &DeployArgs{
		Environment: environment,
		ConfigFile:  configFile,
		DryRun:      dryRun,
	}

	return runFullDeploy(ctx, args)
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
