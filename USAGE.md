# Usage Guide

## Quick Start

1. **Setup environment**:
```bash
cp .env.example .env
```

2. **Run the basic monitor**:
```bash
cargo run
```

3. **Run advanced examples**:
```bash
# Multi-feed with alerts
cargo run --example multi_feed_alerts

# Alloy integration example
cargo run --example alloy_integration
```

## Configuration

### Environment Variables

**Required**:
- `BTC_USD_PRICE_FEED_ID`: Pyth price feed identifier for BTC/USD

**Optional**:
- `HERMES_ENDPOINT`: Hermes API endpoint (default: https://hermes.pyth.network)
- `ETH_RPC_URL`: Ethereum RPC endpoint for Alloy integration
- `ETH_RPC_WS_URL`: Ethereum WebSocket RPC endpoint for fast JSON-RPC calls
- `ETH_RPC_API_KEY`: API key used to replace `{$API_KEY}` / `${API_KEY}` placeholders
- `OUTPUT_TIMEZONE`: Timestamp output timezone (`UTC`, `EST`, `CST`, `MST`, `PST`, `UTC+HH:MM`, `UTC-HH:MM`)
- `LOG_LEVEL`: Logging level (default: info)

### Adding Multiple Price Feeds

Edit your `.env`:
```env
HERMES_ENDPOINT=https://hermes.pyth.network
BTC_USD_PRICE_FEED_ID=0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43
ETH_USD_PRICE_FEED_ID=0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace
SOL_USD_PRICE_FEED_ID=0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d
ETH_RPC_URL=https://eth.llamarpc.com
ETH_RPC_WS_URL=wss://eth.llamarpc.com
OUTPUT_TIMEZONE=EST

# Optional if URL contains placeholder tokens
ETH_RPC_API_KEY=your_provider_key
```

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
println!("Confidence: ±${:.2}", parsed.price.conf);
println!("EMA: ${:.2}", parsed.ema_price.price);
```

## Common Patterns

### 1. Price Monitoring with Alerts

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

### 2. Storing Price History

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

### 3. Reconnection Logic

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

### 4. Using with Alloy for On-Chain Updates

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
    // and submit to the Pyth contract
    
    // Example: Scale price for on-chain storage
    let price_scaled = (price * 100.0) as u64;
    let price_u256 = U256::from(price_scaled);
    
    Ok(())
}
```

## Error Handling

### Network Errors

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

### Stream Errors

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

1. **Batch Requests**: Fetch multiple price feeds in one request
2. **Connection Pooling**: Reuse the HermesClient instance
3. **Buffer Size**: Adjust tokio runtime buffer sizes for high throughput
4. **Logging**: Use appropriate log levels in production

## Troubleshooting

### Connection Failures

```bash
# Check network connectivity
curl https://hermes.pyth.network/v2/updates/price/latest?ids[]=0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43

# Verify price feed ID
# Visit: https://pyth.network/developers/price-feed-ids
```

### Rate Limiting

If using the public endpoint intensively, consider:
- Using a private Hermes endpoint
- Implementing request throttling
- Caching recent prices

### Stream Disconnections

The SSE stream auto-closes after 24 hours. Implement:
- Automatic reconnection with exponential backoff
- Health checks
- Monitoring for connection status

## Production Deployment

### Systemd Service

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

### Docker Deployment

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

## Resources

- [Pyth Network Documentation](https://docs.pyth.network/)
- [Hermes API Reference](https://hermes-beta.pyth.network/docs/)
- [Alloy Documentation](https://alloy.rs/)
- [Price Feed IDs](https://pyth.network/developers/price-feed-ids)
