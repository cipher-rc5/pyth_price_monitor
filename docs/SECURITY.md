# Security Policy

## Supported Versions

This project tracks the latest stable Rust toolchain and accepts security fixes on the default branch.

## Reporting a Vulnerability

Please do not open public issues for suspected vulnerabilities.

Instead, report security concerns privately with:

- A clear description of the issue
- Steps to reproduce
- Impact assessment
- Any proof-of-concept details

If you are coordinating responsible disclosure, include your preferred timeline.

## Security Practices in This Repository

- Dependency advisories are checked with `cargo audit` in CI.
- Static analysis is performed with GitHub CodeQL.
- `.env` and secret-like files are gitignored.
- API key placeholders are supported so secrets are not hardcoded.

## Hardening Checklist for Deployments

- Use dedicated RPC API keys per environment.
- Rotate credentials periodically and after suspected exposure.
- Keep `Cargo.lock` and dependencies updated.
- Run CI checks (lint/test/audit) before release.
