# Program Usage

## Overview

`pyth-price-monitor` monitors Pyth price feeds via Hermes, logs structured snapshots/updates, and optionally queries Ethereum RPC endpoints over HTTP or WebSocket.

## Run Commands

### Main binary

```bash
cargo run
```

### Optimized run

```bash
cargo run --release
```

### Example binaries

```bash
cargo run --example multi_feed_alerts
cargo run --example alloy_integration
```

## Command Flags

The main binary currently does **not** expose CLI flags (for example, no `--help`/`--config` parser yet).

Runtime behavior is configured through environment variables in `.env`.

## Environment Configuration

### Required

- `BTC_USD_PRICE_FEED_ID`: Pyth BTC/USD feed id

### Optional

- `HERMES_ENDPOINT`: Hermes base URL (default: `https://hermes.pyth.network`)
- `ETH_RPC_URL`: EVM HTTP RPC URL
- `ETH_RPC_WS_URL`: EVM WebSocket RPC URL
- `ETH_RPC_API_KEY`: API key replacement value for URL placeholders
- `OUTPUT_TIMEZONE`: log timestamp timezone (`UTC`, `EST`, `CST`, `MST`, `PST`, `UTC+HH:MM`, `UTC-HH:MM`)

### API-key URL placeholders

Both HTTP and WS RPC URLs support:

- `{$API_KEY}`
- `${API_KEY}`

Example:

```env
ETH_RPC_API_KEY=your_blink_key
ETH_RPC_URL=https://eth.blinklabs.xyz/v1/{$API_KEY}
ETH_RPC_WS_URL=wss://eth.blinklabs.xyz/ws/v1/{$API_KEY}
```

## Structured Output Fields

Price snapshot/update logs include fields intended for ingestion pipelines:

- `event`
- `feed_id`
- `price`
- `confidence`
- `ema_price`
- `publish_time_unix`
- `publish_time_local`
- `timezone`

These fields are designed for easy storage/processing in log backends and databases.

## Dependency Summary

Direct dependencies and roles:

- `alloy`: Ethereum provider and RPC integration
- `dotenvy`: `.env` loading
- `tokio`: async runtime
- `serde`, `serde_json`: payload serialization/deserialization
- `reqwest`: Hermes REST/SSE HTTP client
- `fastwebsockets`: low-latency WebSocket path for RPC
- `hyper`, `http-body-util`: WS handshake/request plumbing
- `tokio-rustls`, `webpki-roots`: TLS for `wss://`
- `futures`: stream combinators
- `anyhow`: error handling
- `tracing`, `tracing-subscriber`: structured logging
- `hex`, `chrono`: utility/time formatting

## Build Profiles

Configured in `Cargo.toml`:

- `profile.dev`: `opt-level = 1`
- `profile.release`: `opt-level = 3`, `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"`
