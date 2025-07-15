use anyhow::Result;
use console::{Emoji, style};

use crate::commands::login;

static CHECK: Emoji<'_, '_> = Emoji("✅", "");

pub async fn execute() -> Result<()> {
    println!("{} Logging out of FTL", style("→").cyan());
    println!();

    match login::clear_stored_credentials() {
        Ok(_) => {
            println!(
                "{} {} Successfully logged out!",
                CHECK,
                style("Success!").green().bold()
            );
            Ok(())
        }
        Err(e) => {
            if e.to_string().contains("No matching entry found") {
                println!("Not currently logged in.");
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}
