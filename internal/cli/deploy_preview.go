package cli

import (
	"fmt"
	"os"
	"strings"
	"text/tabwriter"

	"github.com/AlecAivazis/survey/v2"
	"github.com/fatih/color"

	"github.com/fastertools/ftl/validation"
)

// DeploymentPreview represents the deployment preview information
type DeploymentPreview struct {
	IsUpdate       bool
	AppName        string
	AppID          string
	AccessMode     string
	Environment    string
	Organization   string
	Components     []ComponentPreview
	Variables      map[string]string
	ExistingAppID  string
	ExistingAccess string
	Changes        *DeploymentChanges
}

// ComponentPreview represents a component in the preview
type ComponentPreview struct {
	Name     string
	Type     string
	Source   string
	Size     string
	IsLocal  bool
	Registry string
	Version  string
}

// DeploymentChanges tracks what's changing in an update
type DeploymentChanges struct {
	AccessModeChanged  bool
	OldAccessMode      string
	NewAccessMode      string
	ComponentsAdded    []string
	ComponentsRemoved  []string
	ComponentsUpdated  []string
	VariablesChanged   map[string]VariableChange
	EnvironmentChanged bool
	OldEnvironment     string
	NewEnvironment     string
}

// VariableChange represents a variable modification
type VariableChange struct {
	Old   string
	New   string
	Added bool
}

// ShowDeploymentPreview displays a comprehensive deployment preview
func ShowDeploymentPreview(preview *DeploymentPreview) {
	fmt.Println()

	// Header
	if preview.IsUpdate {
		fmt.Printf("%s\n", color.New(color.FgYellow, color.Bold).Sprint("📋 DEPLOYMENT UPDATE PREVIEW"))
	} else {
		fmt.Printf("%s\n", color.New(color.FgGreen, color.Bold).Sprint("🚀 NEW DEPLOYMENT PREVIEW"))
	}

	fmt.Println(strings.Repeat("─", 60))

	// Basic Information
	fmt.Printf("  %s %s\n", color.New(color.Bold).Sprint("App Name:"), preview.AppName)
	if preview.AppID != "" {
		fmt.Printf("  %s %s\n", color.New(color.Bold).Sprint("App ID:"), color.New(color.FgCyan).Sprint(preview.AppID))
	}

	// Access Mode with visual indicator
	accessColor := getAccessModeColor(preview.AccessMode)
	accessIcon := getAccessModeIcon(preview.AccessMode)
	fmt.Printf("  %s %s %s\n",
		color.New(color.Bold).Sprint("Access Mode:"),
		accessIcon,
		accessColor.Sprint(preview.AccessMode))

	// Show access mode implications
	showAccessModeImplications(preview.AccessMode)

	// Environment
	envColor := color.New(color.FgYellow)
	if preview.Environment == "production" {
		envColor = color.New(color.FgRed, color.Bold)
	}
	fmt.Printf("  %s %s\n",
		color.New(color.Bold).Sprint("Environment:"),
		envColor.Sprint(preview.Environment))

	// Organization
	if preview.Organization != "" {
		fmt.Printf("  %s %s\n",
			color.New(color.Bold).Sprint("Organization:"),
			color.New(color.FgMagenta).Sprint(preview.Organization))
	}

	fmt.Println(strings.Repeat("─", 60))

	// Components Section
	fmt.Printf("\n%s (%d)\n",
		color.New(color.Bold).Sprint("Components"),
		len(preview.Components))

	if len(preview.Components) > 0 {
		w := tabwriter.NewWriter(os.Stdout, 0, 0, 2, ' ', 0)
		_, _ = fmt.Fprintln(w, "  NAME\tTYPE\tSOURCE\tSIZE")
		_, _ = fmt.Fprintln(w, "  ────\t────\t──────\t────")

		for _, comp := range preview.Components {
			sourceIcon := "📦"
			if comp.IsLocal {
				sourceIcon = "💻"
			}
			_, _ = fmt.Fprintf(w, "  %s\t%s\t%s %s\t%s\n",
				comp.Name,
				comp.Type,
				sourceIcon,
				comp.Source,
				comp.Size)
		}
		_ = w.Flush()
	}

	// Variables Section (if any)
	if len(preview.Variables) > 0 {
		fmt.Printf("\n%s (%d)\n",
			color.New(color.Bold).Sprint("Variables"),
			len(preview.Variables))

		for key, value := range preview.Variables {
			// Mask sensitive values
			displayValue := value
			if isSensitiveKey(key) {
				displayValue = maskValue(value)
			}
			fmt.Printf("  %s = %s\n",
				color.New(color.FgBlue).Sprint(key),
				displayValue)
		}
	}

	// Changes Section (for updates)
	if preview.IsUpdate && preview.Changes != nil {
		showDeploymentChanges(preview.Changes)
	}

	fmt.Println(strings.Repeat("─", 60))
}

// showDeploymentChanges displays what's changing in an update
func showDeploymentChanges(changes *DeploymentChanges) {
	fmt.Printf("\n%s\n", color.New(color.FgYellow, color.Bold).Sprint("⚡ Changes"))

	hasChanges := false

	// Access mode change (critical!)
	if changes.AccessModeChanged {
		hasChanges = true
		oldColor := getAccessModeColor(changes.OldAccessMode)
		newColor := getAccessModeColor(changes.NewAccessMode)

		fmt.Printf("  %s Access Mode: %s → %s\n",
			color.New(color.FgRed).Sprint("⚠"),
			oldColor.Sprint(changes.OldAccessMode),
			newColor.Sprint(changes.NewAccessMode))

		// Warn about implications
		if changes.OldAccessMode == "public" && changes.NewAccessMode != "public" {
			fmt.Printf("    %s App will no longer be publicly accessible\n",
				color.New(color.FgYellow).Sprint("→"))
		} else if changes.OldAccessMode != "public" && changes.NewAccessMode == "public" {
			fmt.Printf("    %s App will become publicly accessible!\n",
				color.New(color.FgRed, color.Bold).Sprint("→"))
		}
	}

	// Environment change
	if changes.EnvironmentChanged {
		hasChanges = true
		fmt.Printf("  %s Environment: %s → %s\n",
			color.New(color.FgYellow).Sprint("⚡"),
			changes.OldEnvironment,
			changes.NewEnvironment)
	}

	// Component changes
	if len(changes.ComponentsAdded) > 0 {
		hasChanges = true
		fmt.Printf("  %s Components added: %s\n",
			color.New(color.FgGreen).Sprint("+"),
			strings.Join(changes.ComponentsAdded, ", "))
	}

	if len(changes.ComponentsRemoved) > 0 {
		hasChanges = true
		fmt.Printf("  %s Components removed: %s\n",
			color.New(color.FgRed).Sprint("-"),
			strings.Join(changes.ComponentsRemoved, ", "))
	}

	if len(changes.ComponentsUpdated) > 0 {
		hasChanges = true
		fmt.Printf("  %s Components updated: %s\n",
			color.New(color.FgBlue).Sprint("~"),
			strings.Join(changes.ComponentsUpdated, ", "))
	}

	// Variable changes
	if len(changes.VariablesChanged) > 0 {
		hasChanges = true
		fmt.Printf("  %s Variables:\n", color.New(color.FgCyan).Sprint("○"))
		for key, change := range changes.VariablesChanged {
			if change.Added {
				fmt.Printf("    %s %s = %s\n",
					color.New(color.FgGreen).Sprint("+"),
					key,
					maskIfSensitive(key, change.New))
			} else {
				fmt.Printf("    %s %s: %s → %s\n",
					color.New(color.FgBlue).Sprint("~"),
					key,
					maskIfSensitive(key, change.Old),
					maskIfSensitive(key, change.New))
			}
		}
	}

	if !hasChanges {
		fmt.Printf("  %s No configuration changes detected\n",
			color.New(color.FgGreen).Sprint("✓"))
	}
}

// showAccessModeImplications explains what the access mode means
func showAccessModeImplications(mode string) {
	implications := ""
	switch mode {
	case "public":
		implications = "Anyone on the internet can access this app"
	case "private":
		implications = "Only you can access this app"
	case "org":
		implications = "Only members of your organization can access"
	case "custom":
		implications = "Custom authentication rules apply"
	}

	if implications != "" {
		fmt.Printf("  %s %s\n",
			color.New(color.FgYellow).Sprint("→"),
			color.New(color.Italic).Sprint(implications))
	}
}

// getAccessModeColor returns the appropriate color for an access mode
func getAccessModeColor(mode string) *color.Color {
	switch mode {
	case "public":
		return color.New(color.FgRed, color.Bold)
	case "private":
		return color.New(color.FgGreen)
	case "org":
		return color.New(color.FgBlue)
	case "custom":
		return color.New(color.FgMagenta)
	default:
		return color.New(color.FgWhite)
	}
}

// getAccessModeIcon returns an icon for the access mode
func getAccessModeIcon(mode string) string {
	switch mode {
	case "public":
		return "🌍"
	case "private":
		return "🔒"
	case "org":
		return "👥"
	case "custom":
		return "🔧"
	default:
		return "❓"
	}
}

// isSensitiveKey checks if a variable key is sensitive
func isSensitiveKey(key string) bool {
	lowerKey := strings.ToLower(key)
	sensitivePatterns := []string{
		"password", "secret", "token", "key", "api", "auth",
		"credential", "private", "cert", "ssh",
	}

	for _, pattern := range sensitivePatterns {
		if strings.Contains(lowerKey, pattern) {
			return true
		}
	}
	return false
}

// maskValue masks a sensitive value
func maskValue(value string) string {
	if len(value) <= 4 {
		return "****"
	}
	return value[:2] + "****" + value[len(value)-2:]
}

// maskIfSensitive masks a value if the key is sensitive
func maskIfSensitive(key, value string) string {
	if isSensitiveKey(key) {
		return maskValue(value)
	}
	return value
}

// ConfirmDeployment prompts for deployment confirmation with preview
func ConfirmDeployment(preview *DeploymentPreview, forceYes bool) (bool, error) {
	if forceYes {
		return true, nil
	}

	// Show the preview
	ShowDeploymentPreview(preview)

	// Build confirmation message
	message := "Deploy this application"

	// Add warnings for critical changes
	if preview.IsUpdate && preview.Changes != nil {
		if preview.Changes.AccessModeChanged {
			if preview.Changes.NewAccessMode == "public" {
				message = "⚠️  This will make the app PUBLIC. Deploy"
			} else if preview.Changes.OldAccessMode == "public" {
				message = "⚠️  This will restrict app access. Deploy"
			}
		}
	}

	// For production deployments, add extra warning
	if preview.Environment == "production" {
		if !preview.IsUpdate {
			message = "🔴 Deploy NEW app to PRODUCTION"
		} else {
			message = "🔴 Update PRODUCTION app"
		}
	}

	// Interactive confirmation with no default - user must explicitly choose
	// This follows the CDK pattern for safety
	confirm := false

	// We need to loop until we get a valid response
	for {
		response := ""
		prompt := &survey.Input{
			Message: message + "? (y/n)",
			Help:    "You must explicitly type 'y' for yes or 'n' for no",
		}

		err := survey.AskOne(prompt, &response, survey.WithValidator(func(val interface{}) error {
			str, ok := val.(string)
			if !ok {
				return fmt.Errorf("invalid response")
			}
			str = strings.ToLower(strings.TrimSpace(str))
			if str != "y" && str != "yes" && str != "n" && str != "no" {
				return fmt.Errorf("please enter 'y' for yes or 'n' for no")
			}
			return nil
		}))

		if err != nil {
			return false, err
		}

		response = strings.ToLower(strings.TrimSpace(response))
		if response == "y" || response == "yes" {
			confirm = true
			break
		} else if response == "n" || response == "no" {
			confirm = false
			break
		}
	}

	return confirm, nil
}

// BuildDeploymentPreviewWithOrg creates a preview with full org information
func BuildDeploymentPreviewWithOrg(
	manifest *validation.Application,
	opts *DeployOptions,
	existingAppID string,
	existingAccess string,
	orgID string,
	orgName string,
) *DeploymentPreview {
	preview := BuildDeploymentPreview(manifest, opts, existingAppID, existingAccess, orgID)

	// Use org name if available, otherwise fall back to ID
	if orgName != "" {
		preview.Organization = orgName
	} else if orgID != "" {
		preview.Organization = orgID
	}

	return preview
}

// BuildDeploymentPreview creates a preview from the manifest and context
func BuildDeploymentPreview(
	manifest *validation.Application,
	opts *DeployOptions,
	existingAppID string,
	existingAccess string,
	orgID string,
) *DeploymentPreview {
	preview := &DeploymentPreview{
		IsUpdate:       existingAppID != "",
		AppName:        manifest.Name,
		AccessMode:     manifest.Access,
		Environment:    opts.Environment,
		Organization:   orgID,
		Variables:      opts.Variables,
		ExistingAppID:  existingAppID,
		ExistingAccess: existingAccess,
	}

	// Add existing app info
	if existingAppID != "" {
		preview.AppID = existingAppID

		// Calculate changes
		preview.Changes = calculateChanges(manifest, opts, existingAccess)
	}

	// Build component list
	for _, comp := range manifest.Components {
		compPreview := ComponentPreview{
			Name: comp.ID,
			Type: "wasm",
		}

		// Check source type
		switch src := comp.Source.(type) {
		case *validation.LocalSource:
			compPreview.IsLocal = true
			compPreview.Source = src.Path
			// Get file size if possible
			if info, err := os.Stat(src.Path); err == nil {
				compPreview.Size = formatFileSize(info.Size())
			}
		case *validation.RegistrySource:
			compPreview.IsLocal = false
			compPreview.Source = src.Package
			compPreview.Registry = src.Registry
			compPreview.Version = src.Version
		}

		preview.Components = append(preview.Components, compPreview)
	}

	return preview
}

// calculateChanges determines what's changing in an update
func calculateChanges(
	manifest *validation.Application,
	opts *DeployOptions,
	existingAccess string,
) *DeploymentChanges {
	changes := &DeploymentChanges{
		VariablesChanged: make(map[string]VariableChange),
	}

	// Check access mode change
	if existingAccess != "" && existingAccess != manifest.Access {
		changes.AccessModeChanged = true
		changes.OldAccessMode = existingAccess
		changes.NewAccessMode = manifest.Access
	}

	// Check environment change (if we had previous env info)
	// This would require storing environment in app metadata

	// Component changes would require comparing with existing deployment
	// For now, we'll mark all as updated
	for _, comp := range manifest.Components {
		changes.ComponentsUpdated = append(changes.ComponentsUpdated, comp.ID)
	}

	return changes
}

// formatFileSize formats bytes into human-readable size
func formatFileSize(bytes int64) string {
	const unit = 1024
	if bytes < unit {
		return fmt.Sprintf("%d B", bytes)
	}
	div, exp := int64(unit), 0
	for n := bytes / unit; n >= unit; n /= unit {
		div *= unit
		exp++
	}
	return fmt.Sprintf("%.1f %cB", float64(bytes)/float64(div), "KMGTPE"[exp])
}
