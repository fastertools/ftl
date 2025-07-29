# FTL Telemetry

Privacy-first telemetry infrastructure for the FTL CLI.

## Features

- **Local-only by default**: All telemetry data is stored locally in `~/.ftl/logs/<installation-id>/`
- **Privacy-first**: No PII collection, minimal data collection
- **Configurable**: Can be disabled via config file or environment variable
- **Transparent**: Users can inspect all collected data

## Configuration

Telemetry can be disabled in two ways:

1. Environment variable: `FTL_TELEMETRY_DISABLED=1`
2. Config file (`~/.ftl/config.toml`):
   ```toml
   [telemetry]
   enabled = false
   ```

## Data Collection

The telemetry system collects:

- Command usage (which commands are run)
- Command success/failure rates
- Performance metrics (execution time)
- Feature usage patterns

The telemetry system does NOT collect:

- Personal information
- Project names or paths
- File contents
- Command arguments that might contain sensitive data

## Storage

Telemetry data is stored as JSONL (JSON Lines) files in:
```
~/.ftl/logs/<installation-id>/YYYY-MM-DD.jsonl
```

Log files older than 30 days are automatically cleaned up.

## Future Features

- Opt-in remote telemetry for aggregated usage statistics
- Export functionality for data analysis
- Telemetry dashboard for viewing local data