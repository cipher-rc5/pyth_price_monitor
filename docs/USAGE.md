# Program Usage

## Overview

`pyth-price-monitor` monitors Pyth price feeds via Hermes, logs structured snapshots/updates, and optionally queries Ethereum RPC endpoints over HTTP or WebSocket.

## Quick Start

1. Copy the environment template:

```bash
cp .env.example .env
```

2. Run the basic monitor:

```bash
cargo run
```

3. Run advanced example binaries:

```bash
# Multi-feed with alerts
cargo run --example multi_feed_alerts

# Alloy integration example
cargo run --example alloy_integration
```

4. Build for production:

```bash
cargo build --release
./target/release/pyth-price-monitor
```

## Command Flags

The main binary currently does not expose CLI flags (no `--help`/`--config` parser). Runtime behavior is configured entirely through environment variables in `.env`.

## Environment Configuration

### Required

- `BTC_USD_PRICE_FEED_ID`: Pyth BTC/USD feed id

### Optional

- `HERMES_ENDPOINT`: Hermes base URL (default: `https://hermes.pyth.network`)
- `ETH_RPC_URL`: EVM HTTP RPC URL
- `ETH_RPC_WS_URL`: EVM WebSocket RPC URL
- `ETH_RPC_API_KEY`: API key replacement value for URL placeholders
- `OUTPUT_TIMEZONE`: log timestamp timezone (`UTC`, `EST`, `CST`, `MST`, `PST`, `UTC+HH:MM`, `UTC-HH:MM`)
- `LOG_LEVEL`: logging level (default: `info`)

### API-key URL placeholders

Both HTTP and WS RPC URLs support inline key placeholders:

- `{$API_KEY}`
- `${API_KEY}`

Example:

```env
ETH_RPC_API_KEY=your_blink_key
ETH_RPC_URL=https://eth.blinklabs.xyz/v1/{$API_KEY}
ETH_RPC_WS_URL=wss://eth.blinklabs.xyz/ws/v1/{$API_KEY}
```

### Adding multiple price feeds

```env
HERMES_ENDPOINT=https://hermes.pyth.network
BTC_USD_PRICE_FEED_ID=0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43
ETH_USD_PRICE_FEED_ID=0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace
SOL_USD_PRICE_FEED_ID=0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d
ETH_RPC_URL=https://eth.llamarpc.com
ETH_RPC_WS_URL=wss://eth.llamarpc.com
OUTPUT_TIMEZONE=EST
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

These fields are designed for stable storage and processing in log backends and databases. Do not rename them without coordinating downstream consumers.

## API Reference

### HermesClient

```rust
use pyth_price_monitor::hermes_client::HermesClient;

// Create client
let client = HermesClient::new("https://hermes.pyth.network");

// Fetch latest prices
let price_feeds = vec!["0xe62df6...5b43".to_string()];
let feeds = client.get_latest_price_updates(&price_feeds).await?;

// Stream real-time updates
let mut stream = client.stream_price_updates(&price_feeds).await?;
```

### PriceMonitor

```rust
use pyth_price_monitor::price_monitor::PriceMonitor;

// Create monitor
let monitor = PriceMonitor::new(
    "https://hermes.pyth.network".to_string(),
    vec!["0xe62df6...5b43".to_string()],
    pyth_price_monitor::config::OutputTimezone::parse("EST")?,
);

// Fetch once
let prices = monitor.fetch_latest_once().await?;

// Start streaming
monitor.start_streaming().await?;
```

### Types

```rust
use pyth_price_monitor::types::{PriceFeed, ParsedPriceFeed};

// Parse raw feed
let parsed: ParsedPriceFeed = price_feed.parse()?;

// Access price data
println!("Price: ${:.2}", parsed.price.price);
println!("Confidence: +/-${:.2}", parsed.price.conf);
println!("EMA: ${:.2}", parsed.ema_price.price);
```

## Common Patterns

### Price monitoring with alerts

```rust
struct PriceTracker {
    threshold: f64,
    last_price: Option<f64>,
}

impl PriceTracker {
    fn check(&mut self, price: f64) -> bool {
        if let Some(last) = self.last_price {
            let change = ((price - last) / last).abs() * 100.0;
            if change >= self.threshold {
                self.last_price = Some(price);
                return true;
            }
        } else {
            self.last_price = Some(price);
        }
        false
    }
}
```

### Storing price history

```rust
use std::collections::VecDeque;

struct PriceHistory {
    prices: VecDeque<(i64, f64)>,
    max_size: usize,
}

impl PriceHistory {
    fn new(max_size: usize) -> Self {
        Self {
            prices: VecDeque::new(),
            max_size,
        }
    }

    fn add(&mut self, timestamp: i64, price: f64) {
        if self.prices.len() >= self.max_size {
            self.prices.pop_front();
        }
        self.prices.push_back((timestamp, price));
    }

    fn average(&self) -> Option<f64> {
        if self.prices.is_empty() {
            return None;
        }
        let sum: f64 = self.prices.iter().map(|(_, p)| p).sum();
        Some(sum / self.prices.len() as f64)
    }
}
```

### Reconnection with exponential backoff

```rust
use tokio::time::{sleep, Duration};

async fn stream_with_reconnect(
    client: &HermesClient,
    price_feeds: &[String],
) -> Result<()> {
    let mut retry_count = 0;
    let max_retries = 5;

    loop {
        match client.stream_price_updates(price_feeds).await {
            Ok(mut stream) => {
                retry_count = 0;

                use futures::StreamExt;
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(feeds) => {
                            // Process feeds
                        }
                        Err(e) => {
                            tracing::error!("Stream error: {}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                retry_count += 1;
                if retry_count > max_retries {
                    return Err(e);
                }

                let wait_time = 2u64.pow(retry_count);
                tracing::warn!("Reconnecting in {} seconds...", wait_time);
                sleep(Duration::from_secs(wait_time)).await;
            }
        }
    }
}
```

### On-chain price updates with Alloy

```rust
use alloy::{
    primitives::U256,
    providers::{Provider, ProviderBuilder},
    sol,
};

sol! {
    #[sol(rpc)]
    contract PythPriceConsumer {
        function updatePriceFeeds(bytes[] calldata updateData)
            external payable;
    }
}

async fn update_on_chain_price(
    price: f64,
    contract_address: Address,
    rpc_url: &str,
) -> Result<()> {
    let provider = ProviderBuilder::new().connect(rpc_url).await?;

    // In production, fetch binary update data from Hermes
    // and submit to the Pyth contract.

    // Example: scale price for on-chain storage
    let price_scaled = (price * 100.0) as u64;
    let price_u256 = U256::from(price_scaled);

    Ok(())
}
```

## Error Handling

### Network errors

```rust
use anyhow::{Context, Result};

async fn fetch_with_retry(client: &HermesClient) -> Result<Vec<PriceFeed>> {
    let price_feeds = vec!["0xe62df6...5b43".to_string()];

    client
        .get_latest_price_updates(&price_feeds)
        .await
        .context("Failed to fetch price updates from Hermes")
}
```

### Stream errors

```rust
use futures::StreamExt;

async fn handle_stream_errors(mut stream: impl Stream<Item = Result<Vec<PriceFeed>>>) {
    while let Some(result) = stream.next().await {
        match result {
            Ok(feeds) => {
                // Process feeds
            }
            Err(e) => {
                tracing::error!("Stream error: {}", e);
                // Implement reconnection logic
                break;
            }
        }
    }
}
```

## Performance Tips

1. **Batch requests**: fetch multiple price feeds in one request
2. **Connection pooling**: reuse the `HermesClient` instance
3. **Buffer size**: adjust tokio runtime buffer sizes for high throughput
4. **Logging**: use `LOG_LEVEL=warn` or `error` in latency-sensitive production paths

## Troubleshooting

### Connection failures

```bash
# Check network connectivity
curl "https://hermes.pyth.network/v2/updates/price/latest?ids[]=0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43"

# Verify price feed ID at:
# https://pyth.network/developers/price-feed-ids
```

### Rate limiting

If the public endpoint returns 429s or drops connections:

- Switch to a private Hermes endpoint (e.g., Triton One)
- Implement request throttling
- Cache recent prices between fetches

### Stream disconnections

The SSE stream auto-closes after 24 hours. Implement:

- Automatic reconnection with exponential backoff (see pattern above)
- Health checks and connection-status monitoring

## Production Deployment

### Systemd service

Create `/etc/systemd/system/pyth-monitor.service`:

```ini
[Unit]
Description=Pyth Price Monitor
After=network.target

[Service]
Type=simple
User=pyth
WorkingDirectory=/opt/pyth-monitor
Environment=RUST_LOG=info
ExecStart=/opt/pyth-monitor/target/release/pyth-price-monitor
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable pyth-monitor
sudo systemctl start pyth-monitor
sudo systemctl status pyth-monitor
```

### Docker deployment

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/pyth-price-monitor /usr/local/bin/
COPY .env /etc/pyth-monitor/.env
WORKDIR /etc/pyth-monitor
CMD ["pyth-price-monitor"]
```

Build and run:

```bash
docker build -t pyth-monitor .
docker run -d --name pyth-monitor --env-file .env pyth-monitor
```

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
- `hex`, `chrono`: utility and time formatting

## Build Profiles

Configured in `Cargo.toml`:

- `profile.dev`: `opt-level = 1`
- `profile.release`: `opt-level = 3`, `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"`

## Resources

- [Pyth Network Documentation](https://docs.pyth.network/)
- [Hermes API Reference](https://hermes-beta.pyth.network/docs/)
- [Alloy Documentation](https://alloy.rs/)
- [Price Feed IDs](https://pyth.network/developers/price-feed-ids)
