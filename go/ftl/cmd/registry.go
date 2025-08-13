package cmd

import (
	"context"
	"fmt"

	"github.com/spf13/cobra"
	"github.com/fastertools/ftl-cli/go/shared/spin"
)

func newRegistryCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "registry",
		Short: "Manage registry operations",
		Long:  `Manage registry operations including push, pull, and list.`,
	}
	
	// Add subcommands
	cmd.AddCommand(
		newRegistryPushCmd(),
		newRegistryPullCmd(),
		newRegistryListCmd(),
	)
	
	return cmd
}

func newRegistryPushCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "push [reference]",
		Short: "Push application to registry",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := context.Background()
			reference := args[0]
			
			fmt.Printf("Pushing to registry: %s\n", reference)
			
			// Use spin registry push
			if err := spin.Registry(ctx, "push", reference); err != nil {
				return fmt.Errorf("failed to push to registry: %w", err)
			}
			
			fmt.Println("Push completed successfully")
			return nil
		},
	}
}

func newRegistryPullCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "pull [reference]",
		Short: "Pull application from registry",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			reference := args[0]
			fmt.Printf("Pulling from registry: %s\n", reference)
			// TODO: Implement registry pull logic
			fmt.Println("Registry pull not yet implemented")
			return nil
		},
	}
}

func newRegistryListCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "list",
		Short: "List registry contents",
		RunE: func(cmd *cobra.Command, args []string) error {
			fmt.Println("Listing registry contents...")
			// TODO: Implement registry list logic
			fmt.Println("Registry list not yet implemented")
			return nil
		},
	}
}