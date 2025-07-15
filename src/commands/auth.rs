use anyhow::Result;
use chrono::Utc;
use console::{Emoji, style};

use crate::commands::login;

static LOCK: Emoji<'_, '_> = Emoji("🔐", "");
static WARNING: Emoji<'_, '_> = Emoji("⚠️ ", "");
static CHECK: Emoji<'_, '_> = Emoji("✅", "");

pub async fn status() -> Result<()> {
    println!("{} Authentication Status", style("→").cyan());
    println!();

    match login::get_stored_credentials() {
        Ok(credentials) => {
            println!("{} {}", CHECK, style("Logged in").green().bold());
            println!();
            println!(
                "AuthKit Domain: {}",
                style(&credentials.authkit_domain).cyan()
            );

            if let Some(expires_at) = credentials.expires_at {
                let now = Utc::now();
                if expires_at < now {
                    println!("Access Token: {} {}", WARNING, style("Expired").yellow());
                } else {
                    let duration = expires_at - now;
                    let hours = duration.num_hours();
                    let minutes = duration.num_minutes() % 60;
                    println!(
                        "Access Token: Valid for {}h {}m",
                        style(hours).green(),
                        style(minutes).green()
                    );
                }
            } else {
                println!("Access Token: {}", style("Valid").green());
            }

            if credentials.refresh_token.is_some() {
                println!("Refresh Token: {}", style("Available").green());
            }

            Ok(())
        }
        Err(e) => {
            if e.to_string().contains("No matching entry found") {
                println!("{LOCK} Not logged in");
                println!();
                println!("Run {} to authenticate", style("ftl login").cyan().bold());
            } else {
                println!("{WARNING} Error checking authentication status");
                println!();
                println!(
                    "Run {} to re-authenticate",
                    style("ftl login").cyan().bold()
                );
            }
            Ok(())
        }
    }
}
