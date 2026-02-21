# 20 Testing and Quality

## Required Validation

Every non-trivial change must be validated with:

- `cargo fmt --all`
- `cargo check --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-targets`

## Test Expectations

- Add focused unit tests for:
  - config parsing and env interpolation
  - timezone parsing/formatting
  - data transformation logic
- Keep tests deterministic and independent from external network services.
- Prefer table-driven tests for parser behavior and edge cases.

## Warning and Dead Code Policy

- Zero warnings policy in CI and local checks.
- Remove dead code rather than silencing with lint allows unless justified.
- If a lint must be allowed, scope it narrowly and document why.
