# Pyth Price Monitor - Documentation

This directory contains all detailed documentation for `pyth-price-monitor`.

For a project overview and quick-start, see the root [README.md](../README.md).

## Contents

| File | Description |
|------|-------------|
| [USAGE.md](./USAGE.md) | Configuration reference, run commands, API reference, common patterns, error handling, performance tips, troubleshooting, and production deployment |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | Module overview, data flow diagram, module interfaces, Pyth/Hermes internals, and production considerations |
| [CONTRIBUTING.md](./CONTRIBUTING.md) | Development workflow, quality gate commands, code standards, and PR guidelines |
| [SECURITY.md](./SECURITY.md) | Vulnerability reporting process, security practices, and deployment hardening checklist |

## Quick Start

1. Create your runtime env file:

```bash
cp .env.example .env
```

2. Build and run:

```bash
cargo run --release
```

3. Optional example binaries:

```bash
cargo run --example multi_feed_alerts
cargo run --example alloy_integration
```
