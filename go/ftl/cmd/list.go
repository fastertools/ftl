package cmd

import (
	"context"
	"encoding/json"
	"fmt"
	"text/tabwriter"

	"github.com/fatih/color"
	"github.com/google/uuid"
	"github.com/spf13/cobra"

	"github.com/fastertools/ftl-cli/go/shared/api"
	"github.com/fastertools/ftl-cli/go/shared/auth"
)

func newListCmd() *cobra.Command {
	var format string

	cmd := &cobra.Command{
		Use:   "list",
		Short: "List all FTL applications",
		Long:  `List all FTL applications deployed on the platform.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := context.Background()
			return runList(ctx, format)
		},
	}

	cmd.Flags().StringVarP(&format, "output", "o", "table", "Output format (table, json)")

	return cmd
}

// Allow overriding for tests
var runList = runListImpl

func runListImpl(ctx context.Context, format string) error {
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

	// List all apps
	response, err := apiClient.ListApps(ctx, nil)
	if err != nil {
		return fmt.Errorf("failed to list apps: %w", err)
	}

	if len(response.Apps) == 0 {
		color.Yellow("No applications found.")
		return nil
	}

	switch format {
	case "json":
		return displayAppsJSON(response.Apps)
	case "table":
		return displayAppsTable(response.Apps)
	default:
		return fmt.Errorf("invalid output format: %s (use 'table' or 'json')", format)
	}
}

func displayAppsTable(apps []struct {
	AccessControl *api.ListAppsResponseAppsAccessControl `json:"accessControl,omitempty"`
	AllowedRoles  *[]string                               `json:"allowedRoles,omitempty"`
	AppId         uuid.UUID                               `json:"appId"`
	AppName       string                                  `json:"appName"`
	CreatedAt     string                                  `json:"createdAt"`
	CustomAuth    *struct {
		Audience string `json:"audience"`
		Issuer   string `json:"issuer"`
	} `json:"customAuth,omitempty"`
	OrgId         *string                                 `json:"orgId,omitempty"`
	ProviderError *string                                 `json:"providerError,omitempty"`
	ProviderUrl   *string                                 `json:"providerUrl,omitempty"`
	Status        api.ListAppsResponseAppsStatus         `json:"status"`
	UpdatedAt     string                                  `json:"updatedAt"`
}) error {
	fmt.Fprintln(colorOutput)
	
	// Create a tabwriter for aligned output
	w := tabwriter.NewWriter(colorOutput, 0, 0, 2, ' ', 0)
	
	// Print header
	fmt.Fprintf(w, "NAME\tID\tSTATUS\tURL\n")
	fmt.Fprintf(w, "----\t--\t------\t---\n")
	
	for _, app := range apps {
		url := "-"
		if app.ProviderUrl != nil && *app.ProviderUrl != "" {
			url = *app.ProviderUrl
		}
		
		statusColor := color.New(color.FgWhite)
		switch app.Status {
		case api.ACTIVE:
			statusColor = color.New(color.FgGreen)
		case api.FAILED:
			statusColor = color.New(color.FgRed)
		case api.PENDING, api.CREATING:
			statusColor = color.New(color.FgYellow)
		case api.DELETED, api.DELETING:
			statusColor = color.New(color.FgHiBlack)
		}
		
		status := statusColor.Sprint(app.Status)
		fmt.Fprintf(w, "%s\t%s\t%s\t%s\n", app.AppName, app.AppId.String(), status, url)
	}
	
	w.Flush()
	fmt.Fprintln(colorOutput)
	
	count := len(apps)
	plural := ""
	if count != 1 {
		plural = "s"
	}
	fmt.Fprintf(colorOutput, "Total: %d application%s\n", count, plural)
	
	return nil
}

func displayAppsJSON(apps []struct {
	AccessControl *api.ListAppsResponseAppsAccessControl `json:"accessControl,omitempty"`
	AllowedRoles  *[]string                               `json:"allowedRoles,omitempty"`
	AppId         uuid.UUID                               `json:"appId"`
	AppName       string                                  `json:"appName"`
	CreatedAt     string                                  `json:"createdAt"`
	CustomAuth    *struct {
		Audience string `json:"audience"`
		Issuer   string `json:"issuer"`
	} `json:"customAuth,omitempty"`
	OrgId         *string                                 `json:"orgId,omitempty"`
	ProviderError *string                                 `json:"providerError,omitempty"`
	ProviderUrl   *string                                 `json:"providerUrl,omitempty"`
	Status        api.ListAppsResponseAppsStatus         `json:"status"`
	UpdatedAt     string                                  `json:"updatedAt"`
}) error {
	encoder := json.NewEncoder(colorOutput)
	encoder.SetIndent("", "  ")
	return encoder.Encode(apps)
}