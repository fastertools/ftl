package cmd

import (
	"context"
	"fmt"

	"github.com/spf13/cobra"
	"github.com/fastertools/ftl-cli/go/shared/spin"
)

func newBuildCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "build",
		Short: "Build the FTL application",
		Long:  `Build compiles the FTL application and its components.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := context.Background()
			
			// Ensure spin is installed
			if err := spin.EnsureInstalled(); err != nil {
				return err
			}
			
			fmt.Println("Building FTL application...")
			
			// Use spin build
			if err := spin.Build(ctx); err != nil {
				return fmt.Errorf("failed to build: %w", err)
			}
			
			fmt.Println("Build completed successfully")
			return nil
		},
	}
	
	return cmd
}