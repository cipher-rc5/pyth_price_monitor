# Pyth Price Monitor

Real-time Bitcoin/USD price monitoring using Pyth Network's Hermes API, built with Rust, Alloy, and dotenvy.

## Features

- Real-time BTC/USD price streaming via Pyth Hermes SSE endpoint
- Latest price fetch with confidence intervals
- Exponentially-weighted moving average (EMA) prices
- Ethereum integration using Alloy for blockchain interactions
- Optional Ethereum WebSocket RPC path via fastwebsockets for low-latency calls
- Environment-based configuration using dotenvy
- Structured logging with tracing

## Prerequisites

- Rust 1.70+
- Internet connection for Hermes API access

## Quick Start

1. Clone and navigate to the project:

```bash
cd pyth-price-monitor
```

2. Copy the environment template:

```bash
cp .env.example .env
```

3. Configure your `.env` file:

```env
HERMES_ENDPOINT=https://hermes.pyth.network
BTC_USD_PRICE_FEED_ID=0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43
ETH_RPC_URL=https://eth.llamarpc.com
ETH_RPC_WS_URL=wss://eth.llamarpc.com
OUTPUT_TIMEZONE=EST
LOG_LEVEL=info
```

4. Run:

```bash
cargo run
```

## Documentation

All detailed documentation lives in [`docs/`](docs/):

- [docs/USAGE.md](docs/USAGE.md) - configuration, API reference, patterns, deployment
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - module overview, data flow, Pyth price details
- [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) - development workflow and standards
- [docs/SECURITY.md](docs/SECURITY.md) - vulnerability reporting and hardening checklist

## License

MIT
