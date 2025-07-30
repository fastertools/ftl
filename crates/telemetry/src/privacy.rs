//! Privacy utilities for telemetry

use regex::Regex;

/// Sanitize error messages to remove potentially sensitive information
pub fn sanitize_error_message(error: &str) -> String {
    // Remove URLs first (before file paths) to avoid conflicts
    let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
    let sanitized = url_regex.replace_all(error, "[URL_REDACTED]");
    
    // Remove file paths that might contain user information
    let path_regex = Regex::new(r"(/[^/\s]+)+/([^/\s]+)").unwrap();
    let sanitized = path_regex.replace_all(&sanitized, ".../[REDACTED]/$2");
    
    // Remove email addresses
    let email_regex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
    let sanitized = email_regex.replace_all(&sanitized, "[EMAIL_REDACTED]");
    
    // Remove IP addresses
    let ip_regex = Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b").unwrap();
    let sanitized = ip_regex.replace_all(&sanitized, "[IP_REDACTED]");
    
    // Truncate to reasonable length
    if sanitized.len() > 200 {
        format!("{}...", &sanitized[..200])
    } else {
        sanitized.to_string()
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
        assert!(sanitized.contains(".../[REDACTED]/file.txt"));
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
    fn test_truncate_long_errors() {
        let long_error = "a".repeat(300);
        let sanitized = sanitize_error_message(&long_error);
        assert_eq!(sanitized.len(), 203); // 200 + "..."
    }
}