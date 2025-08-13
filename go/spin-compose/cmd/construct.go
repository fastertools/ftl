package cmd

import (
	"fmt"

	"github.com/spf13/cobra"

	"github.com/fastertools/ftl-cli/go/spin-compose/pkg/construct"
)

var constructCmd = &cobra.Command{
	Use:   "construct",
	Short: "Manage constructs",
	Long: `Manage high-level constructs for building applications.

Constructs are pre-built patterns that encapsulate best practices and common
architectures. The flagship construct is the MCP (Model Context Protocol)
application construct.`,
}

var constructListCmd = &cobra.Command{
	Use:   "list",
	Short: "List available constructs",
	Long:  `List all available constructs that can be used in your projects.`,
	RunE: func(cmd *cobra.Command, args []string) error {
		return runConstructList()
	},
}

var constructAddCmd = &cobra.Command{
	Use:   "add [construct-name]",
	Short: "Add a construct to your project",
	Long: `Add a construct to your current project configuration.

This command will modify your spinc.yaml file to include the specified construct
with sensible defaults that you can then customize.`,
	Args: cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		constructName := args[0]
		return runConstructAdd(constructName)
	},
}

var constructShowCmd = &cobra.Command{
	Use:   "show [construct-name]",
	Short: "Show construct details",
	Long:  `Show detailed information about a specific construct including its parameters and usage.`,
	Args:  cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		constructName := args[0]
		return runConstructShow(constructName)
	},
}

func init() {
	constructCmd.AddCommand(constructListCmd)
	constructCmd.AddCommand(constructAddCmd)
	constructCmd.AddCommand(constructShowCmd)
}

func runConstructList() error {
	printInfo("Available constructs:")
	fmt.Println()
	
	constructs := construct.GetAvailableConstructs()
	
	for _, c := range constructs {
		status := ""
		if c.Status == "stable" {
			status = successColor.Sprint("✓ stable")
		} else if c.Status == "preview" {
			status = warningColor.Sprint("⚠ preview")
		} else {
			status = dimColor.Sprint("○ planned")
		}
		
		fmt.Printf("  %s %s - %s\n", 
			infoColor.Sprint(c.Name),
			status,
			c.Description,
		)
		
		if c.Status == "stable" || c.Status == "preview" {
			fmt.Printf("    %s\n", dimColor.Sprintf("Usage: spin-compose init my-app --template %s", c.Name))
		}
		fmt.Println()
	}
	
	printInfo("Learn more:")
	fmt.Printf("  %s\n", dimColor.Sprint("Run 'spin-compose construct show <name>' for detailed information"))
	
	return nil
}

func runConstructAdd(constructName string) error {
	printInfo("Adding construct '%s' to project", constructName)
	
	// Get construct definition
	constructDef := construct.GetConstruct(constructName)
	if constructDef == nil {
		printError("Unknown construct '%s'", constructName)
		printInfo("Run 'spin-compose construct list' to see available constructs")
		return fmt.Errorf("unknown construct: %s", constructName)
	}
	
	if constructDef.Status != "stable" && constructDef.Status != "preview" {
		printError("Construct '%s' is not yet available (status: %s)", constructName, constructDef.Status)
		return fmt.Errorf("construct not available: %s", constructName)
	}
	
	// For now, just provide guidance
	printWarning("Construct addition is not yet implemented")
	printInfo("To use this construct:")
	fmt.Printf("  1. %s\n", infoColor.Sprint("Create a new project: spin-compose init my-app --template "+constructName))
	fmt.Printf("  2. %s\n", infoColor.Sprint("Or manually configure your spinc.yaml based on the template"))
	
	return nil
}

func runConstructShow(constructName string) error {
	constructDef := construct.GetConstruct(constructName)
	if constructDef == nil {
		printError("Unknown construct '%s'", constructName)
		printInfo("Run 'spin-compose construct list' to see available constructs")
		return fmt.Errorf("unknown construct: %s", constructName)
	}
	
	fmt.Printf("%s %s\n\n", infoColor.Sprint("Construct:"), constructDef.Name)
	
	status := ""
	switch constructDef.Status {
	case "stable":
		status = successColor.Sprint("✓ Stable")
	case "preview":
		status = warningColor.Sprint("⚠ Preview")
	default:
		status = dimColor.Sprint("○ Planned")
	}
	fmt.Printf("%s %s\n", infoColor.Sprint("Status:"), status)
	fmt.Printf("%s %s\n\n", infoColor.Sprint("Description:"), constructDef.Description)
	
	if len(constructDef.Features) > 0 {
		fmt.Printf("%s\n", infoColor.Sprint("Features:"))
		for _, feature := range constructDef.Features {
			fmt.Printf("  • %s\n", feature)
		}
		fmt.Println()
	}
	
	if len(constructDef.Examples) > 0 {
		fmt.Printf("%s\n", infoColor.Sprint("Examples:"))
		for _, example := range constructDef.Examples {
			fmt.Printf("  %s\n", dimColor.Sprint(example))
		}
		fmt.Println()
	}
	
	if constructDef.Status == "stable" || constructDef.Status == "preview" {
		fmt.Printf("%s\n", infoColor.Sprint("Usage:"))
		fmt.Printf("  %s\n", "spin-compose init my-app --template "+constructName)
		fmt.Println()
	}
	
	return nil
}