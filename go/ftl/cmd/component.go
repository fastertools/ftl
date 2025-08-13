package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

func newComponentCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "component",
		Short: "Manage FTL components",
		Long:  `Manage FTL components including adding, removing, and listing components.`,
	}
	
	// Add subcommands
	cmd.AddCommand(
		newComponentAddCmd(),
		newComponentListCmd(),
		newComponentRemoveCmd(),
	)
	
	return cmd
}

func newComponentAddCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "add [name]",
		Short: "Add a new component",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			name := args[0]
			fmt.Printf("Adding component: %s\n", name)
			// TODO: Implement component add logic
			fmt.Println("Component add not yet implemented")
			return nil
		},
	}
}

func newComponentListCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "list",
		Short: "List all components",
		RunE: func(cmd *cobra.Command, args []string) error {
			fmt.Println("Listing components...")
			// TODO: Implement component list logic
			fmt.Println("Component list not yet implemented")
			return nil
		},
	}
}

func newComponentRemoveCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "remove [name]",
		Short: "Remove a component",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			name := args[0]
			fmt.Printf("Removing component: %s\n", name)
			// TODO: Implement component remove logic
			fmt.Println("Component remove not yet implemented")
			return nil
		},
	}
}