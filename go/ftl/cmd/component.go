package cmd

import (
	"fmt"
	"os"

	"github.com/fatih/color"
	"github.com/spf13/cobra"

	"github.com/fastertools/ftl-cli/go/shared/config"
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
	// Load config
	cfg, err := loadSpinConfig("ftl.yaml")
	if err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("ftl.yaml not found. Run 'ftl init' first")
		}
		return err
	}

	if len(cfg.Components) == 0 {
		fmt.Println("No components found.")
		fmt.Println("\nAdd a component with: ftl component add <type> ...")
		return nil
	}

	// Display components using shared DataWriter
	dw := NewDataWriter(colorOutput, "table")
	tb := NewTableBuilder("ID", "SOURCE", "ROUTE", "DESCRIPTION")

	for _, comp := range cfg.Components {
		// Format source
		var source string
		switch src := comp.Source.(type) {
		case string:
			source = src
		case map[string]interface{}:
			if url, ok := src["url"]; ok {
				source = fmt.Sprintf("%s (URL)", url)
			} else if registry, ok := src["registry"]; ok {
				pkg := src["package"]
				version := src["version"]
				source = fmt.Sprintf("%s/%s:%s", registry, pkg, version)
			}
		}

		// Find route from triggers
		route := "-"
		for _, trigger := range cfg.Triggers {
			if trigger.Component == comp.ID {
				if trigger.Type == config.TriggerTypeHTTP && trigger.Route != "" {
					route = trigger.Route
				}
			}
		}

		// Format description (truncate if too long)
		description := comp.Description
		if len(description) > 40 {
			description = description[:37] + "..."
		}
		if description == "" {
			description = "-"
		}

		tb.AddRow(comp.ID, source, route, description)
	}

	fmt.Printf("\nComponents in %s:\n", cfg.Application.Name)
	return tb.Write(dw)
}

func newComponentRemoveCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "remove <name>",
		Short: "Remove a component",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			name := args[0]
			return removeComponent(name)
		},
	}
}

func removeComponent(name string) error {
	// Color helpers
	green := color.New(color.FgGreen).SprintFunc()

	// Load config
	cfg, err := loadSpinConfig("ftl.yaml")
	if err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("ftl.yaml not found. Run 'ftl init' first")
		}
		return err
	}

	// Find and remove component
	found := false
	newComponents := []config.ComponentConfig{}
	for _, comp := range cfg.Components {
		if comp.ID != name {
			newComponents = append(newComponents, comp)
		} else {
			found = true
		}
	}

	if !found {
		return fmt.Errorf("component '%s' not found", name)
	}

	cfg.Components = newComponents

	// Also remove associated triggers
	newTriggers := []config.TriggerConfig{}
	for _, trigger := range cfg.Triggers {
		if trigger.Component != name {
			newTriggers = append(newTriggers, trigger)
		}
	}
	cfg.Triggers = newTriggers

	// Save updated config
	if err := saveSpinConfig("ftl.yaml", cfg); err != nil {
		return fmt.Errorf("failed to save config: %w", err)
	}

	fmt.Printf("%s Component '%s' removed\n", green("✓"), name)
	return nil
}
