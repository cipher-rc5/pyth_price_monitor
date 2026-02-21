//! Pyth price monitoring crate with optional Ethereum RPC integrations.
//!
//! This crate provides:
//! - Hermes REST + SSE client support
//! - In-memory price monitoring utilities
//! - Typed payload models for parsed and raw Pyth responses
//! - Optional low-latency Ethereum JSON-RPC over WebSocket
//!
//! # Configuration
//!
//! Runtime configuration is loaded from environment variables via
//! [`config::Config::from_env`]. API-key templating is supported for RPC URLs:
//! - `ETH_RPC_URL=https://provider/v1/{$API_KEY}`
//! - `ETH_RPC_WS_URL=wss://provider/ws/v1/${API_KEY}`
//! - `ETH_RPC_API_KEY=...`
//! - `OUTPUT_TIMEZONE=EST` (or `UTC`, `CST`, `MST`, `PST`, `UTC+HH:MM`, `UTC-HH:MM`)
//!
//! # Example
//!
//! ```no_run
//! use pyth_price_monitor::{config::Config, price_monitor::PriceMonitor};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let cfg = Config::from_env()?;
//!     let monitor = PriceMonitor::new(
//!         cfg.hermes_endpoint.clone(),
//!         cfg.get_price_feed_ids(),
//!         cfg.output_timezone.clone(),
//!     );
//!     let _prices = monitor.fetch_latest_once().await?;
//!     Ok(())
//! }
//! ```

/// Environment-driven runtime configuration.
pub mod config;
/// Hermes API client (latest snapshots + SSE streaming updates).
pub mod hermes_client;
/// High-level monitor service that tracks and logs parsed prices.
pub mod price_monitor;
/// Low-latency Ethereum JSON-RPC over WebSocket.
pub mod rpc_ws_client;
/// Data types for raw and parsed Pyth responses.
pub mod types;
