package cli

import (
	"context"
	"fmt"
	"os"
	"strings"
	"text/tabwriter"
	"time"

	"github.com/AlecAivazis/survey/v2"
	"github.com/spf13/cobra"

	"github.com/fastertools/ftl-cli/internal/api"
	"github.com/fastertools/ftl-cli/internal/auth"
	"github.com/fastertools/ftl-cli/internal/config"
)

// newOrgCmd creates the org command group
func newOrgCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "org",
		Short: "Manage organization context",
		Long: `Manage organization context for deployments.
		
The org commands allow you to list available organizations, set the current
organization context, and view the currently selected organization.`,
	}

	cmd.AddCommand(
		newOrgListCmd(),
		newOrgSetCmd(),
		newOrgCurrentCmd(),
	)

	return cmd
}

// newOrgListCmd creates the 'org list' command
func newOrgListCmd() *cobra.Command {
	var refresh bool

	cmd := &cobra.Command{
		Use:     "list",
		Short:   "List available organizations",
		Long:    "List all organizations you have access to, showing the current selection.",
		Aliases: []string{"ls"},
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := context.Background()
			return runOrgList(ctx, refresh)
		},
	}

	cmd.Flags().BoolVar(&refresh, "refresh", false, "Refresh organization list from server")

	return cmd
}

// newOrgSetCmd creates the 'org set' command
func newOrgSetCmd() *cobra.Command {
	var interactive bool

	cmd := &cobra.Command{
		Use:   "set [ORG_ID]",
		Short: "Set the current organization",
		Long: `Set the current organization context for future deployments.
		
If no ORG_ID is provided, you'll be prompted to select from available organizations.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := context.Background()

			var orgID string
			if len(args) > 0 {
				orgID = args[0]
			}

			return runOrgSet(ctx, orgID, interactive)
		},
	}

	cmd.Flags().BoolVarP(&interactive, "interactive", "i", false, "Force interactive selection")

	return cmd
}

// newOrgCurrentCmd creates the 'org current' command
func newOrgCurrentCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "current",
		Short:   "Show the current organization",
		Long:    "Display the currently selected organization context.",
		Aliases: []string{"show"},
		RunE: func(cmd *cobra.Command, args []string) error {
			return runOrgCurrent()
		},
	}

	return cmd
}

// runOrgList lists available organizations
func runOrgList(ctx context.Context, refresh bool) error {
	// Load config
	cfg, err := config.Load()
	if err != nil {
		return fmt.Errorf("failed to load config: %w", err)
	}

	// Get available orgs from API if refresh requested or no cached orgs
	var orgs []string
	needsRefresh := refresh || len(cfg.ListOrganizations()) == 0

	if needsRefresh {
		// Initialize auth
		store, err := auth.NewKeyringStore()
		if err != nil {
			return fmt.Errorf("failed to initialize credential store: %w", err)
		}
		authManager := auth.NewManager(store, nil)

		// Check authentication
		if _, err := authManager.GetToken(ctx); err != nil {
			return fmt.Errorf("authentication required (run 'ftl auth login'): %w", err)
		}

		// Create FTL client
		client, err := api.NewFTLClient(authManager, "")
		if err != nil {
			return fmt.Errorf("failed to create API client: %w", err)
		}

		// Get user info and organizations
		userInfo, err := client.GetUserInfo(ctx)
		if err != nil {
			return fmt.Errorf("failed to get user info: %w", err)
		}

		if userInfo == nil || len(userInfo.Organizations) == 0 {
			Warn("No organizations found")
			return nil
		}

		// Update config with user info
		userCfg := &config.UserInfo{
			UserID:    userInfo.User.Id,
			UpdatedAt: time.Now().Format(time.RFC3339),
		}
		// Handle nullable fields
		if userInfo.User.Email != nil {
			userCfg.Email = *userInfo.User.Email
		}
		if userInfo.User.Name != nil {
			userCfg.Username = *userInfo.User.Name
		}
		if err := cfg.SetCurrentUser(userCfg); err != nil {
			// Non-fatal
			fmt.Printf("Warning: failed to save user info: %v\n", err)
		}

		// Update config with org info
		for _, org := range userInfo.Organizations {
			orgInfo := config.OrgInfo{
				ID:       org.Id,
				Name:     org.Name,
				LastUsed: time.Now().Format(time.RFC3339),
			}
			_ = cfg.AddOrganization(orgInfo)
			orgs = append(orgs, org.Id)
		}
	} else {
		// Use cached orgs
		for _, org := range cfg.ListOrganizations() {
			orgs = append(orgs, org.ID)
		}
	}

	if len(orgs) == 0 {
		Warn("No organizations available")
		Info("Run 'ftl org list --refresh' to check for new organizations")
		return nil
	}

	// Display organizations
	currentOrg := cfg.GetCurrentOrg()

	// Use tabwriter for aligned output
	w := tabwriter.NewWriter(os.Stdout, 0, 0, 2, ' ', 0)
	fmt.Fprintln(w, "CURRENT\tORG ID\tNAME\tLAST USED")
	fmt.Fprintln(w, "-------\t------\t----\t---------")

	for _, orgID := range orgs {
		current := " "
		if orgID == currentOrg {
			current = "*"
		}

		orgInfo, exists := cfg.GetOrganization(orgID)
		name := "-"
		lastUsed := "-"

		if exists {
			if orgInfo.Name != "" {
				name = orgInfo.Name
			}
			if orgInfo.LastUsed != "" {
				if t, err := time.Parse(time.RFC3339, orgInfo.LastUsed); err == nil {
					lastUsed = t.Format("2006-01-02 15:04")
				}
			}
		}

		fmt.Fprintf(w, "%s\t%s\t%s\t%s\n", current, orgID, name, lastUsed)
	}

	w.Flush()

	if currentOrg == "" {
		fmt.Println()
		Info("No organization currently selected. Use 'ftl org set' to select one.")
	}

	return nil
}

// runOrgSet sets the current organization
func runOrgSet(ctx context.Context, orgID string, forceInteractive bool) error {
	// Load config
	cfg, err := config.Load()
	if err != nil {
		return fmt.Errorf("failed to load config: %w", err)
	}

	// Initialize auth
	store, err := auth.NewKeyringStore()
	if err != nil {
		return fmt.Errorf("failed to initialize credential store: %w", err)
	}
	authManager := auth.NewManager(store, nil)

	// Check authentication
	if _, err := authManager.GetToken(ctx); err != nil {
		return fmt.Errorf("authentication required (run 'ftl auth login'): %w", err)
	}

	// Create FTL client
	client, err := api.NewFTLClient(authManager, "")
	if err != nil {
		return fmt.Errorf("failed to create API client: %w", err)
	}

	// Get user info and organizations
	userInfo, err := client.GetUserInfo(ctx)
	if err != nil {
		return fmt.Errorf("failed to get user info: %w", err)
	}

	if userInfo == nil || len(userInfo.Organizations) == 0 {
		return fmt.Errorf("no organizations available")
	}

	// Build list of available orgs
	availableOrgs := make(map[string]string) // ID -> Name
	var orgIDs []string

	for _, org := range userInfo.Organizations {
		availableOrgs[org.Id] = org.Name
		orgIDs = append(orgIDs, org.Id)

		// Update cached org info
		orgInfo := config.OrgInfo{
			ID:   org.Id,
			Name: org.Name,
		}
		_ = cfg.AddOrganization(orgInfo)
	}

	// If interactive mode or no org specified, prompt for selection
	if forceInteractive || orgID == "" {
		if !isInteractive() {
			return fmt.Errorf("organization ID required in non-interactive mode")
		}

		// Build options with names
		options := make([]string, len(orgIDs))
		for i, id := range orgIDs {
			name := availableOrgs[id]
			if name != "" {
				options[i] = fmt.Sprintf("%s (%s)", id, name)
			} else {
				options[i] = id
			}
		}

		var selected string
		prompt := &survey.Select{
			Message: "Select organization:",
			Options: options,
		}

		if err := survey.AskOne(prompt, &selected); err != nil {
			return err
		}

		// Extract org ID from selection
		orgID = strings.Split(selected, " ")[0]
	}

	// Validate org ID
	if _, exists := availableOrgs[orgID]; !exists {
		return fmt.Errorf("organization '%s' not found in your available organizations", orgID)
	}

	// Update config
	if err := cfg.SetCurrentOrg(orgID); err != nil {
		return fmt.Errorf("failed to save configuration: %w", err)
	}

	// Update last used time
	if orgInfo, exists := cfg.GetOrganization(orgID); exists {
		orgInfo.LastUsed = time.Now().Format(time.RFC3339)
		_ = cfg.AddOrganization(orgInfo)
	}

	name := availableOrgs[orgID]
	if name != "" {
		Success("Current organization set to: %s (%s)", orgID, name)
	} else {
		Success("Current organization set to: %s", orgID)
	}

	return nil
}

// runOrgCurrent shows the current organization
func runOrgCurrent() error {
	// Load config
	cfg, err := config.Load()
	if err != nil {
		return fmt.Errorf("failed to load config: %w", err)
	}

	currentOrg := cfg.GetCurrentOrg()
	if currentOrg == "" {
		Warn("No organization currently selected")
		Info("Use 'ftl org set' to select an organization")
		return nil
	}

	// Display current org with any cached metadata
	if orgInfo, exists := cfg.GetOrganization(currentOrg); exists && orgInfo.Name != "" {
		fmt.Printf("%s (%s)\n", currentOrg, orgInfo.Name)
	} else {
		fmt.Println(currentOrg)
	}

	return nil
}
