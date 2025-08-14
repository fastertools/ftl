package cmd

import (
	"context"
	"fmt"
	"os"

	"github.com/fastertools/ftl-cli/go/shared/config"
	"github.com/fastertools/ftl-cli/go/shared/spin"
	"github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis"
	"github.com/fatih/color"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"
)

func newUpCmd() *cobra.Command {
	var build bool
	var watch bool
	var skipSynth bool
	var configFile string

	cmd := &cobra.Command{
		Use:   "up",
		Short: "Run the FTL application locally",
		Long:  `Run the FTL application locally with hot reload support.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := context.Background()

			// Color helpers
			blue := color.New(color.FgBlue).SprintFunc()
			green := color.New(color.FgGreen).SprintFunc()
			yellow := color.New(color.FgYellow).SprintFunc()

			// Ensure spin is installed
			if err := spin.EnsureInstalled(); err != nil {
				return err
			}

			// Check if ftl.yaml exists and synthesize
			if _, err := os.Stat(configFile); err == nil && !skipSynth {
				fmt.Printf("%s Synthesizing spin.toml from %s\n", blue("→"), configFile)

				// Read ftl.yaml
				configData, err := os.ReadFile(configFile)
				if err != nil {
					return fmt.Errorf("failed to read %s: %w", configFile, err)
				}

				// Parse YAML config
				var cfg config.FTLConfig
				if err := yaml.Unmarshal(configData, &cfg); err != nil {
					return fmt.Errorf("failed to parse %s: %w", configFile, err)
				}

				// Create FTL app from config
				app := synthesis.NewApp(cfg.Application.Name)
				if cfg.Application.Version != "" {
					app.SetVersion(cfg.Application.Version)
				}
				if cfg.Application.Description != "" {
					app.SetDescription(cfg.Application.Description)
				}

				// Add components
				for _, comp := range cfg.Components {
					tb := app.AddTool(comp.ID)
					
					// Handle source
					switch src := comp.Source.(type) {
					case string:
						tb.FromLocal(src)
					case map[string]interface{}:
						if registry, ok := src["registry"].(string); ok {
							pkg, _ := src["package"].(string)
							version, _ := src["version"].(string)
							tb.FromRegistry(registry, pkg, version)
						}
					}
					
					// Add environment variables
					for k, v := range comp.Environment {
						tb.WithEnv(k, v)
					}
					
					tb.Build()
				}

				// Synthesize using CUE engine
				synth := synthesis.NewSynthesizer()
				manifest, err := synth.SynthesizeApp(app)
				if err != nil {
					return fmt.Errorf("synthesis failed: %w", err)
				}

				// Write spin.toml
				if err := os.WriteFile("spin.toml", []byte(manifest), 0644); err != nil {
					return fmt.Errorf("failed to write spin.toml: %w", err)
				}

				fmt.Printf("%s Generated spin.toml\n", green("✓"))
			} else if err != nil && !os.IsNotExist(err) {
				return fmt.Errorf("failed to check %s: %w", configFile, err)
			} else if os.IsNotExist(err) {
				// Check if spin.toml exists directly
				if _, err := os.Stat("spin.toml"); os.IsNotExist(err) {
					return fmt.Errorf("neither %s nor spin.toml found. Run 'ftl init' first", configFile)
				}
				fmt.Printf("%s Using existing spin.toml\n", yellow("ℹ"))
			}

			// Build if requested
			if build {
				fmt.Printf("%s Building application first...\n", blue("→"))
				if err := spin.Build(ctx); err != nil {
					return fmt.Errorf("failed to build: %w", err)
				}
				fmt.Printf("%s Build completed\n", green("✓"))
			}

			fmt.Printf("%s Starting FTL application...\n", blue("→"))

			// Run with watch if requested
			if watch {
				fmt.Printf("%s Starting with watch mode...\n", yellow("ℹ"))
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
	cmd.Flags().BoolVar(&skipSynth, "skip-synth", false, "Skip synthesis of spin.toml from ftl.yaml")
	cmd.Flags().StringVarP(&configFile, "config", "c", "ftl.yaml", "Configuration file to synthesize")

	return cmd
}
