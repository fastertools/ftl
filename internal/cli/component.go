package cli

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/AlecAivazis/survey/v2"
	"github.com/fatih/color"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"

	"github.com/fastertools/ftl-cli/pkg/types"
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
	// Load manifest
	manifest, err := loadComponentManifest("ftl.yaml")
	if err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("ftl.yaml not found. Run 'ftl init' first")
		}
		return err
	}

	if len(manifest.Components) == 0 {
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
	for _, comp := range manifest.Components {
		fmt.Printf("  • %s\n", comp.ID)

		// Show source info
		if localPath, registrySource := types.ParseComponentSource(comp.Source); localPath != "" {
			fmt.Printf("    Source: %s\n", localPath)
		} else if registrySource != nil {
			fmt.Printf("    Source: %s/%s:%s\n",
				registrySource.Registry,
				registrySource.Package,
				registrySource.Version)
		}

		// Show build info if present
		if comp.Build != nil && comp.Build.Command != "" {
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
	// Load manifest
	manifest, err := loadComponentManifest("ftl.yaml")
	if err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("ftl.yaml not found")
		}
		return err
	}

	if len(manifest.Components) == 0 {
		return fmt.Errorf("no components to remove")
	}

	// If no name provided, show interactive selection
	if name == "" {
		var options []string
		for _, comp := range manifest.Components {
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

	// Find and remove component
	found := false
	newComponents := []types.Component{}
	for _, comp := range manifest.Components {
		if comp.ID == name {
			found = true
			continue
		}
		newComponents = append(newComponents, comp)
	}

	if !found {
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

	// Update manifest
	manifest.Components = newComponents

	// Save manifest
	if err := saveComponentManifest("ftl.yaml", manifest); err != nil {
		return fmt.Errorf("failed to save manifest: %w", err)
	}

	color.Green("✓ Component '%s' removed", name)
	return nil
}

func loadComponentManifest(path string) (*types.Manifest, error) {
	// Clean the path to prevent directory traversal
	path = filepath.Clean(path)
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	var manifest types.Manifest
	if err := yaml.Unmarshal(data, &manifest); err != nil {
		return nil, fmt.Errorf("failed to parse manifest: %w", err)
	}

	return &manifest, nil
}

func saveComponentManifest(path string, manifest *types.Manifest) error {
	data, err := yaml.Marshal(manifest)
	if err != nil {
		return fmt.Errorf("failed to marshal manifest: %w", err)
	}
	return os.WriteFile(path, data, 0600)
}
