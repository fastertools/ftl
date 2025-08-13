package cmd

import (
	"context"
	"fmt"

	"github.com/spf13/cobra"
	"github.com/fastertools/ftl-cli/go/shared/spin"
)

func newDeployCmd() *cobra.Command {
	var environment string
	
	cmd := &cobra.Command{
		Use:   "deploy",
		Short: "Deploy the FTL application",
		Long:  `Deploy the FTL application to the specified environment.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := context.Background()
			
			// Ensure spin is installed
			if err := spin.EnsureInstalled(); err != nil {
				return err
			}
			
			fmt.Printf("Deploying FTL application to %s...\n", environment)
			
			// Use spin deploy
			if err := spin.Deploy(ctx); err != nil {
				return fmt.Errorf("failed to deploy: %w", err)
			}
			
			fmt.Println("Deployment completed successfully")
			return nil
		},
	}
	
	cmd.Flags().StringVarP(&environment, "environment", "e", "production", "Deployment environment")
	
	return cmd
}