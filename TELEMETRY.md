# FTL CLI Telemetry Policy

We believe in building our platform in the open, and that transparency is key to earning and keeping the trust of our community. This document outlines the telemetry we collect from our Command Line Interface (CLI), why we collect it, and how you can control it.

Our Guiding Principles
Transparency & Verifiability: Our telemetry collection is part of our open-source code. You can review the implementation to verify our claims. This document will always be an accurate representation of what we collect.

User-Centric Improvement: We only collect data that helps us make the CLI better for you. The data answers questions like: "Which features are most used?", "Where are the performance bottlenecks?", and "What are the most common errors?".

Anonymity by Default: We do not collect any Personally Identifiable Information (PII). The data is aggregated to understand trends and is not used to track individual users. You are not required to be logged in to our platform to use the CLI, and the telemetry system reflects that.

What We Collect
For each command execution, we may collect the following anonymous information:

| Data Point | Description | Example |
| event_version | The schema version of this telemetry event. | "1.0" |
| installation_id | A randomly generated UUID created on first run to anonymously count unique CLI installations. See below for more details. | "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d" |
| session_id | A randomly generated UUID for each CLI invocation to group related events. | "f1e2d3c4-b5a6-4789-abcd-ef0123456789" |
| ftl_version | The version of the FTL CLI being used. | "0.0.36" |
| command | The specific CLI command that was executed (e.g., deploy, build). | "build" |
| os | The operating system (e.g., macos, linux, windows). | "macos" |
| arch | The CPU architecture (e.g., x86_64, aarch64). | "aarch64" |
| duration_ms | The time the command took to execute, in milliseconds. | 850 |
| event_type | The type of event (command_executed, command_success, command_error). | "command_success" |
| error | If the command failed, a sanitized error message with PII removed. | "Failed to compile: syntax error" |
| args | Command arguments with sensitive values filtered (tokens, passwords, etc. are redacted). | ["build", "--token", "[REDACTED]"] |

What We DO NOT Collect
We are committed to user privacy and never collect:

Personally Identifiable Information (PII): No names, email addresses, usernames, etc.

Sensitive Information: No IP addresses, hostnames, or MAC addresses.

Sensitive Command Arguments: Passwords, tokens, API keys, and other secrets are automatically filtered and replaced with [REDACTED].

Environment Variables: Your shell environment is your own.

Contents of your files or code.

The Anonymous installation_id
When you run the FTL CLI for the first time, it generates a standard Version 4 UUID and saves it to a local configuration file at ~/.ftl/config.toml. This ID is completely random and contains no information about you or your machine. It allows us to distinguish between an error affecting 100 different users and an error affecting a single user 100 times, which is critical for prioritization.

How to Control Telemetry
We believe in giving you full control.

First-Run Notice
The very first time you run a command, we will display a one-time notice informing you that we collect telemetry and how to disable it. The command will execute successfully without requiring any input from you.

Disabling Telemetry
You can opt-out of telemetry at any time in two ways:

Via the CLI:

# Disable telemetry collection
ftl telemetry disable

# Check telemetry status
ftl telemetry status

# Re-enable telemetry
ftl telemetry enable

Via an Environment Variable:
For non-interactive environments like CI/CD, you can set an environment variable to disable telemetry for a single command execution:

FTL_TELEMETRY_DISABLED=1 ftl deploy


Where the Data Goes
Telemetry data is collected to help us improve the FTL platform. The data helps us understand usage patterns and prioritize features.

Telemetry is enabled by default to help us understand usage patterns and improve FTL for everyone. You can disable it at any time using the methods described above.

We welcome you to inspect the source code for our telemetry collection in the `crates/telemetry/` directory.

## Privacy Features

1. **Automatic PII Sanitization**: Error messages are automatically sanitized to remove:
   - File paths containing user directories
   - URLs that might contain credentials
   - Email addresses
   - IP addresses

2. **Anonymous Collection**: All data is anonymized before collection

3. **Transparent Implementation**: The telemetry code is open source and can be inspected

4. **Full Control**: Enable or disable telemetry at any time, with environment variable overrides for CI/CD environments