package cmd

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"

	"github.com/fatih/color"
	"github.com/google/uuid"
	"github.com/spf13/cobra"

	"github.com/fastertools/ftl-cli/go/shared/api"
	"github.com/fastertools/ftl-cli/go/shared/auth"
)

func newStatusCmd() *cobra.Command {
	var format string

	cmd := &cobra.Command{
		Use:   "status <app-id|app-name>",
		Short: "Get status of an FTL application",
		Long:  `Get detailed status information for a specific FTL application.`,
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := context.Background()
			return runStatus(ctx, args[0], format)
		},
	}

	cmd.Flags().StringVarP(&format, "output", "o", "table", "Output format (table, json)")

	return cmd
}

// Allow overriding for tests
var runStatus = runStatusImpl

func runStatusImpl(ctx context.Context, appIdentifier string, format string) error {
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

	// Determine if identifier is UUID or name
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

	switch format {
	case "json":
		return displayAppStatusJSON(app)
	case "table":
		return displayAppStatusTable(app)
	default:
		return fmt.Errorf("invalid output format: %s (use 'table' or 'json')", format)
	}
}

func displayAppStatusTable(app *api.App) error {
	fmt.Fprintln(colorOutput)
	color.New(color.Bold).Fprintln(colorOutput, "Application Details")
	
	fmt.Fprintf(colorOutput, "  Name:         %s\n", app.AppName)
	fmt.Fprintf(colorOutput, "  ID:           %s\n", app.AppId.String())
	
	// Color-code status
	statusColor := color.New(color.FgWhite)
	switch app.Status {
	case api.AppStatusACTIVE:
		statusColor = color.New(color.FgGreen)
	case api.AppStatusFAILED:
		statusColor = color.New(color.FgRed)
	case api.AppStatusPENDING, api.AppStatusCREATING:
		statusColor = color.New(color.FgYellow)
	case api.AppStatusDELETED, api.AppStatusDELETING:
		statusColor = color.New(color.FgHiBlack)
	}
	fmt.Fprintf(colorOutput, "  Status:       %s\n", statusColor.Sprint(app.Status))
	
	if app.ProviderUrl != nil && *app.ProviderUrl != "" {
		fmt.Fprintf(colorOutput, "  URL:          %s\n", *app.ProviderUrl)
	}
	
	if app.ProviderError != nil && *app.ProviderError != "" {
		fmt.Fprintf(colorOutput, "  %s\n", color.RedString("Error:        %s", *app.ProviderError))
	}
	
	// Timestamps are already strings from the API
	fmt.Fprintf(colorOutput, "  Created:      %s\n", app.CreatedAt)
	fmt.Fprintf(colorOutput, "  Updated:      %s\n", app.UpdatedAt)
	
	// Show access control
	if app.AccessControl != nil {
		access := strings.ToLower(string(*app.AccessControl))
		fmt.Fprintf(colorOutput, "  Access:       %s\n", access)
	}
	
	fmt.Fprintln(colorOutput)
	
	return nil
}

func displayAppStatusJSON(app *api.App) error {
	encoder := json.NewEncoder(colorOutput)
	encoder.SetIndent("", "  ")
	return encoder.Encode(app)
}