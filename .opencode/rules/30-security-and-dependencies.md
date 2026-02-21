# 30 Security and Dependencies

## Security Baseline

- Treat all external input (HTTP, WebSocket, env vars) as untrusted.
- Validate configuration values early and fail fast with actionable errors.
- Never log sensitive material (API keys, auth headers, private endpoints with secrets).
- Prefer least-privilege GitHub Actions permissions.

## Dependency Management

- Keep dependencies current and compatible with stable Rust.
- Use Dependabot for Cargo and GitHub Actions updates.
- Run `cargo audit` regularly via CI schedule.
- Avoid adding heavy dependencies when standard library or existing crates suffice.

## Supply Chain Practices

- Pin behavior through `Cargo.lock` in CI using `--locked` where appropriate.
- Review transitive impact for new networking or crypto dependencies.
- Prefer widely adopted crates with active maintenance.

## Secrets and Environment

- Keep `.env` ignored in git; only commit `.env.example`.
- Support placeholder-based API key interpolation without hardcoding secrets.
- Document all required secrets and safe rotation expectations.
