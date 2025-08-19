package cli

import (
	"context"
	"fmt"
	"os"

	"github.com/AlecAivazis/survey/v2"
	"github.com/fatih/color"
	"github.com/google/uuid"
	"github.com/spf13/cobra"

	"github.com/fastertools/ftl-cli/internal/api"
	"github.com/fastertools/ftl-cli/internal/auth"
)

func newDeleteCmd() *cobra.Command {
	var force bool

	cmd := &cobra.Command{
		Use:   "delete <app-id|app-name>",
		Short: "Delete an FTL application",
		Long:  `Delete an FTL application from the platform.`,
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := context.Background()
			return runDelete(ctx, args[0], force)
		},
	}

	cmd.Flags().BoolVarP(&force, "force", "f", false, "Force deletion without confirmation")

	return cmd
}

// Allow overriding for tests
var runDelete = runDeleteImpl

func runDeleteImpl(ctx context.Context, appIdentifier string, force bool) error {
	// Initialize auth manager
	store, err := auth.NewKeyringStore()
	if err != nil {
		return fmt.Errorf("failed to initialize credential store: %w", err)
	}
	authManager := auth.NewManager(store, nil)

	// Check authentication
	if _, err := authManager.GetToken(ctx); err != nil {
		return fmt.Errorf("not logged in to FTL. Run 'ftl auth login' first")
	}

	// Create API client
	apiClient, err := api.NewFTLClient(authManager, "")
	if err != nil {
		return fmt.Errorf("failed to create API client: %w", err)
	}

	// Get app details first to show what will be deleted
	var app *api.App

	// Try to parse as UUID
	if _, err := uuid.Parse(appIdentifier); err == nil {
		// It's a UUID, get app directly
		app, err = apiClient.GetApp(ctx, appIdentifier)
		if err != nil {
			return fmt.Errorf("failed to get app: %w", err)
		}
	} else {
		// Not a UUID, assume it's a name - list apps with name filter
		response, err := apiClient.ListApps(ctx, &api.ListAppsParams{
			Name: &appIdentifier,
		})
		if err != nil {
			return fmt.Errorf("failed to list apps: %w", err)
		}

		if len(response.Apps) == 0 {
			return fmt.Errorf("application '%s' not found", appIdentifier)
		}

		// Get full details using the UUID
		appID := response.Apps[0].AppId.String()
		app, err = apiClient.GetApp(ctx, appID)
		if err != nil {
			return fmt.Errorf("failed to get app details: %w", err)
		}
	}

	// Display what will be deleted using shared DataWriter
	color.Yellow("Application to be deleted:")
	dw := NewDataWriter(colorOutput, "table")
	kvb := NewKeyValueBuilder("")
	kvb.Add("Name", app.AppName)
	kvb.Add("ID", app.AppId.String())
	if app.ProviderUrl != nil && *app.ProviderUrl != "" {
		kvb.Add("URL", *app.ProviderUrl)
	}
	if err := kvb.Write(dw); err != nil {
		return fmt.Errorf("failed to display app details: %w", err)
	}

	// Ask for confirmation unless --force is used
	if !force {
		// Check if we're in an interactive terminal
		if !isInteractive() {
			return fmt.Errorf("deletion requires confirmation. Use --force to skip confirmation in non-interactive mode")
		}

		_, _ = color.New(color.FgRed, color.Bold).Println("⚠️  This action cannot be undone!")

		// Ask user to type the app name to confirm
		prompt := &survey.Input{
			Message: fmt.Sprintf("Type '%s' to confirm deletion:", app.AppName),
		}

		var confirmation string
		if err := survey.AskOne(prompt, &confirmation); err != nil {
			return fmt.Errorf("failed to get confirmation: %w", err)
		}

		if confirmation != app.AppName {
			color.Yellow("Deletion cancelled.")
			return nil
		}
	}

	// Perform deletion
	Info("Deleting application...")

	err = apiClient.DeleteApp(ctx, app.AppId.String())
	if err != nil {
		return fmt.Errorf("failed to delete app: %w", err)
	}

	Success("Application deleted successfully")

	return nil
}

// isInteractive checks if we're running in an interactive terminal
func isInteractive() bool {
	fileInfo, err := os.Stdin.Stat()
	if err != nil {
		return false
	}
	// Check if stdin is a terminal (not a pipe or file)
	return fileInfo.Mode()&os.ModeCharDevice != 0
}
