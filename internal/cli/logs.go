package cli

import (
	"context"
	"fmt"
	"net/http"
	"strings"

	"github.com/fatih/color"
	"github.com/google/uuid"
	openapi_types "github.com/oapi-codegen/runtime/types"
	"github.com/spf13/cobra"

	"github.com/fastertools/ftl/internal/api"
	"github.com/fastertools/ftl/internal/auth"
)

// LogsOptions holds configuration for the logs command.
// All options are validated server-side to ensure consistency.
type LogsOptions struct {
	// AppID can be either a UUID or an application name.
	// When a name is provided, it will be resolved to a UUID.
	AppID string

	// Since specifies the time range for logs.
	// Supported formats:
	// - Relative: "30m", "1h", "7d"
	// - RFC3339: "2024-01-15T10:00:00Z"
	// - Unix timestamp: "1705315200"
	Since string

	// Tail limits the number of log lines returned.
	// Valid range: 1-1000, default: 100
	Tail string
}

func newLogsCmd() *cobra.Command {
	opts := &LogsOptions{}

	cmd := &cobra.Command{
		Use:   "logs [app-name-or-id] [flags]",
		Short: "View application logs",
		Long: `View logs for an FTL application deployed to FTL Engine.

This command retrieves recent logs from your deployed application.

Examples:
  # Get logs for an app by name (interactive selection if multiple matches)
  ftl logs my-app

  # Get logs for a specific app ID
  ftl logs 123e4567-e89b-12d3-a456-426614174000

  # Get logs from the last hour
  ftl logs my-app --since 1h

  # Get the last 500 lines
  ftl logs my-app --tail 500

  # Get logs from the last 30 minutes, showing only last 50 lines
  ftl logs my-app --since 30m --tail 50`,
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := context.Background()

			// Get app identifier from args if provided
			if len(args) > 0 {
				opts.AppID = args[0]
			}

			return runLogs(ctx, opts)
		},
	}

	cmd.Flags().StringVar(&opts.Since, "since", "7d", "Time range for logs (e.g., '30m', '1h', '7d', or RFC3339/Unix timestamp)")
	cmd.Flags().StringVar(&opts.Tail, "tail", "100", "Number of log lines from the end (1-1000)")

	return cmd
}

// runLogs executes the logs retrieval logic with comprehensive error handling
func runLogs(ctx context.Context, opts *LogsOptions) error {
	// Validate options before making any API calls
	if err := validateLogsOptions(opts); err != nil {
		return err
	}

	// Get auth manager
	store, err := auth.NewKeyringStore()
	if err != nil {
		// Provide helpful context for keyring failures
		return fmt.Errorf("failed to access credential store: %w\n"+
			"If you're experiencing keyring issues, try:\n"+
			"  - On Linux: Install gnome-keyring or similar\n"+
			"  - On macOS: Ensure Keychain Access is working\n"+
			"  - On Windows: Check Windows Credential Manager", err)
	}

	authManager := auth.NewManager(store, nil)
	token, err := authManager.GetOrRefreshToken(ctx)
	if err != nil {
		return fmt.Errorf("authentication required. Please run 'ftl auth login' first")
	}

	// Create API client with auth
	client, err := api.NewClientWithResponses(
		api.DefaultAPIBaseURL,
		api.WithRequestEditorFn(func(ctx context.Context, req *http.Request) error {
			req.Header.Set("Authorization", "Bearer "+token)
			return nil
		}),
	)
	if err != nil {
		return fmt.Errorf("failed to create API client: %w", err)
	}

	// If no app ID provided, list apps for selection
	if opts.AppID == "" {
		return fmt.Errorf("app name or ID is required")
	}

	// Check if the provided value might be a name instead of ID
	var appID string
	if !isUUID(opts.AppID) {
		// Try to find app by name
		resp, err := client.ListAppsWithResponse(ctx, &api.ListAppsParams{
			Name: &opts.AppID,
		})
		if err != nil {
			return fmt.Errorf("failed to list apps: %w", err)
		}

		if resp.StatusCode() != 200 {
			return fmt.Errorf("failed to list apps: status %d", resp.StatusCode())
		}

		if resp.JSON200 == nil {
			return fmt.Errorf("empty response from server")
		}

		// Filter for exact name match
		var foundAppID openapi_types.UUID
		found := false
		for _, app := range resp.JSON200.Apps {
			if strings.EqualFold(app.AppName, opts.AppID) {
				foundAppID = app.AppId
				found = true
				break
			}
		}

		if !found {
			return fmt.Errorf("no app found with name '%s'", opts.AppID)
		}

		// Convert UUID to string
		appID = foundAppID.String()
	} else {
		appID = opts.AppID
	}

	// Parse UUID
	appUUID, err := uuid.Parse(appID)
	if err != nil {
		return fmt.Errorf("invalid app ID: %w", err)
	}

	// Get logs
	Info("Fetching logs for app %s...", appID)

	params := &api.GetAppLogsParams{
		Since: &opts.Since,
		Tail:  &opts.Tail,
	}

	resp, err := client.GetAppLogsWithResponse(ctx, openapi_types.UUID(appUUID), params)
	if err != nil {
		return fmt.Errorf("failed to get logs: %w", err)
	}

	if resp.StatusCode() != 200 {
		if resp.JSON401 != nil && resp.JSON401.Message != "" {
			return fmt.Errorf("unauthorized: %s", resp.JSON401.Message)
		}
		if resp.JSON404 != nil && resp.JSON404.Message != "" {
			return fmt.Errorf("app not found: %s", resp.JSON404.Message)
		}
		if resp.JSON500 != nil && resp.JSON500.Message != "" {
			return fmt.Errorf("server error: %s", resp.JSON500.Message)
		}
		return fmt.Errorf("failed to get logs: status %d", resp.StatusCode())
	}

	if resp.JSON200 == nil {
		return fmt.Errorf("empty response from server")
	}

	logsResp := resp.JSON200

	// Display logs
	if logsResp.Logs == "" {
		Info("No logs found for the specified time range")
		return nil
	}

	// Print metadata using color package directly like deploy.go does
	fmt.Println()
	color.Cyan("▶ Logs for app %s (last %.0f lines from %s)", appID, logsResp.Metadata.Tail, logsResp.Metadata.Since)
	fmt.Println(strings.Repeat("─", 80))

	// Print the logs
	fmt.Println(logsResp.Logs)

	return nil
}

// validateLogsOptions performs client-side validation of logs command options
func validateLogsOptions(opts *LogsOptions) error {
	// Validate app ID is provided
	if opts.AppID == "" {
		return fmt.Errorf("app name or ID is required")
	}

	// Validate tail parameter if provided
	if opts.Tail != "" {
		// Check if it's a valid number
		var tailNum int
		if _, err := fmt.Sscanf(opts.Tail, "%d", &tailNum); err != nil {
			return fmt.Errorf("--tail must be a number, got: %s", opts.Tail)
		}

		// Check range (1-1000)
		if tailNum < 1 || tailNum > 1000 {
			return fmt.Errorf("--tail must be between 1 and 1000, got: %d", tailNum)
		}
	}

	// Since parameter validation is done server-side as it's more complex
	// (relative times, RFC3339, Unix timestamps)

	return nil
}

// isUUID checks if a string looks like a UUID
func isUUID(s string) bool {
	// Simple check for UUID format (8-4-4-4-12 hex characters)
	if len(s) != 36 {
		return false
	}

	// Check for hyphens in the right places
	if s[8] != '-' || s[13] != '-' || s[18] != '-' || s[23] != '-' {
		return false
	}

	// Check that all other characters are hex
	for i, c := range s {
		if i == 8 || i == 13 || i == 18 || i == 23 {
			continue
		}
		// Character must be a valid hex digit (0-9, a-f, A-F)
		if (c < '0' || c > '9') && (c < 'a' || c > 'f') && (c < 'A' || c > 'F') {
			return false
		}
	}

	return true
}
