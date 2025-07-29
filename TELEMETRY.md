CLI Telemetry Policy
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
| cli_version | The version of the CLI being used (e.g., from your-cli --version). | "0.2.1" |
| command | The specific CLI command that was executed (e.g., deploy, build). | "build" |
| os_arch | The operating system and architecture (e.g., from std::env::consts::OS and ARCH). | "linux-x86_64" |
| duration_ms | The time the command took to execute, in milliseconds. | 850 |
| status | Whether the command succeeded or failed. | "success" |
| error_code | If the command failed, a sanitized, high-level error code. We never collect full error messages or stack traces. | "invalid_manifest" |
| is_interactive | Whether the command was run in an interactive terminal or a script (e.g., CI/CD). | true |

What We DO NOT Collect
We are committed to user privacy and never collect:

Personally Identifiable Information (PII): No names, email addresses, usernames, etc.

Sensitive Information: No IP addresses, hostnames, or MAC addresses.

Command Arguments or Flags: These could contain sensitive data like file paths, secrets, or other private information.

Environment Variables: Your shell environment is your own.

Contents of your files or code.

The Anonymous installation_id
When you run the CLI for the first time, it generates a standard Version 4 UUID and saves it to a local configuration file at ~/.your-platform/config.json. This ID is completely random and contains no information about you or your machine. It allows us to distinguish between an error affecting 100 different users and an error affecting a single user 100 times, which is critical for prioritization.

How to Control Telemetry
We believe in giving you full control.

First-Run Notice
The very first time you run a command, we will display a one-time notice informing you that we collect telemetry and how to disable it. The command will execute successfully without requiring any input from you.

Disabling Telemetry
You can opt-out of telemetry at any time in two ways:

Via the CLI:

# Disable telemetry collection
your-cli telemetry disable


You can re-enable it at any time with your-cli telemetry enable.

Via an Environment Variable:
For non-interactive environments like CI/CD, you can set an environment variable to disable telemetry for a single command execution:

YOUR_PLATFORM_TELEMETRY_OPT_OUT=1 your-cli deploy


Where the Data Goes
Telemetry data is sent via HTTPS to a public endpoint (https://telemetry.your-platform.com/v1/event) where it is processed and stored in our secure analytics database. This data helps our engineering team understand usage patterns and improve the product.

We welcome you to inspect the source code for our telemetry collection in the cli/src/telemetry.rs module.