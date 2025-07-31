# FTL Telemetry

Privacy-first telemetry infrastructure for the FTL CLI.

## Overview

This crate implements anonymous usage telemetry for FTL CLI to help improve the tool and understand usage patterns.

## Features

- **Privacy by design**: No PII collection, automatic sanitization of sensitive data
- **User control**: Easy opt-out via config or environment variable
- **Transparent**: Open source implementation
- **First-run notice**: Users are informed about telemetry on first use

## Architecture

```
telemetry/
├── config.rs       # Telemetry configuration and settings
├── events.rs       # Event types and builders
├── logger.rs       # Telemetry logging implementation
├── notice.rs       # First-run notice system
└── privacy.rs      # Privacy utilities for sanitizing data and filtering arguments
```

## Usage

### For CLI Integration

```rust
use ftl_telemetry::{TelemetryClient, events::TelemetryEvent};

// Initialize telemetry (shows first-run notice if needed)
let telemetry = TelemetryClient::initialize()?;

// Log command execution
let event = TelemetryEvent::command_executed(
    "build",
    vec!["--release"],
    session_id,
);
telemetry.log_event(event).await?;

// Log command completion
let event = TelemetryEvent::command_success("build", duration_ms, session_id);
telemetry.log_event(event).await?;
```

### Configuration

Telemetry configuration is stored in `~/.ftl/config.toml`:

```toml
[telemetry]
enabled = true
installation_id = "550e8400-e29b-41d4-a716-446655440000"
upload_enabled = false  # Reserved for future use
retention_days = 30
```

### Environment Variables

- `FTL_TELEMETRY_DISABLED=1` - Disables telemetry regardless of config
- `CI=true` - Disables interactive first-run notice in CI environments

## Privacy

The telemetry system implements several privacy protections:

1. **Command argument filtering** to automatically redact:
   - Passwords, tokens, and API keys
   - URLs, email addresses, and IP addresses (including IPv6)
   - Sensitive file paths

2. **Automatic sanitization** of error messages to remove:
   - File paths containing user directories
   - URLs that might contain credentials  
   - Email addresses
   - IP addresses (both IPv4 and IPv6)

3. **Minimal data collection** - only essential usage metrics

4. **User control** - easy opt-out mechanisms

See [PRIVACY_AUDIT.md](./PRIVACY_AUDIT.md) for a detailed privacy analysis.

## Data Format

Telemetry events are stored in JSONL format (one JSON object per line):

```json
{"event_type":"command_executed","timestamp":"2025-07-17T10:00:00Z","session_id":"...","command":"build","args":["--release"],"ftl_version":"0.0.36","os":"macos","arch":"aarch64"}
{"event_type":"command_success","timestamp":"2025-07-17T10:00:05Z","session_id":"...","command":"build","duration_ms":5000}
```

## Future Enhancements

- [ ] Aggregate local statistics for user insights
- [ ] Optional crash reporting (with explicit opt-in)
- [ ] Argument filtering for sensitive flags
- [ ] Local analytics dashboard