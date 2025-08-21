package cli

import (
	"context"
	"fmt"
	"time"

	"github.com/fastertools/ftl-cli/internal/api"
	"github.com/fastertools/ftl-cli/internal/auth"
	"github.com/fastertools/ftl-cli/internal/config"
	"github.com/fatih/color"
	"github.com/spf13/cobra"
)

func newAuthCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "auth",
		Short: "Manage authentication",
		Long:  `Manage authentication for FTL platform and registries.`,
	}

	// Add subcommands
	cmd.AddCommand(
		newAuthLoginCmd(),
		newAuthLogoutCmd(),
		newAuthStatusCmd(),
	)

	return cmd
}

func newAuthLoginCmd() *cobra.Command {
	var noBrowser bool
	var force bool
	var authKitDomain string
	var machine bool
	var machineToken string

	cmd := &cobra.Command{
		Use:   "login",
		Short: "Login to FTL platform",
		Long: `Authenticate with FTL platform using OAuth device flow or machine credentials.

For user authentication, the standard OAuth device flow is used.

For machine authentication (CI/CD pipelines), use one of these methods:
  1. Set environment variables: FTL_CLIENT_ID and FTL_CLIENT_SECRET
  2. Pass a pre-generated token with --token flag`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Create credential store
			store, err := auth.NewKeyringStore()
			if err != nil {
				return fmt.Errorf("failed to initialize credential store: %w", err)
			}

			// Create auth manager
			loginConfig := &auth.LoginConfig{
				NoBrowser:     noBrowser,
				Force:         force,
				AuthKitDomain: authKitDomain,
			}
			if authKitDomain == "" {
				loginConfig.AuthKitDomain = auth.DefaultAuthKitDomain
			}

			manager := auth.NewManager(store, loginConfig)

			// Handle machine authentication
			if machine {
				fmt.Println("→ Logging in as machine (M2M authentication)")
				fmt.Println()

				ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
				defer cancel()

				// If token provided directly, use it
				if machineToken != "" {
					if err := manager.LoginMachineWithToken(ctx, machineToken); err != nil {
						return fmt.Errorf("failed to login with token: %w", err)
					}
					color.Green("✅ Successfully logged in as machine with provided token")
					return nil
				}

				// Otherwise use client credentials flow
				if err := manager.LoginMachine(ctx); err != nil {
					return fmt.Errorf("machine login failed: %w", err)
				}

				color.Green("✅ Successfully logged in as machine")
				fmt.Println()
				fmt.Println("Note: Machine tokens are typically short-lived.")
				fmt.Println("For CI/CD, set FTL_CLIENT_ID and FTL_CLIENT_SECRET environment variables.")
				return nil
			}

			// Print login header
			fmt.Println("→ Logging in to FTL Engine")
			fmt.Println()

			// Perform login
			ctx, cancel := context.WithTimeout(context.Background(), auth.LoginTimeout)
			defer cancel()

			// Check if already logged in
			if !force {
				if status := manager.Status(); status.LoggedIn && !status.NeedsRefresh {
					// Show full auth status
					color.Green("✅ Already logged in")

					// Show user info
					if cfg, err := config.Load(); err == nil {
						if user := cfg.GetCurrentUser(); user != nil && user.Email != "" {
							fmt.Printf("   Logged in as: %s\n", color.CyanString(user.Email))
						}
					}

					// Show token validity
					if status.Credentials != nil && status.Credentials.ExpiresAt != nil {
						duration := time.Until(*status.Credentials.ExpiresAt)
						if duration > 0 {
							fmt.Printf("   Access token valid for %dh %dm\n",
								int(duration.Hours()),
								int(duration.Minutes())%60)
						}
					}

					fmt.Println()
					fmt.Printf("Use %s to force re-authentication\n", color.CyanString("ftl auth login --force"))
					return nil
				}
			}

			// Start device flow
			deviceAuth, err := manager.StartDeviceFlow(ctx)
			if err != nil {
				return fmt.Errorf("failed to start authentication: %w", err)
			}

			// Display instructions
			fmt.Println("🌐 To complete login, visit:")
			color.Cyan("   %s", deviceAuth.VerificationURIComplete)
			fmt.Println()
			fmt.Println("📋 Or manually enter code:")
			color.Yellow("   %s", deviceAuth.UserCode)
			fmt.Println()

			if !noBrowser {
				fmt.Println("🚀 Opening browser...")
			}

			// Complete login
			creds, err := manager.CompleteDeviceFlow(ctx, deviceAuth)
			if err != nil {
				return fmt.Errorf("login failed: %w", err)
			}

			// Success
			fmt.Println()
			color.Green("✅ Successfully logged in!")

			// Try to fetch and display user info
			if apiClient, err := api.NewFTLClient(manager, ""); err == nil {
				if userInfo, err := apiClient.GetUserInfo(ctx); err == nil && userInfo.User.Email != nil {
					fmt.Printf("   Logged in as: %s\n", color.CyanString(*userInfo.User.Email))

					// Save user info and refresh org list to config
					if userConfig, err := config.Load(); err == nil {
						// Save user info
						userCfg := &config.UserInfo{
							UserID:    userInfo.User.Id,
							Email:     *userInfo.User.Email,
							UpdatedAt: time.Now().Format(time.RFC3339),
						}
						if userInfo.User.Name != nil {
							userCfg.Username = *userInfo.User.Name
						}
						_ = userConfig.SetCurrentUser(userCfg)

						// Clear and refresh organization list
						// This ensures we have the correct orgs for the new user
						userConfig.Organizations = make(map[string]config.OrgInfo)
						for _, org := range userInfo.Organizations {
							orgInfo := config.OrgInfo{
								ID:   org.Id,
								Name: org.Name,
							}
							_ = userConfig.AddOrganization(orgInfo)
						}

						// Clear current org selection since it may not be valid for new user
						_ = userConfig.SetCurrentOrg("")
					}
				}
			}

			if creds.ExpiresAt != nil {
				duration := time.Until(*creds.ExpiresAt)
				fmt.Printf("   Access token valid for %dh %dm\n",
					int(duration.Hours()),
					int(duration.Minutes())%60)
			}

			return nil
		},
	}

	cmd.Flags().BoolVar(&noBrowser, "no-browser", false, "Don't open browser automatically")
	cmd.Flags().BoolVar(&force, "force", false, "Force re-authentication even if already logged in")
	cmd.Flags().StringVar(&authKitDomain, "auth-domain", "", "Override AuthKit domain (for testing)")
	cmd.Flags().BoolVar(&machine, "machine", false, "Login as machine using M2M authentication")
	cmd.Flags().StringVar(&machineToken, "token", "", "Pre-generated M2M token (use with --machine)")

	return cmd
}

func newAuthLogoutCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "logout",
		Short: "Logout from FTL platform",
		Long:  `Remove stored authentication credentials.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Create credential store
			store, err := auth.NewKeyringStore()
			if err != nil {
				return fmt.Errorf("failed to initialize credential store: %w", err)
			}

			// Create auth manager
			manager := auth.NewManager(store, nil)

			// Perform logout
			if err := manager.Logout(); err != nil {
				// Check if not logged in
				if err.Error() == "not logged in" {
					color.Yellow("⚠️  Not logged in")
					return nil
				}
				return fmt.Errorf("logout failed: %w", err)
			}

			// Clear user info and org list from config
			if cfg, err := config.Load(); err == nil {
				_ = cfg.ClearCurrentUser()
				_ = cfg.SetCurrentOrg("")
				// Clear all cached org info
				cfg.Organizations = make(map[string]config.OrgInfo)
				_ = cfg.Save()
			}

			color.Green("✅ Successfully logged out")
			return nil
		},
	}
}

func newAuthStatusCmd() *cobra.Command {
	var showToken bool

	cmd := &cobra.Command{
		Use:   "status",
		Short: "Show authentication status",
		Long:  `Display current authentication status and token information.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Create credential store
			store, err := auth.NewKeyringStore()
			if err != nil {
				return fmt.Errorf("failed to initialize credential store: %w", err)
			}

			// Create auth manager
			manager := auth.NewManager(store, nil)

			// Get status
			status := manager.Status()

			// If --show-token is used, just output the token and nothing else
			if showToken {
				if !status.LoggedIn || status.Credentials == nil {
					return fmt.Errorf("not logged in")
				}
				fmt.Print(status.Credentials.AccessToken)
				return nil
			}

			// Display status
			color.Cyan("→ Authentication Status\n")
			fmt.Println()

			if !status.LoggedIn {
				fmt.Println("🔐 Not logged in")
				fmt.Println()
				fmt.Printf("Run %s to authenticate\n", color.CyanString("ftl auth login"))
				return nil
			}

			// Show logged in status
			color.Green("✅ Logged in")

			// Check actor type and show user info
			actorType, _ := manager.GetActorType(context.Background())
			if actorType == "machine" {
				fmt.Printf(" (as %s)\n", color.CyanString("machine"))
			} else {
				// Load user info from config
				if cfg, err := config.Load(); err == nil {
					if user := cfg.GetCurrentUser(); user != nil {
						if user.Email != "" {
							fmt.Printf(" as %s\n", color.CyanString(user.Email))
						} else if user.Username != "" {
							fmt.Printf(" as %s\n", color.CyanString(user.Username))
						} else {
							fmt.Printf(" (as %s)\n", color.CyanString("user"))
						}
					} else {
						fmt.Printf(" (as %s)\n", color.CyanString("user"))
					}
				} else {
					fmt.Printf(" (as %s)\n", color.CyanString("user"))
				}
			}
			fmt.Println()

			if status.Credentials != nil {
				creds := status.Credentials

				// Show token status
				if creds.ExpiresAt != nil {
					if creds.IsExpired() {
						color.Yellow("Access Token: ⚠️  Expired")
						if creds.RefreshToken != "" {
							fmt.Println("              (will auto-refresh on next use)")
						}
					} else {
						duration := time.Until(*creds.ExpiresAt)
						hours := int(duration.Hours())
						minutes := int(duration.Minutes()) % 60

						fmt.Printf("Access Token: Valid for %s\n",
							color.GreenString("%dh %dm", hours, minutes))
					}
				} else {
					color.Green("Access Token: Valid")
				}

				// Show refresh token status
				if creds.RefreshToken != "" {
					color.Green("Refresh Token: Available")
				}
			}

			return nil
		},
	}

	cmd.Flags().BoolVar(&showToken, "show-token", false, "Output only the access token (for use in scripts)")
	return cmd
}
