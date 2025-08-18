package cmd

import (
	"context"
	"fmt"
	"strings"

	openapi_types "github.com/oapi-codegen/runtime/types"
	"github.com/spf13/cobra"

	"github.com/fastertools/ftl-cli/go/shared/api"
	"github.com/fastertools/ftl-cli/go/shared/auth"
)

func newListCmd() *cobra.Command {
	var format string
	var detailed bool

	cmd := &cobra.Command{
		Use:   "list",
		Short: "List all FTL applications",
		Long:  `List all FTL applications deployed on the platform.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := context.Background()
			return runList(ctx, format, detailed)
		},
	}

	cmd.Flags().StringVarP(&format, "output", "o", "table", "Output format (table, json)")
	cmd.Flags().BoolVarP(&detailed, "detailed", "d", false, "Show additional details (app ID, deployment info)")

	return cmd
}

// Allow overriding for tests
var runList = runListImpl

func runListImpl(ctx context.Context, format string, detailed bool) error {
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

	// List all apps - use empty params instead of nil
	// (some backends may filter differently with nil vs empty params)
	Debug("Calling ListApps with empty params")
	response, err := apiClient.ListApps(ctx, &api.ListAppsParams{})
	if err != nil {
		return fmt.Errorf("failed to list apps: %w", err)
	}

	Debug("ListApps returned %d apps", len(response.Apps))
	if len(response.Apps) == 0 {
		fmt.Fprintln(colorOutput, "No applications found.")
		return nil
	}

	// Use shared DataWriter for consistent output
	dw := NewDataWriter(colorOutput, format)

	switch format {
	case "json":
		return dw.WriteStruct(response.Apps)
	case "table":
		return displayAppsTable(response.Apps, detailed, dw)
	default:
		return fmt.Errorf("invalid output format: %s (use 'table' or 'json')", format)
	}
}

func displayAppsTable(apps []struct {
	AccessControl *api.ListAppsResponseBodyAppsAccessControl `json:"accessControl,omitempty"`
	AllowedRoles  *[]string                                  `json:"allowedRoles,omitempty"`
	AppId         openapi_types.UUID                         `json:"appId"`
	AppName       string                                     `json:"appName"`
	CreatedAt     string                                     `json:"createdAt"`
	CustomAuth    *struct {
		Audience string `json:"audience"`
		Issuer   string `json:"issuer"`
	} `json:"customAuth,omitempty"`
	LatestDeployment *struct {
		CreatedAt          *float32                                           `json:"createdAt,omitempty"`
		DeployedAt         *float32                                           `json:"deployedAt,omitempty"`
		DeploymentDuration *float32                                           `json:"deploymentDuration,omitempty"`
		DeploymentId       string                                             `json:"deploymentId"`
		Environment        *string                                            `json:"environment,omitempty"`
		Status             api.ListAppsResponseBodyAppsLatestDeploymentStatus `json:"status"`
		StatusMessage      *string                                            `json:"statusMessage,omitempty"`
	} `json:"latestDeployment"`
	OrgId         *string                            `json:"orgId,omitempty"`
	ProviderError *string                            `json:"providerError,omitempty"`
	ProviderUrl   *string                            `json:"providerUrl,omitempty"`
	Status        api.ListAppsResponseBodyAppsStatus `json:"status"`
	UpdatedAt     string                             `json:"updatedAt"`
}, detailed bool, dw *DataWriter) error {
	// Build headers
	var headers []string
	if detailed {
		headers = []string{"NAME", "ID", "STATUS", "ACCESS", "URL", "DEPLOYMENT", "ENVIRONMENT", "CREATED"}
	} else {
		headers = []string{"NAME", "STATUS", "ACCESS", "URL", "CREATED"}
	}

	// Build table
	tb := NewTableBuilder(headers...)

	for _, app := range apps {
		url := "-"
		if app.ProviderUrl != nil && *app.ProviderUrl != "" {
			url = *app.ProviderUrl
		}

		// Format access control
		access := "public"
		if app.AccessControl != nil {
			access = strings.ToLower(string(*app.AccessControl))
		}

		// If failed, show error in URL column
		if app.Status == api.ListAppsResponseBodyAppsStatusFAILED {
			if app.ProviderError != nil && *app.ProviderError != "" {
				// Truncate long errors for table display
				errMsg := *app.ProviderError
				if len(errMsg) > 40 {
					errMsg = errMsg[:37] + "..."
				}
				url = errMsg
			}
		}

		// Format timestamp (just date for list view)
		created := app.CreatedAt
		if len(created) > 10 {
			created = created[:10] // Just YYYY-MM-DD
		}

		// Add row based on detailed mode
		if detailed {
			// Get deployment info if available
			deploymentId := "-"
			environment := "-"
			if app.LatestDeployment != nil {
				deploymentId = app.LatestDeployment.DeploymentId
				if app.LatestDeployment.Environment != nil {
					environment = *app.LatestDeployment.Environment
				}
			}
			tb.AddRow(app.AppName, app.AppId.String(), string(app.Status), access, url, deploymentId, environment, created)
		} else {
			tb.AddRow(app.AppName, string(app.Status), access, url, created)
		}
	}

	// Write the table
	if err := tb.Write(dw); err != nil {
		return err
	}

	count := len(apps)
	plural := ""
	if count != 1 {
		plural = "s"
	}
	fmt.Fprintf(colorOutput, "Total: %d application%s\n", count, plural)

	return nil
}

// displayAppsJSON is no longer needed - using shared DataWriter
