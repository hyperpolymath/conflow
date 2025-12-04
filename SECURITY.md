# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue,
please report it responsibly.

### How to Report

1. **Do NOT** open a public issue for security vulnerabilities
2. Email security concerns to: `security@conflow.dev` (or create a confidential issue)
3. Include as much detail as possible:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### What to Expect

- **Acknowledgment**: Within 48 hours of your report
- **Initial Assessment**: Within 7 days
- **Resolution Timeline**: Depends on severity
  - Critical: 24-48 hours
  - High: 7 days
  - Medium: 30 days
  - Low: 90 days

### Disclosure Policy

- We follow coordinated disclosure
- We will credit reporters (unless anonymity is requested)
- We aim to fix vulnerabilities before public disclosure

## Security Measures

### Build Security

- All releases are built with `cargo build --release`
- Dependencies are audited using `cargo audit`
- Binary stripping enabled to reduce attack surface

### Supply Chain Security

- Dependencies are pinned via `Cargo.lock`
- Minimal dependency footprint
- No runtime network access required (offline-first design)

### Code Security

- Written in Rust for memory safety
- No `unsafe` blocks in core functionality
- Input validation on all user-provided data
- Path traversal protection in file operations

## Security-Related Configuration

### Safe Defaults

conflow is designed with security-conscious defaults:

- No automatic code execution without explicit pipeline definition
- Cache is local-only (no network sync)
- No telemetry or data collection
- Sandboxed execution where possible

### Permissions

conflow requires:
- Read access to configuration files
- Write access to output directories and cache
- Execute access for CUE and Nickel binaries

## Known Limitations

- Pipeline definitions can execute arbitrary shell commands via the `shell` tool type
- Users should review `.conflow.yaml` files from untrusted sources before running

## Security Contacts

- Primary: security@conflow.dev
- GitLab Issues: Use confidential issue feature
- PGP Key: Available upon request

## Acknowledgments

We thank all security researchers who responsibly disclose vulnerabilities.
