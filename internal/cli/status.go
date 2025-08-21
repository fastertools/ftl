package cli

import (
	"context"
	"fmt"
	"strings"

	"github.com/google/uuid"
	"github.com/spf13/cobra"

	"github.com/fastertools/ftl/internal/api"
	"github.com/fastertools/ftl/internal/auth"
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

	// Use shared DataWriter for consistent output
	dw := NewDataWriter(colorOutput, format)

	switch format {
	case "json":
		return dw.WriteStruct(app)
	case "table":
		return displayAppStatusTable(app)
	default:
		return fmt.Errorf("invalid output format: %s (use 'table' or 'json')", format)
	}
}

func displayAppStatusTable(app *api.App) error {
	dw := NewDataWriter(colorOutput, "table")
	kvb := NewKeyValueBuilder("Application Details")

	// Add basic fields
	kvb.Add("Name", app.AppName)
	kvb.Add("ID", app.AppId.String())
	kvb.Add("Status", app.Status)

	// Add optional fields
	if app.ProviderUrl != nil && *app.ProviderUrl != "" {
		kvb.Add("URL", *app.ProviderUrl)
	}

	if app.ProviderError != nil && *app.ProviderError != "" {
		kvb.Add("Error", *app.ProviderError)
	}

	// Timestamps
	kvb.Add("Created", app.CreatedAt)
	kvb.Add("Updated", app.UpdatedAt)

	// Access control
	if app.AccessControl != nil {
		access := strings.ToLower(string(*app.AccessControl))
		kvb.Add("Access", access)

		// Show additional auth details based on access type
		if *app.AccessControl == api.AppAccessControlOrg && app.OrgId != nil {
			kvb.Add("OrgID", *app.OrgId)
		}

		if *app.AccessControl == api.AppAccessControlOrg && app.AllowedRoles != nil && len(*app.AllowedRoles) > 0 {
			kvb.Add("AllowedRoles", strings.Join(*app.AllowedRoles, ", "))
		}

		if *app.AccessControl == api.AppAccessControlCustom && app.CustomAuth != nil {
			kvb.Add("JWTIssuer", app.CustomAuth.Issuer)
			kvb.Add("JWTAudience", app.CustomAuth.Audience)
		}
	}

	return kvb.Write(dw)
}

// displayAppStatusJSON is no longer needed - using shared DataWriter
