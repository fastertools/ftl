package cli

import (
	"context"
	"fmt"
	"os"

	"github.com/fastertools/ftl-cli/pkg/spin"
	"github.com/fastertools/ftl-cli/pkg/synthesis"
	"github.com/fatih/color"
	"github.com/spf13/cobra"
)

func newUpCmd() *cobra.Command {
	var build bool
	var watch bool
	var skipSynth bool
	var configFile string
	
	// Spin up specific flags
	var componentIDs []string
	var cacheDir string
	var directMounts bool
	var env []string
	var from string
	var insecure bool
	var temp string
	var allowTransientWrite bool
	var cache string
	var disableCache bool
	var disablePooling bool
	var follow []string
	var keyValue []string
	var logDir string
	var maxInstanceMemory string
	var quiet bool
	var runtimeConfigFile string
	var sqlite []string
	var stateDir string
	var listen string

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

			// Auto-detect config file if not specified
			if configFile == "" {
				// Try to detect the config format
				if _, err := os.Stat("ftl.yaml"); err == nil {
					configFile = "ftl.yaml"
				} else if _, err := os.Stat("ftl.json"); err == nil {
					configFile = "ftl.json"
				} else if _, err := os.Stat("app.cue"); err == nil {
					configFile = "app.cue"
				} else if _, err := os.Stat("main.go"); err == nil {
					configFile = "main.go"
				}
			}

			// Check if config file exists and synthesize
			if configFile != "" && !skipSynth {
				if _, err := os.Stat(configFile); err == nil {
					fmt.Printf("%s Synthesizing spin.toml from %s\n", blue("→"), configFile)

					// Use unified synthesis helper
					manifest, err := synthesis.SynthesizeFromConfig(configFile)
					if err != nil {
						return fmt.Errorf("synthesis failed: %w", err)
					}

					// Write spin.toml
					if err := os.WriteFile("spin.toml", []byte(manifest), 0600); err != nil {
						return fmt.Errorf("failed to write spin.toml: %w", err)
					}

					fmt.Printf("%s Generated spin.toml\n", green("✓"))
				}
			} else if configFile == "" && !skipSynth {
				// No config file found, check for spin.toml
				if _, err := os.Stat("spin.toml"); os.IsNotExist(err) {
					return fmt.Errorf("no ftl.yaml, ftl.json, app.cue, or spin.toml found. Run 'ftl init' first")
				}
				fmt.Printf("%s No FTL config found, using existing spin.toml\n", yellow("ℹ"))
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

			// Build options array for spin up/watch command
			var spinOptions []string
			
			// Add component IDs
			for _, id := range componentIDs {
				spinOptions = append(spinOptions, "--component-id", id)
			}
			
			// Add cache directory
			if cacheDir != "" {
				spinOptions = append(spinOptions, "--cache-dir", cacheDir)
			}
			
			// Add direct mounts flag
			if directMounts {
				spinOptions = append(spinOptions, "--direct-mounts")
			}
			
			// Add environment variables
			for _, e := range env {
				spinOptions = append(spinOptions, "--env", e)
			}
			
			// Add from flag
			if from != "" {
				spinOptions = append(spinOptions, "--from", from)
			}
			
			// Add insecure flag
			if insecure {
				spinOptions = append(spinOptions, "--insecure")
			}
			
			// Add temp directory
			if temp != "" {
				spinOptions = append(spinOptions, "--temp", temp)
			}
			
			// Add transient write flag
			if allowTransientWrite {
				spinOptions = append(spinOptions, "--allow-transient-write")
			}
			
			// Add cache file
			if cache != "" {
				spinOptions = append(spinOptions, "--cache", cache)
			}
			
			// Add disable cache flag
			if disableCache {
				spinOptions = append(spinOptions, "--disable-cache")
			}
			
			// Add disable pooling flag
			if disablePooling {
				spinOptions = append(spinOptions, "--disable-pooling")
			}
			
			// Add follow components
			for _, f := range follow {
				spinOptions = append(spinOptions, "--follow", f)
			}
			
			// Add key-value pairs
			for _, kv := range keyValue {
				spinOptions = append(spinOptions, "--key-value", kv)
			}
			
			// Add log directory
			if logDir != "" {
				spinOptions = append(spinOptions, "--log-dir", logDir)
			}
			
			// Add max instance memory
			if maxInstanceMemory != "" {
				spinOptions = append(spinOptions, "--max-instance-memory", maxInstanceMemory)
			}
			
			// Add quiet flag
			if quiet {
				spinOptions = append(spinOptions, "--quiet")
			}
			
			// Add runtime config file
			if runtimeConfigFile != "" {
				spinOptions = append(spinOptions, "--runtime-config-file", runtimeConfigFile)
			}
			
			// Add SQLite statements
			for _, sql := range sqlite {
				spinOptions = append(spinOptions, "--sqlite", sql)
			}
			
			// Add state directory
			if stateDir != "" {
				spinOptions = append(spinOptions, "--state-dir", stateDir)
			}
			
			// Add listen address
			if listen != "" {
				spinOptions = append(spinOptions, "--listen", listen)
			}

			// Run with watch if requested
			if watch {
				fmt.Printf("%s Starting with watch mode...\n", yellow("ℹ"))
				if err := spin.Watch(ctx, spinOptions...); err != nil {
					return fmt.Errorf("failed to start with watch: %w", err)
				}
			} else {
				// Run normally
				if err := spin.Up(ctx, spinOptions...); err != nil {
					return fmt.Errorf("failed to start: %w", err)
				}
			}

			return nil
		},
	}

	// FTL-specific flags
	cmd.Flags().BoolVarP(&build, "build", "b", false, "Build before running")
	cmd.Flags().BoolVarP(&watch, "watch", "w", false, "Watch for changes and reload")
	cmd.Flags().BoolVar(&skipSynth, "skip-synth", false, "Skip synthesis of spin.toml from FTL config")
	cmd.Flags().StringVarP(&configFile, "config", "c", "", "Configuration file to synthesize (auto-detects if not specified)")

	// Spin up pass-through flags
	cmd.Flags().StringArrayVar(&componentIDs, "component-id", nil, "[Experimental] Component ID to run. This can be specified multiple times. The default is all components")
	cmd.Flags().StringVar(&cacheDir, "cache-dir", "", "Cache directory for downloaded components and assets")
	cmd.Flags().BoolVar(&directMounts, "direct-mounts", false, "For local apps with directory mounts and no excluded files, mount them directly instead of using a temporary directory")
	cmd.Flags().StringArrayVarP(&env, "env", "e", nil, "Pass an environment variable (key=value) to all components of the application")
	cmd.Flags().StringVarP(&from, "from", "f", "", "The application to run. This may be a manifest (spin.toml) file, a directory containing a spin.toml file, a remote registry reference, or a Wasm module (a .wasm file). If omitted, it defaults to \"spin.toml\"")
	cmd.Flags().BoolVarP(&insecure, "insecure", "k", false, "Ignore server certificate errors from a registry")
	cmd.Flags().StringVar(&temp, "temp", "", "Temporary directory for the static assets of the components")
	
	// Trigger options
	cmd.Flags().BoolVar(&allowTransientWrite, "allow-transient-write", false, "Set the static assets of the components in the temporary directory as writable")
	cmd.Flags().StringVar(&cache, "cache", "", "Wasmtime cache configuration file")
	cmd.Flags().BoolVar(&disableCache, "disable-cache", false, "Disable Wasmtime cache")
	cmd.Flags().BoolVar(&disablePooling, "disable-pooling", false, "Disable Wasmtime's pooling instance allocator")
	cmd.Flags().StringArrayVar(&follow, "follow", nil, "Print output to stdout/stderr only for given component(s)")
	cmd.Flags().StringArrayVar(&keyValue, "key-value", nil, "Set a key/value pair (key=value) in the application's default store. Any existing value will be overwritten. Can be used multiple times")
	cmd.Flags().StringVarP(&logDir, "log-dir", "L", "", "Log directory for the stdout and stderr of components. Setting to the empty string disables logging to disk")
	cmd.Flags().StringVar(&maxInstanceMemory, "max-instance-memory", "", "Sets the maximum memory allocation limit for an instance in bytes")
	cmd.Flags().BoolVarP(&quiet, "quiet", "q", false, "Silence all component output to stdout/stderr")
	cmd.Flags().StringVar(&runtimeConfigFile, "runtime-config-file", "", "Configuration file for config providers and wasmtime config")
	cmd.Flags().StringArrayVar(&sqlite, "sqlite", nil, "Run a SQLite statement such as a migration against the default database. To run from a file, prefix the filename with @ e.g. spin up --sqlite @migration.sql")
	cmd.Flags().StringVar(&stateDir, "state-dir", "", "Set the application state directory path. This is used in the default locations for logs, key value stores, etc.")
	cmd.Flags().StringVar(&listen, "listen", "", "Set the listen address for HTTP applications (default: localhost:3000)")

	return cmd
}
