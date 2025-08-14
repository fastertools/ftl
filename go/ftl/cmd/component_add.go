package cmd

import (
	"fmt"
	"os"
	"strings"

	"github.com/fatih/color"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"

	"github.com/fastertools/ftl-cli/go/shared/config"
)

// newComponentAddCmd creates the parent 'component add' command
func newComponentAddCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "add",
		Short: "Add components to the application",
		Long: `Add components to the application from various sources.

Use subcommands to specify the source type:
  - local: Add from a local path
  - url: Add from an HTTP/HTTPS URL
  - registry: Add from a Warg/Spin registry
  - oci: Add from an OCI/Docker registry`,
	}

	// Add subcommands for each source type
	cmd.AddCommand(
		newComponentAddLocalCmd(),
		newComponentAddURLCmd(),
		newComponentAddRegistryCmd(),
		newComponentAddOCICmd(),
	)

	return cmd
}

// newComponentAddLocalCmd adds a component from a local path
func newComponentAddLocalCmd() *cobra.Command {
	var (
		name         string
		description  string
		allowedHosts []string
	)

	cmd := &cobra.Command{
		Use:   "local <path>",
		Short: "Add a component from a local path",
		Long: `Add a component from a local file or directory.

Examples:
  # From compiled WASM file
  ftl component add local ./build/component.wasm --name my-component
  
  # From source directory (will need to be built)
  ftl component add local ./src/my-component --name my-component`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			path := args[0]

			if name == "" {
				return fmt.Errorf("--name is required")
			}

			return addComponentLocal(name, path, description, allowedHosts)
		},
	}

	cmd.Flags().StringVar(&name, "name", "", "Component name (required)")
	cmd.Flags().StringVar(&description, "description", "", "Description of the component")
	cmd.Flags().StringSliceVar(&allowedHosts, "allowed-hosts", nil, "Allowed outbound hosts")
	_ = cmd.MarkFlagRequired("name")

	return cmd
}

// newComponentAddURLCmd adds a component from an HTTP URL
func newComponentAddURLCmd() *cobra.Command {
	var (
		name         string
		digest       string
		description  string
		allowedHosts []string
	)

	cmd := &cobra.Command{
		Use:   "url <url>",
		Short: "Add a component from an HTTP/HTTPS URL",
		Long: `Add a component from an HTTP/HTTPS URL.

Examples:
  # Download from URL with digest verification
  ftl component add url https://example.com/component.wasm \
    --name my-component \
    --digest sha256:abc123...`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			url := args[0]

			if name == "" {
				return fmt.Errorf("--name is required")
			}

			if !strings.HasPrefix(url, "http://") && !strings.HasPrefix(url, "https://") {
				return fmt.Errorf("URL must start with http:// or https://")
			}

			if digest == "" {
				return fmt.Errorf("--digest is required for security. Format: sha256:hexdigest")
			}

			return addComponentURL(name, url, digest, description, allowedHosts)
		},
	}

	cmd.Flags().StringVar(&name, "name", "", "Component name (required)")
	cmd.Flags().StringVar(&digest, "digest", "", "SHA256 digest (required, format: sha256:...)")
	cmd.Flags().StringVar(&description, "description", "", "Description of the component")
	cmd.Flags().StringSliceVar(&allowedHosts, "allowed-hosts", nil, "Allowed outbound hosts")
	_ = cmd.MarkFlagRequired("name")
	_ = cmd.MarkFlagRequired("digest")

	return cmd
}

// newComponentAddRegistryCmd adds a component from a Warg/Spin registry
func newComponentAddRegistryCmd() *cobra.Command {
	var (
		name         string
		registry     string
		pkg          string
		version      string
		description  string
		allowedHosts []string
	)

	cmd := &cobra.Command{
		Use:   "registry",
		Short: "Add a component from a Warg/Spin registry",
		Long: `Add a component from a Warg/Spin registry.

Examples:
  # From a Spin registry
  ftl component add registry \
    --name my-component \
    --registry registrytest.fermyon.app \
    --package component:hello-world \
    --version 0.0.1
    
  # From ttl.sh
  ftl component add registry \
    --name my-component \
    --registry ttl.sh \
    --package user:mypackage \
    --version 1.0.0`,
		Args: cobra.NoArgs,
		RunE: func(cmd *cobra.Command, args []string) error {
			if name == "" {
				return fmt.Errorf("--name is required")
			}
			if registry == "" {
				return fmt.Errorf("--registry is required")
			}
			if pkg == "" {
				return fmt.Errorf("--package is required")
			}
			if version == "" {
				return fmt.Errorf("--version is required")
			}

			return addComponentRegistry(name, registry, pkg, version, description, allowedHosts)
		},
	}

	cmd.Flags().StringVar(&name, "name", "", "Component name (required)")
	cmd.Flags().StringVar(&registry, "registry", "", "Registry domain (required)")
	cmd.Flags().StringVar(&pkg, "package", "", "Package name (required)")
	cmd.Flags().StringVar(&version, "version", "", "Package version (required)")
	cmd.Flags().StringVar(&description, "description", "", "Description of the component")
	cmd.Flags().StringSliceVar(&allowedHosts, "allowed-hosts", nil, "Allowed outbound hosts")
	_ = cmd.MarkFlagRequired("name")
	cmd.MarkFlagRequired("registry")
	cmd.MarkFlagRequired("package")
	cmd.MarkFlagRequired("version")

	return cmd
}

// newComponentAddOCICmd adds a component from an OCI/Docker registry
func newComponentAddOCICmd() *cobra.Command {
	var (
		name         string
		description  string
		allowedHosts []string
	)

	cmd := &cobra.Command{
		Use:   "oci <reference>",
		Short: "Add a component from an OCI/Docker registry",
		Long: `Add a component from an OCI/Docker registry. This converts OCI references
to Spin's registry format.

Examples:
  # From GitHub Container Registry
  ftl component add oci ghcr.io/fermyon/spin-hello:latest --name hello
  
  # From Docker Hub
  ftl component add oci myorg/mycomponent:v1.0.0 --name my-component
  
  # From Amazon ECR
  ftl component add oci 123456789.dkr.ecr.us-east-1.amazonaws.com/my-app:latest --name my-app`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			reference := args[0]

			if name == "" {
				return fmt.Errorf("--name is required")
			}

			return addComponentOCI(name, reference, description, allowedHosts)
		},
	}

	cmd.Flags().StringVar(&name, "name", "", "Component name (required)")
	cmd.Flags().StringVar(&description, "description", "", "Description of the component")
	cmd.Flags().StringSliceVar(&allowedHosts, "allowed-hosts", nil, "Allowed outbound hosts")
	_ = cmd.MarkFlagRequired("name")

	return cmd
}

// Implementation functions

func addComponentLocal(name, path, description string, allowedHosts []string) error {
	blue := color.New(color.FgBlue).SprintFunc()
	green := color.New(color.FgGreen).SprintFunc()

	fmt.Printf("%s Adding component '%s' from local path\n", blue("→"), name)
	fmt.Printf("  Path: %s\n", path)

	// Validate component name
	if err := validateComponentName(name); err != nil {
		return err
	}

	// Check if path exists
	info, err := os.Stat(path)
	if err != nil {
		return fmt.Errorf("local path not found: %s", path)
	}

	// Load existing config
	cfg, err := loadSpinConfig("ftl.yaml")
	if err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("ftl.yaml not found. Run 'ftl init' first")
		}
		return err
	}

	// Check for duplicate
	for _, comp := range cfg.Components {
		if comp.ID == name {
			return fmt.Errorf("component '%s' already exists", name)
		}
	}

	// Create component with string source (local path)
	newComp := config.ComponentConfig{
		ID:                   name,
		Source:               path, // Simple string for local path
		Description:          description,
		AllowedOutboundHosts: allowedHosts,
	}

	// If it's a source directory (not .wasm), add build config
	if info.IsDir() || !strings.HasSuffix(path, ".wasm") {
		fmt.Printf("%s This is a source directory. You'll need to configure build settings.\n", blue("ℹ"))
	}

	// Always add a private trigger for FTL components
	trigger := config.TriggerConfig{
		Type:      config.TriggerTypeHTTP,
		Component: name,
		Route:     "private",
	}
	cfg.Triggers = append(cfg.Triggers, trigger)

	cfg.Components = append(cfg.Components, newComp)

	// Save config
	if err := saveSpinConfig("ftl.yaml", cfg); err != nil {
		return fmt.Errorf("failed to save config: %w", err)
	}

	fmt.Printf("%s Component '%s' added successfully\n", green("✓"), name)

	if info.IsDir() || !strings.HasSuffix(path, ".wasm") {
		fmt.Println("\nNext steps:")
		fmt.Println("  1. Configure build settings in ftl.yaml")
		fmt.Println("  2. Run 'ftl build' to compile")
		fmt.Println("  3. Run 'ftl up' to test locally")
	} else {
		fmt.Println("\nNext steps:")
		fmt.Println("  1. Run 'ftl build' to prepare the application")
		fmt.Println("  2. Run 'ftl up' to test locally")
	}

	return nil
}

func addComponentURL(name, url, digest, description string, allowedHosts []string) error {
	blue := color.New(color.FgBlue).SprintFunc()
	green := color.New(color.FgGreen).SprintFunc()

	fmt.Printf("%s Adding component '%s' from URL\n", blue("→"), name)
	fmt.Printf("  URL: %s\n", url)
	fmt.Printf("  Digest: %s\n", digest)

	// Validate component name
	if err := validateComponentName(name); err != nil {
		return err
	}

	// Load existing config
	cfg, err := loadSpinConfig("ftl.yaml")
	if err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("ftl.yaml not found. Run 'ftl init' first")
		}
		return err
	}

	// Check for duplicate
	for _, comp := range cfg.Components {
		if comp.ID == name {
			return fmt.Errorf("component '%s' already exists", name)
		}
	}

	// Create component with URL source (table format)
	// Source needs to be a map for URL type
	sourceMap := map[string]interface{}{
		"url":    url,
		"digest": digest,
	}

	newComp := config.ComponentConfig{
		ID:                   name,
		Source:               sourceMap, // Table format for URL source
		Description:          description,
		AllowedOutboundHosts: allowedHosts,
	}

	// Always add a private trigger for FTL components
	trigger := config.TriggerConfig{
		Type:      config.TriggerTypeHTTP,
		Component: name,
		Route:     "private",
	}
	cfg.Triggers = append(cfg.Triggers, trigger)

	cfg.Components = append(cfg.Components, newComp)

	// Save config
	if err := saveSpinConfig("ftl.yaml", cfg); err != nil {
		return fmt.Errorf("failed to save config: %w", err)
	}

	fmt.Printf("%s Component '%s' added successfully\n", green("✓"), name)
	fmt.Printf("\n%s Component will be downloaded during build/deployment\n", blue("ℹ"))

	fmt.Println("\nNext steps:")
	fmt.Println("  1. Run 'ftl build' to prepare the application")
	fmt.Println("  2. Run 'ftl up' to test locally")

	return nil
}

func addComponentRegistry(name, registry, pkg, version, description string, allowedHosts []string) error {
	blue := color.New(color.FgBlue).SprintFunc()
	green := color.New(color.FgGreen).SprintFunc()

	fmt.Printf("%s Adding component '%s' from registry\n", blue("→"), name)
	fmt.Printf("  Registry: %s\n", registry)
	fmt.Printf("  Package: %s\n", pkg)
	fmt.Printf("  Version: %s\n", version)

	// Validate component name
	if err := validateComponentName(name); err != nil {
		return err
	}

	// Load existing config
	cfg, err := loadSpinConfig("ftl.yaml")
	if err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("ftl.yaml not found. Run 'ftl init' first")
		}
		return err
	}

	// Check for duplicate
	for _, comp := range cfg.Components {
		if comp.ID == name {
			return fmt.Errorf("component '%s' already exists", name)
		}
	}

	// Create component with registry source (table format)
	sourceMap := map[string]interface{}{
		"registry": registry,
		"package":  pkg,
		"version":  version,
	}

	newComp := config.ComponentConfig{
		ID:                   name,
		Source:               sourceMap, // Table format for registry source
		Description:          description,
		AllowedOutboundHosts: allowedHosts,
	}

	// Always add a private trigger for FTL components
	trigger := config.TriggerConfig{
		Type:      config.TriggerTypeHTTP,
		Component: name,
		Route:     "private",
	}
	cfg.Triggers = append(cfg.Triggers, trigger)

	cfg.Components = append(cfg.Components, newComp)

	// Save config
	if err := saveSpinConfig("ftl.yaml", cfg); err != nil {
		return fmt.Errorf("failed to save config: %w", err)
	}

	fmt.Printf("%s Component '%s' added successfully\n", green("✓"), name)
	fmt.Printf("\n%s Registry components will be pulled during deployment\n", blue("ℹ"))

	fmt.Println("\nNext steps:")
	fmt.Println("  1. Run 'ftl build' to prepare the application")
	fmt.Println("  2. Run 'ftl up' to test locally")

	return nil
}

func addComponentOCI(name, reference, description string, allowedHosts []string) error {
	blue := color.New(color.FgBlue).SprintFunc()
	green := color.New(color.FgGreen).SprintFunc()

	fmt.Printf("%s Adding component '%s' from OCI registry\n", blue("→"), name)
	fmt.Printf("  Reference: %s\n", reference)

	// Parse OCI reference to extract registry, package, and version
	// Format: [registry/]namespace/image[:tag|@digest]

	var registry, pkg, version string

	// Handle different OCI formats
	parts := strings.Split(reference, "/")

	if len(parts) >= 3 {
		// Full format: registry.io/namespace/image:tag
		registry = parts[0]
		imageParts := strings.Split(parts[len(parts)-1], ":")
		if len(imageParts) == 2 {
			// namespace/image:tag format
			pkg = strings.Join(parts[1:len(parts)-1], "/") + "/" + imageParts[0]
			version = imageParts[1]
		} else {
			// No tag specified, use latest
			pkg = strings.Join(parts[1:], "/")
			version = "latest"
		}
	} else if len(parts) == 2 {
		// Docker Hub shorthand: namespace/image:tag
		registry = "docker.io"
		imageParts := strings.Split(parts[1], ":")
		if len(imageParts) == 2 {
			pkg = parts[0] + "/" + imageParts[0]
			version = imageParts[1]
		} else {
			pkg = reference
			version = "latest"
		}
	} else {
		// Single name, assume Docker Hub official image
		registry = "docker.io"
		imageParts := strings.Split(reference, ":")
		if len(imageParts) == 2 {
			pkg = "library/" + imageParts[0]
			version = imageParts[1]
		} else {
			pkg = "library/" + reference
			version = "latest"
		}
	}

	fmt.Printf("  → Registry: %s\n", registry)
	fmt.Printf("  → Package: %s\n", pkg)
	fmt.Printf("  → Version: %s\n", version)

	// Validate component name
	if err := validateComponentName(name); err != nil {
		return err
	}

	// Load existing config
	cfg, err := loadSpinConfig("ftl.yaml")
	if err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("ftl.yaml not found. Run 'ftl init' first")
		}
		return err
	}

	// Check for duplicate
	for _, comp := range cfg.Components {
		if comp.ID == name {
			return fmt.Errorf("component '%s' already exists", name)
		}
	}

	// Create component with registry source (table format)
	// OCI references get converted to Spin's registry format
	sourceMap := map[string]interface{}{
		"registry": registry,
		"package":  pkg,
		"version":  version,
	}

	newComp := config.ComponentConfig{
		ID:                   name,
		Source:               sourceMap, // Table format for registry source
		Description:          description,
		AllowedOutboundHosts: allowedHosts,
	}

	// Always add a private trigger for FTL components
	trigger := config.TriggerConfig{
		Type:      config.TriggerTypeHTTP,
		Component: name,
		Route:     "private",
	}
	cfg.Triggers = append(cfg.Triggers, trigger)

	cfg.Components = append(cfg.Components, newComp)

	// Save config
	if err := saveSpinConfig("ftl.yaml", cfg); err != nil {
		return fmt.Errorf("failed to save config: %w", err)
	}

	fmt.Printf("%s Component '%s' added successfully\n", green("✓"), name)
	fmt.Printf("\n%s OCI registry components will be pulled during deployment\n", blue("ℹ"))

	fmt.Println("\nNext steps:")
	fmt.Println("  1. Run 'ftl build' to prepare the application")
	fmt.Println("  2. Run 'ftl up' to test locally")

	return nil
}

func validateComponentName(name string) error {
	if name == "" {
		return fmt.Errorf("component name cannot be empty")
	}

	// Check for valid characters (lowercase, numbers, hyphens, underscores)
	for _, c := range name {
		if !((c >= 'a' && c <= 'z') || (c >= '0' && c <= '9') || c == '-' || c == '_') {
			return fmt.Errorf("component name must contain only lowercase letters, numbers, hyphens, and underscores")
		}
	}

	// Check for leading/trailing or double special characters
	if strings.HasPrefix(name, "-") || strings.HasPrefix(name, "_") ||
		strings.HasSuffix(name, "-") || strings.HasSuffix(name, "_") ||
		strings.Contains(name, "--") || strings.Contains(name, "__") {
		return fmt.Errorf("component name cannot start/end with or contain double hyphens/underscores")
	}

	return nil
}

func loadSpinConfig(path string) (*config.FTLConfig, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	var cfg config.FTLConfig
	if err := yaml.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse config: %w", err)
	}

	return &cfg, nil
}

func saveSpinConfig(path string, cfg *config.FTLConfig) error {
	data, err := yaml.Marshal(cfg)
	if err != nil {
		return fmt.Errorf("failed to marshal config: %w", err)
	}

	return os.WriteFile(path, data, 0644)
}
