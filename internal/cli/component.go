package cli

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/AlecAivazis/survey/v2"
	"github.com/fatih/color"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"
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

	components, ok := manifest["components"].([]interface{})
	if !ok || len(components) == 0 {
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
	for _, c := range components {
		comp := c.(map[interface{}]interface{})
		fmt.Printf("  • %s\n", comp["id"])

		// Show source info
		switch src := comp["source"].(type) {
		case string:
			fmt.Printf("    Source: %s\n", src)
		case map[interface{}]interface{}:
			if registry, ok := src["registry"]; ok {
				fmt.Printf("    Source: %s/%s:%s\n",
					registry, src["package"], src["version"])
			}
		}

		// Show build info if present
		if build, ok := comp["build"].(map[interface{}]interface{}); ok {
			if cmd, ok := build["command"]; ok {
				fmt.Printf("    Build: %s\n", cmd)
			}
		}

		// Show variables if present
		if vars, ok := comp["variables"].(map[interface{}]interface{}); ok && len(vars) > 0 {
			fmt.Printf("    Variables: %d configured\n", len(vars))
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

	components, ok := manifest["components"].([]interface{})
	if !ok || len(components) == 0 {
		return fmt.Errorf("no components to remove")
	}

	// If no name provided, show interactive selection
	if name == "" {
		var options []string
		for _, c := range components {
			comp := c.(map[interface{}]interface{})
			options = append(options, comp["id"].(string))
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
	newComponents := []interface{}{}
	for _, c := range components {
		comp := c.(map[interface{}]interface{})
		if comp["id"] == name {
			found = true
			continue
		}
		newComponents = append(newComponents, c)
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
	manifest["components"] = newComponents

	// Save manifest
	if err := saveComponentManifest("ftl.yaml", manifest); err != nil {
		return fmt.Errorf("failed to save manifest: %w", err)
	}

	color.Green("✓ Component '%s' removed", name)
	return nil
}

func loadComponentManifest(path string) (map[interface{}]interface{}, error) {
	// Clean the path to prevent directory traversal
	path = filepath.Clean(path)
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	var manifest map[interface{}]interface{}
	if err := yaml.Unmarshal(data, &manifest); err != nil {
		return nil, fmt.Errorf("failed to parse manifest: %w", err)
	}

	return manifest, nil
}

func saveComponentManifest(path string, manifest map[interface{}]interface{}) error {
	data, err := yaml.Marshal(manifest)
	if err != nil {
		return fmt.Errorf("failed to marshal manifest: %w", err)
	}
	return os.WriteFile(path, data, 0600)
}
