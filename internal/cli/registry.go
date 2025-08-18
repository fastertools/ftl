package cli

import (
	"context"
	"fmt"

	"github.com/fastertools/ftl-cli/pkg/spin"
	"github.com/spf13/cobra"
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
		Long: `Pull a Spin application from a registry.

Example:
  ftl registry pull ghcr.io/myorg/myapp:v1.0.0`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := context.Background()
			reference := args[0]

			fmt.Printf("Pulling from registry: %s\n", reference)

			// Use spin registry pull
			if err := spin.Registry(ctx, "pull", reference); err != nil {
				return fmt.Errorf("failed to pull from registry: %w", err)
			}

			fmt.Println("Pull completed successfully")
			return nil
		},
	}
}

func newRegistryListCmd() *cobra.Command {
	var registry string

	cmd := &cobra.Command{
		Use:   "list",
		Short: "List applications in registry",
		Long: `List available Spin applications in a registry.

Example:
  ftl registry list --registry ghcr.io/myorg`,
		RunE: func(cmd *cobra.Command, args []string) error {
			if registry == "" {
				return fmt.Errorf("--registry flag is required")
			}

			ctx := context.Background()
			fmt.Printf("Listing applications in registry: %s\n", registry)

			// Use spin registry list (if available) or catalog
			if err := spin.Registry(ctx, "catalog", "list", registry); err != nil {
				return fmt.Errorf("failed to list registry contents: %w", err)
			}

			return nil
		},
	}

	cmd.Flags().StringVarP(&registry, "registry", "r", "", "Registry URL to list")

	return cmd
}
