# Security Policy

## Our Commitment to Security

FTL Core is designed to be used in performance-critical AI agent environments where security is paramount. We take security vulnerabilities seriously and appreciate the security community's help in keeping FTL Core safe for everyone.

## Reporting a Vulnerability

### What to Report

Please report any security vulnerability that could affect FTL Core users, including:

- Code execution vulnerabilities in WASM components
- Input validation bypasses in MCP tools
- Authentication or authorization flaws in the MCP protocol implementation
- Memory safety issues (though Rust's memory safety helps prevent many of these)
- Cryptographic vulnerabilities in tools or the core library
- Supply chain vulnerabilities in dependencies
- Build system security issues that could compromise deployments

### How to Report

**For security vulnerabilities, please DO NOT open a public GitHub issue.**

Instead, please report security vulnerabilities through one of these private channels:

1. **GitHub Security Advisories** (Preferred)
   - Go to the [FTL Core Security tab](https://github.com/fastertools/core/security)
   - Click "Report a vulnerability"
   - Fill out the vulnerability report form

2. **Direct Email** (If GitHub Security Advisories unavailable)
   - Email: [mashh.labs@gmail.com]
   - Subject: "FTL Core Security Vulnerability Report"

### What to Include

When reporting a vulnerability, please include:

- **Description** of the vulnerability and its potential impact
- **Steps to reproduce** the issue (if applicable)
- **Proof of concept** code or examples (if safe to share)
- **Affected versions** or components
- **Potential fix suggestions** (if you have them)
- **Your contact information** for follow-up questions

### What to Expect

- **Initial Response**: We'll acknowledge receipt within 24-48 hours
- **Assessment**: We'll assess the vulnerability within 3-5 business days
- **Updates**: We'll provide regular updates on our progress
- **Resolution**: We'll work to address critical vulnerabilities as quickly as possible

## Security Best Practices for FTL Core

### For Users

When using FTL Core tools:

- Validate inputs from untrusted sources before passing to FTL Core tools
- Run tools in isolated environments when processing sensitive data
- Keep FTL Core tools updated to the latest versions
- Monitor for security advisories and apply updates promptly
- Use HTTPS when deploying MCP servers in production
- Implement proper authentication for production deployments

### For Contributors

When contributing to FTL Core:

- Follow secure coding practices outlined in our [Contributing Guide](CONTRIBUTING.md)
- Validate all inputs thoroughly in tool implementations
- Use the `thiserror` crate for proper error handling
- Avoid `unsafe` code unless absolutely necessary (and document why)
- Run security checks with `cargo audit` before submitting PRs
- Review dependencies for known vulnerabilities

### For Maintainers

Our security maintenance practices:

- Regular dependency audits using `cargo audit`
- Automated security scanning in CI/CD pipelines
- Security-focused code reviews for all contributions
- Prompt security updates for critical vulnerabilities
- Clear security communication to users about vulnerabilities and fixes

## Security Tools and Checks

### OpenSSF Security Baseline Compliance

FTL Core follows the OpenSSF Security Baseline recommendations:

#### Automated Security Checks

```bash
# Run comprehensive security suite
just security-full

# Individual security commands:
just security-check    # cargo-deny dependency checks
just audit-supply-chain # cargo-vet supply chain auditing
just generate-sbom     # Generate Software Bill of Materials
```

#### CI/CD Security Pipeline

Our CI/CD pipeline automatically:
- **Dependency Auditing**: `cargo-audit` for known vulnerabilities
- **Supply Chain Security**: `cargo-vet` for dependency auditing
- **SAST**: Semgrep static analysis for security issues
- **License Compliance**: Comprehensive license checking with cargo-deny
- **SBOM Generation**: Software Bill of Materials for all releases
- **SLSA Level 2**: Build provenance attestations and reproducible builds
- **Security Scanning**: Automated vulnerability scanning with multiple tools

#### Supply Chain Security

We implement comprehensive supply chain security:
- **SBOM Generation**: Every release includes JSON/XML Software Bill of Materials
- **Build Attestations**: SLSA Level 2 compliant build provenance
- **Dependency Auditing**: cargo-vet with trusted audit sources (Google, Mozilla, Embark)
- **Reproducible Builds**: SOURCE_DATE_EPOCH for consistent build outputs
- **Secure Registries**: Only trusted crate registries allowed
- **License Enforcement**: Strict license compliance with deny lists for GPL/AGPL

### Manual Security Testing

For security-sensitive contributions, consider:

- **Fuzzing input validation** with tools like `cargo-fuzz`
- **Static analysis** with `cargo-clippy` security lints
- **Dependency review** for new crates added to the project
- **WASM security analysis** for WebAssembly-specific vulnerabilities

## Known Security Considerations

### WebAssembly Deployment

- **Sandboxing**: WASM provides natural sandboxing, but be aware of host function security
- **Resource limits**: WASM environments may have resource constraints
- **Side-channel attacks**: Consider timing attacks in cryptographic operations

### MCP Protocol

- **Input validation**: All MCP tool inputs should be validated thoroughly
- **Error handling**: Don't leak sensitive information in error messages
- **Rate limiting**: Consider implementing rate limiting in production deployments

### Cryptographic Tools

- **Use well-tested libraries**: Prefer established cryptographic libraries
- **Avoid custom crypto**: Don't implement custom cryptographic primitives
- **Secure randomness**: Use cryptographically secure random number generators
- **Timing attacks**: Be aware of timing-based side-channel attacks

## Security Disclosure Timeline

Our typical vulnerability disclosure process:

1. **Day 0**: Vulnerability reported privately
2. **Day 1-2**: Initial response and assessment
3. **Day 3-7**: Investigation and fix development
4. **Day 7-14**: Testing and validation of fixes
5. **Day 14+**: Coordinated public disclosure with security advisory

**Note**: Timeline may vary based on vulnerability severity and complexity.

## Security Acknowledgments

We believe in recognizing security researchers who help make FTL Core safer:

- **Hall of Fame**: Security researchers who report valid vulnerabilities
- **CVE Credits**: Proper attribution in CVE records
- **Public Thanks**: Recognition in security advisories and release notes

### Responsible Disclosure

We ask security researchers to:

- **Allow reasonable time** for fixes before public disclosure
- **Avoid accessing or modifying** user data during testing
- **Report vulnerabilities in good faith** without malicious intent
- **Respect user privacy** and system integrity during research

## Contact Information

For security-related questions or concerns:

- **Security Reports**: Use GitHub Security Advisories (preferred)
- **General Security Questions**: Open a GitHub Discussion
- **Urgent Security Issues**: [mashh.labs@gmail.com]

## Policy Updates

This security policy may be updated periodically to reflect:

- Changes in supported versions
- Updates to our security processes
- New security tools or practices
- Community feedback and suggestions

Last updated: Fri Jun 27 2025

---

Security is everyone's responsibility. Thank you for helping keep FTL Core secure.
