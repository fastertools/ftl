//! Privacy utilities for telemetry

use once_cell::sync::Lazy;
use regex::Regex;

// Compile regexes once at startup
static URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"https?://[^\s]+").expect("Invalid URL regex"));

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").expect("Invalid email regex")
});

static IPV4_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b").expect("Invalid IPv4 regex"));

static IPV6_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?:[0-9a-fA-F]{0,4}:){2,7}[0-9a-fA-F]{0,4}\b").expect("Invalid IPv6 regex")
});

// Sensitive command line arguments that should be filtered
const SENSITIVE_ARGS: &[&str] = &[
    "--token",
    "--password",
    "--key",
    "--secret",
    "--api-key",
    "--auth",
    "--credential",
    "--private",
    "--cert",
    "--certificate",
    "--passphrase",
    "-t",
    "-p",
    "-k", // Common short forms
];

/// Filter sensitive command arguments
pub fn filter_command_args(args: Vec<String>) -> Vec<String> {
    let mut filtered = Vec::new();
    let mut skip_next = false;

    for arg in args.iter() {
        if skip_next {
            filtered.push("[REDACTED]".to_string());
            skip_next = false;
            continue;
        }

        // Check if this is a sensitive argument
        let is_sensitive = SENSITIVE_ARGS
            .iter()
            .any(|&sensitive| arg == sensitive || arg.starts_with(&format!("{}=", sensitive)));

        if is_sensitive {
            if arg.contains('=') {
                // Format: --token=value
                let parts: Vec<&str> = arg.splitn(2, '=').collect();
                filtered.push(format!("{}=[REDACTED]", parts[0]));
            } else {
                // Format: --token value
                filtered.push(arg.clone());
                skip_next = true;
            }
        } else {
            // Also check for URLs, paths, emails in regular arguments
            let sanitized = sanitize_value(arg);
            filtered.push(sanitized);
        }
    }

    filtered
}

/// Sanitize a single value (used for command args and error messages)
fn sanitize_value(value: &str) -> String {
    let mut result = value.to_string();

    // Apply all sanitization patterns
    result = URL_REGEX.replace_all(&result, "[URL_REDACTED]").to_string();
    result = EMAIL_REGEX
        .replace_all(&result, "[EMAIL_REDACTED]")
        .to_string();
    result = IPV4_REGEX.replace_all(&result, "[IP_REDACTED]").to_string();
    result = IPV6_REGEX
        .replace_all(&result, "[IPV6_REDACTED]")
        .to_string();

    // Handle file paths - replace directory parts but keep filename
    if result.contains('/') {
        // Split by whitespace to handle paths in sentences
        let parts: Vec<&str> = result.split_whitespace().collect();
        let mut new_parts = Vec::new();

        for part in parts {
            if part.starts_with('/') && part.contains('/') {
                // This looks like a Unix path
                if let Some(last_slash) = part.rfind('/') {
                    let filename = &part[last_slash..];
                    new_parts.push(format!("[REDACTED]{}", filename));
                } else {
                    new_parts.push(part.to_string());
                }
            } else if part.contains(":\\") {
                // Windows path
                if let Some(last_sep) = part.rfind('\\') {
                    let filename = &part[last_sep..];
                    new_parts.push(format!("[REDACTED]{}", filename));
                } else {
                    new_parts.push(part.to_string());
                }
            } else {
                new_parts.push(part.to_string());
            }
        }

        result = new_parts.join(" ");
    }

    result
}

/// Sanitize error messages to remove potentially sensitive information
pub fn sanitize_error_message(error: &str) -> String {
    let sanitized = sanitize_value(error);

    // Truncate to reasonable length
    if sanitized.len() > 200 {
        format!("{}...", &sanitized[..200])
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_file_paths() {
        let error = "Failed to open /Users/johndoe/secret/file.txt";
        let sanitized = sanitize_error_message(error);
        assert!(!sanitized.contains("johndoe"));
        assert!(!sanitized.contains("/Users/"));
        assert!(sanitized.contains("[REDACTED]/file.txt"));
    }

    #[test]
    fn test_sanitize_urls() {
        let error = "Failed to connect to https://user:pass@example.com/api";
        let sanitized = sanitize_error_message(error);
        assert!(!sanitized.contains("user:pass"));
        assert!(sanitized.contains("[URL_REDACTED]"));
    }

    #[test]
    fn test_sanitize_emails() {
        let error = "Invalid email: john.doe@example.com";
        let sanitized = sanitize_error_message(error);
        assert!(!sanitized.contains("john.doe@example.com"));
        assert!(sanitized.contains("[EMAIL_REDACTED]"));
    }

    #[test]
    fn test_sanitize_ip_addresses() {
        let error = "Connection refused to 192.168.1.100";
        let sanitized = sanitize_error_message(error);
        assert!(!sanitized.contains("192.168.1.100"));
        assert!(sanitized.contains("[IP_REDACTED]"));
    }

    #[test]
    fn test_sanitize_ipv6_addresses() {
        let error = "Connection refused to 2001:db8::1";
        let sanitized = sanitize_error_message(error);
        assert!(!sanitized.contains("2001:db8::1"));
        assert!(sanitized.contains("[IPV6_REDACTED]"));
    }

    #[test]
    fn test_truncate_long_errors() {
        let long_error = "a".repeat(300);
        let sanitized = sanitize_error_message(&long_error);
        assert_eq!(sanitized.len(), 203); // 200 + "..."
    }

    #[test]
    fn test_filter_command_args_token() {
        let args = vec![
            "build".to_string(),
            "--token".to_string(),
            "secret123".to_string(),
            "--verbose".to_string(),
        ];
        let filtered = filter_command_args(args);
        assert_eq!(
            filtered,
            vec!["build", "--token", "[REDACTED]", "--verbose"]
        );
    }

    #[test]
    fn test_filter_command_args_equals_format() {
        let args = vec![
            "deploy".to_string(),
            "--password=secret456".to_string(),
            "--region=us-west".to_string(),
        ];
        let filtered = filter_command_args(args);
        assert_eq!(
            filtered,
            vec!["deploy", "--password=[REDACTED]", "--region=us-west"]
        );
    }

    #[test]
    fn test_filter_command_args_mixed_sensitive() {
        let args = vec![
            "login".to_string(),
            "-t".to_string(),
            "token123".to_string(),
            "--api-key=key456".to_string(),
            "https://api.example.com".to_string(),
        ];
        let filtered = filter_command_args(args);
        assert_eq!(
            filtered,
            vec![
                "login",
                "-t",
                "[REDACTED]",
                "--api-key=[REDACTED]",
                "[URL_REDACTED]"
            ]
        );
    }

    #[test]
    fn test_filter_command_args_paths() {
        let args = vec![
            "read".to_string(),
            "/home/user/secret/file.txt".to_string(),
            "output.log".to_string(),
        ];
        let filtered = filter_command_args(args);
        assert_eq!(filtered[1], "[REDACTED]/file.txt");
        assert_eq!(filtered[2], "output.log"); // Relative path not filtered
    }
}
