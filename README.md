# Pyth Price Monitor

Real-time Bitcoin/USD price monitoring using Pyth Network's Hermes API, built with Rust, Alloy, and dotenvy

## Features

- Real-time BTC/USD price streaming via Pyth Hermes SSE endpoint
- Latest price fetch with confidence intervals
- Exponentially-weighted moving average (EMA) prices
- Ethereum integration using Alloy for blockchain interactions
- Optional Ethereum WebSocket RPC path via fastwebsockets for low-latency calls
- Environment-based configuration using dotenvy
- Structured logging with tracing

## Architecture

The codebase follows a modular architecture:

- **config.rs**: Environment configuration management using dotenvy
- **types.rs**: Data structures for Pyth price feeds
- **hermes_client.rs**: HTTP client for Hermes API (latest + streaming)
- **price_monitor.rs**: Service layer for price monitoring and state management
- **main.rs**: Application entry point with Alloy integration examples

## Prerequisites

- Rust 1.70+
- Internet connection for Hermes API access

## Setup

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

API-key based RPC endpoints are also supported. If your endpoint uses a placeholder,
set `ETH_RPC_API_KEY` and use either `{$API_KEY}` or `${API_KEY}` in the URL:

```env
ETH_RPC_API_KEY=your_blink_key
ETH_RPC_URL=https://eth.blinklabs.xyz/v1/{$API_KEY}
ETH_RPC_WS_URL=wss://eth.blinklabs.xyz/ws/v1/{$API_KEY}
```

Timezone output can be configured with:
- `OUTPUT_TIMEZONE=UTC`
- `OUTPUT_TIMEZONE=EST` (also supports `CST`, `MST`, `PST`)
- `OUTPUT_TIMEZONE=UTC+02:00` (or `UTC-07:00`)

## Usage

### Run the price monitor:
```bash
cargo run
```

### Build for production:
```bash
cargo build --release
./target/release/pyth-price-monitor
```

## Output Example

```
Initializing Pyth Price Monitor
Hermes Endpoint: https://hermes.pyth.network
BTC/USD Price Feed ID: 0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43
Fetching initial BTC/USD price...
Initial BTC/USD Price: $97432.15
Confidence Interval: ±$42.71
EMA Price: $97445.28
Connecting to Ethereum RPC: https://eth.llamarpc.com
Current Ethereum block: 21234567
Starting real-time price stream...
Press Ctrl+C to stop
Price Update - ID: 0xe62df6...5b43 | Price: $97435.20 ± $43.15 | EMA: $97446.12 | Time: 2025-02-19 15:23:45 UTC
Price Update - ID: 0xe62df6...5b43 | Price: $97438.50 ± $41.82 | EMA: $97447.03 | Time: 2025-02-19 15:23:47 UTC
```

## Price Feed IDs

Find additional price feed IDs at: https://pyth.network/developers/price-feed-ids

Common feeds:
- BTC/USD: `0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43`
- ETH/USD: `0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace`
- SOL/USD: `0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d`

## Pyth Network Details

### Hermes API

Hermes is Pyth's off-chain price service that provides:
- REST API for latest price queries
- Server-Sent Events (SSE) for real-time streaming
- 400+ price feeds across multiple asset classes
- Sub-second latency for price updates

### Price Structure

Each price feed includes:
- **Price**: Current price with confidence interval
- **EMA Price**: Exponentially-weighted moving average
- **Exponent**: Power of 10 to multiply the price by
- **Publish Time**: Unix timestamp of price publication

### Price Calculation

Raw prices are integers that must be adjusted by the exponent:
```
actual_price = price * 10^expo
```

Example:
- Raw price: `4426101900000`
- Exponent: `-8`
- Actual price: `44261.019` USD

## Alloy Integration

The codebase demonstrates Alloy integration for:
- Connecting to Ethereum nodes
- Fetching current block numbers
- Querying account balances
- Ready for smart contract interactions

## Error Handling

The application uses `anyhow` for comprehensive error handling:
- Network connection failures
- JSON parsing errors
- Stream disconnections with automatic reconnection support

## Production Considerations

1. **Private Hermes Endpoint**: For production, consider using a private Hermes endpoint from providers like Triton One for better reliability

2. **Reconnection Logic**: The stream auto-closes after 24 hours. Implement reconnection logic for long-running services

3. **Rate Limiting**: Monitor request rates if using the public endpoint

4. **Error Recovery**: Add exponential backoff for connection retries

5. **Data Persistence**: Store price history in a database for analytics

## Dependencies

Core dependencies:
- `alloy`: Ethereum library for blockchain interactions
- `dotenvy`: Environment variable management
- `tokio`: Async runtime
- `reqwest`: HTTP client for Hermes API
- `fastwebsockets`: WebSocket JSON-RPC client path
- `serde`: Serialization/deserialization
- `tracing`: Structured logging

## License

MIT
