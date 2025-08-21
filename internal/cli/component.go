package cli

import (
	"fmt"

	"github.com/AlecAivazis/survey/v2"
	"github.com/fastertools/ftl/internal/manifest"
	"github.com/fatih/color"
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

func newComponentListCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "list",
		Short: "List all components",
		RunE: func(cmd *cobra.Command, args []string) error {
			return listComponents()
		},
	}
}

func listComponents() error {
	// Load manifest (tries ftl.yaml, ftl.yml, ftl.json)
	m, err := manifest.LoadAuto()
	if err != nil {
		return fmt.Errorf("failed to load manifest: %w", err)
	}

	if len(m.Components) == 0 {
		fmt.Println("No components found.")
		fmt.Println()
		fmt.Println("Add a component with:")
		color.Cyan("  ftl component add")
		return nil
	}

	// Print header
	color.Cyan("Components:")
	fmt.Println()

	// List components
	for _, comp := range m.Components {
		fmt.Printf("  • %s\n", comp.ID)

		// Show source info
		switch src := comp.Source.(type) {
		case string:
			fmt.Printf("    Source: %s\n", src)
		case manifest.SourceRegistry:
			fmt.Printf("    Source: %s/%s:%s\n",
				src.Registry, src.Package, src.Version)
		case map[string]interface{}:
			// Handle case where source is still a map (shouldn't happen with proper unmarshaling)
			if registry, ok := src["registry"].(string); ok {
				fmt.Printf("    Source: %s/%s:%s\n",
					registry, src["package"], src["version"])
			}
		case map[interface{}]interface{}:
			// Handle case where source is still a map (shouldn't happen with proper unmarshaling)
			if registry, ok := src["registry"].(string); ok {
				fmt.Printf("    Source: %s/%s:%s\n",
					registry, src["package"], src["version"])
			}
		default:
			fmt.Printf("    Source: %v (type: %T)\n", src, src)
		}

		// Show build info if present
		if comp.Build != nil {
			fmt.Printf("    Build: %s\n", comp.Build.Command)
		}

		// Show variables if present
		if len(comp.Variables) > 0 {
			fmt.Printf("    Variables: %d configured\n", len(comp.Variables))
		}

		fmt.Println()
	}

	return nil
}

func newComponentRemoveCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "remove [name]",
		Short: "Remove a component",
		Args:  cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var name string
			if len(args) > 0 {
				name = args[0]
			}
			return removeComponent(name)
		},
	}
}

func removeComponent(name string) error {
	// Load manifest (tries ftl.yaml, ftl.yml, ftl.json)
	m, err := manifest.LoadAuto()
	if err != nil {
		return fmt.Errorf("failed to load manifest: %w", err)
	}

	if len(m.Components) == 0 {
		return fmt.Errorf("no components to remove")
	}

	// If no name provided, show interactive selection
	if name == "" {
		var options []string
		for _, comp := range m.Components {
			options = append(options, comp.ID)
		}

		prompt := &survey.Select{
			Message: "Select component to remove:",
			Options: options,
		}
		if err := survey.AskOne(prompt, &name); err != nil {
			return err
		}
	}

	// Check if component exists
	if _, idx := m.FindComponent(name); idx == -1 {
		return fmt.Errorf("component '%s' not found", name)
	}

	// Confirm removal
	confirm := false
	prompt := &survey.Confirm{
		Message: fmt.Sprintf("Remove component '%s'?", name),
		Default: false,
	}
	if err := survey.AskOne(prompt, &confirm); err != nil {
		return err
	}

	if !confirm {
		fmt.Println("Cancelled")
		return nil
	}

	// Remove component
	if err := m.RemoveComponent(name); err != nil {
		return err
	}

	// Save manifest (to the same file format)
	if err := m.SaveAuto(); err != nil {
		return fmt.Errorf("failed to save manifest: %w", err)
	}

	color.Green("✓ Component '%s' removed", name)
	return nil
}

// Removed - now using manifest package
