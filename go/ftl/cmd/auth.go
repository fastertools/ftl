package cmd

import (
	"context"
	"fmt"
	"time"

	"github.com/fastertools/ftl-cli/go/shared/auth"
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

	cmd := &cobra.Command{
		Use:   "login",
		Short: "Login to FTL platform",
		Long:  `Authenticate with FTL platform using OAuth device flow.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Create credential store
			store, err := auth.NewKeyringStore()
			if err != nil {
				return fmt.Errorf("failed to initialize credential store: %w", err)
			}

			// Create auth manager
			config := &auth.LoginConfig{
				NoBrowser:     noBrowser,
				Force:         force,
				AuthKitDomain: authKitDomain,
			}
			if authKitDomain == "" {
				config.AuthKitDomain = auth.DefaultAuthKitDomain
			}

			manager := auth.NewManager(store, config)

			// Print login header
			fmt.Printf("→ Logging in to FTL Engine (%s)\n\n", config.AuthKitDomain)

			// Perform login
			ctx, cancel := context.WithTimeout(context.Background(), auth.LoginTimeout)
			defer cancel()

			// Check if already logged in
			if !force {
				if status := manager.Status(); status.LoggedIn && !status.NeedsRefresh {
					color.Green("✓ Already logged in")
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
			fmt.Println()

			if status.Credentials != nil {
				creds := status.Credentials

				// Show domain
				fmt.Printf("AuthKit Domain: %s\n", color.CyanString(creds.AuthKitDomain))

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

				// Show actual token if requested
				if showToken {
					fmt.Println()
					fmt.Println("Access Token:")
					fmt.Println(creds.AccessToken)
				}
			}

			return nil
		},
	}

	cmd.Flags().BoolVar(&showToken, "show-token", false, "Display the actual access token")
	return cmd
}
