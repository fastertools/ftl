package cmd

import (
	"fmt"
	"os"
	"strings"

	"github.com/fatih/color"
	"github.com/spf13/cobra"

	"github.com/fastertools/ftl-cli/go/spin-compose/internal/synth"
)

var diffCmd = &cobra.Command{
	Use:   "diff [input-file] [current-manifest]",
	Short: "Show what would change in the manifest",
	Long: `Show the differences between the current manifest and what would be generated.

The diff command synthesizes a new manifest from your configuration and compares
it with the existing manifest, highlighting any changes that would be made.`,
	Args: cobra.MaximumNArgs(2),
	RunE: func(cmd *cobra.Command, args []string) error {
		inputFile := "spinc.yaml"
		currentManifest := "spin.toml"
		
		if len(args) > 0 {
			inputFile = args[0]
		}
		if len(args) > 1 {
			currentManifest = args[1]
		}
		
		return runDiff(inputFile, currentManifest)
	},
}

func runDiff(inputFile, currentManifest string) error {
	printInfo("Computing diff for %s", inputFile)
	
	// Read and synthesize new configuration
	configData, err := os.ReadFile(inputFile)
	if err != nil {
		printError("Failed to read %s: %v", inputFile, err)
		return err
	}
	
	format := determineFormat(inputFile)
	engine := synth.NewEngine()
	
	newManifestData, err := engine.SynthesizeConfig(configData, format)
	if err != nil {
		printError("Synthesis failed: %v", err)
		return err
	}
	
	// Read current manifest (if it exists)
	var currentManifestData []byte
	if _, err := os.Stat(currentManifest); err == nil {
		currentManifestData, err = os.ReadFile(currentManifest)
		if err != nil {
			printError("Failed to read current manifest %s: %v", currentManifest, err)
			return err
		}
	} else {
		// If current manifest doesn't exist, treat as empty
		currentManifestData = []byte("")
	}
	
	// Compare and show diff
	if err := showDiff(string(currentManifestData), string(newManifestData), currentManifest); err != nil {
		return err
	}
	
	return nil
}

// showDiff displays a colored diff between two strings
func showDiff(current, new, filename string) error {
	currentLines := strings.Split(strings.TrimSpace(current), "\n")
	newLines := strings.Split(strings.TrimSpace(new), "\n")
	
	// If current is empty, treat it as no lines
	if len(currentLines) == 1 && currentLines[0] == "" {
		currentLines = []string{}
	}
	
	// Simple line-by-line diff
	if len(currentLines) == 0 && len(newLines) == 0 {
		printSuccess("No changes detected")
		return nil
	}
	
	if strings.Join(currentLines, "\n") == strings.Join(newLines, "\n") {
		printSuccess("No changes detected")
		return nil
	}
	
	fmt.Printf("%s Changes detected in %s:\n\n", warningColor.Sprint("⚠"), filename)
	
	// Create a simple diff
	maxLines := max(len(currentLines), len(newLines))
	
	for i := 0; i < maxLines; i++ {
		var currentLine, newLine string
		
		if i < len(currentLines) {
			currentLine = currentLines[i]
		}
		if i < len(newLines) {
			newLine = newLines[i]
		}
		
		if currentLine != newLine {
			if currentLine != "" {
				fmt.Printf("%s %s\n", color.RedString("-"), color.RedString(currentLine))
			}
			if newLine != "" {
				fmt.Printf("%s %s\n", color.GreenString("+"), color.GreenString(newLine))
			}
		}
	}
	
	fmt.Println()
	printInfo("Run 'spin-compose synth' to apply these changes")
	
	return nil
}

// max returns the maximum of two integers
func max(a, b int) int {
	if a > b {
		return a
	}
	return b
}