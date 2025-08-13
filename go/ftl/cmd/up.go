package cmd

import (
	"context"
	"fmt"

	"github.com/spf13/cobra"
	"github.com/fastertools/ftl-cli/go/shared/spin"
)

func newUpCmd() *cobra.Command {
	var build bool
	var watch bool
	
	cmd := &cobra.Command{
		Use:   "up",
		Short: "Run the FTL application locally",
		Long:  `Run the FTL application locally with hot reload support.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := context.Background()
			
			// Ensure spin is installed
			if err := spin.EnsureInstalled(); err != nil {
				return err
			}
			
			fmt.Println("Starting FTL application...")
			
			// Build if requested
			if build {
				fmt.Println("Building application first...")
				if err := spin.Build(ctx); err != nil {
					return fmt.Errorf("failed to build: %w", err)
				}
			}
			
			// Run with watch if requested
			if watch {
				fmt.Println("Starting with watch mode...")
				if err := spin.Watch(ctx); err != nil {
					return fmt.Errorf("failed to start with watch: %w", err)
				}
			} else {
				// Run normally
				if err := spin.Up(ctx); err != nil {
					return fmt.Errorf("failed to start: %w", err)
				}
			}
			
			return nil
		},
	}
	
	cmd.Flags().BoolVarP(&build, "build", "b", false, "Build before running")
	cmd.Flags().BoolVarP(&watch, "watch", "w", false, "Watch for changes and reload")
	
	return cmd
}