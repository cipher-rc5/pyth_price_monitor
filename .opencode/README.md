# OpenCode Rules

This directory stores modular instruction files for large language agents.

These files are loaded through `opencode.json` using the `instructions` field,
as recommended by OpenCode rules documentation.

Rule files are ordered by precedence and concern area:

1. `00-core.md`
2. `10-rust-standards.md`
3. `20-testing-and-quality.md`
4. `30-security-and-dependencies.md`
5. `40-docs-and-observability.md`
