# 00 Core Rules

## Mission

Maintain a reliable, secure, and idiomatic Rust codebase for real-time market data and RPC integrations.

## General Principles

- Prefer small, focused changes over broad refactors unless explicitly requested.
- Preserve backward compatibility of existing environment variables unless migration is requested.
- Do not introduce breaking configuration behavior without updating docs and `.env.example`.
- Avoid hidden side effects and global mutable state.
- Keep runtime output ingestion-friendly and schema-stable.

## Workflow Requirements

- Before finishing code changes, run:
  - `cargo fmt`
  - `cargo check --all-targets`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test --all-targets`
- If any command fails, fix root causes rather than suppressing warnings.
- Never commit secrets, credentials, keys, or `.env` values.

## Change Hygiene

- Keep `README.md`, `docs/`, and `.env.example` aligned with runtime behavior.
- When new configuration is added, document required/optional semantics.
- Use structured logging fields for machine parsing whenever possible.
