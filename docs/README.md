# Pyth Price Monitor Documentation

This directory contains comprehensive documentation for running, configuring, and extending `pyth-price-monitor`.

## Contents

- [Program Usage](./USAGE.md)
- [Architecture](./ARCHITECTURE.md)

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
