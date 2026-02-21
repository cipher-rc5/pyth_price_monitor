# Contributing

Thanks for contributing to `pyth-price-monitor`.

## Prerequisites

- Rust stable toolchain
- `cargo` available on PATH

## Setup

```bash
cp .env.example .env
```

Update `.env` with the required feed id and optional RPC settings.

## Development Workflow

1. Create a feature branch.
2. Implement focused changes.
3. Run the full local quality gate:

```bash
cargo fmt --all
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

4. Update docs when behavior or configuration changes:

- `README.md` (project overview and quick-start)
- `docs/` (this directory - all detailed documentation)
- `.env.example`
- rustdoc for public APIs

## Code Standards

- Follow idiomatic Rust patterns and explicit error context.
- Keep structured log fields stable for downstream ingestion.
- Avoid dead code and warning suppressions unless justified.
- Never commit secrets, keys, or `.env` values.

## Pull Request Guidelines

- Use a clear title and summary.
- Explain the motivation and operational impact.
- Include relevant test/verification output.
- Keep PR scope tight and reviewable.
