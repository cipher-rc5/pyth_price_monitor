use alloy::primitives::address;
use alloy::providers::{Provider, ProviderBuilder};
use anyhow::Result;
use pyth_price_monitor::config::Config;
use pyth_price_monitor::price_monitor::PriceMonitor;
use pyth_price_monitor::rpc_ws_client;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let config = Config::from_env()?;

    info!(event = "service_start", "Initializing Pyth Price Monitor");
    info!(
        event = "config_loaded",
        hermes_endpoint = %config.hermes_endpoint,
        btc_usd_price_feed_id = %config.btc_usd_price_feed_id,
        "runtime configuration loaded"
    );

    let monitor = PriceMonitor::new(
        config.hermes_endpoint.clone(),
        config.get_price_feed_ids(),
        config.output_timezone.clone(),
    );

    info!(
        event = "initial_fetch_start",
        "fetching initial BTC/USD price"
    );
    let initial_prices = monitor.fetch_latest_once().await?;

    for price in &initial_prices {
        let publish_time_unix = price.price.publish_time;
        let publish_time_local = config
            .output_timezone
            .format_unix_timestamp(publish_time_unix);

        info!(
            event = "price_snapshot",
            feed_id = %price.id,
            price = price.price.price,
            confidence = price.price.conf,
            ema_price = price.ema_price.price,
            publish_time_unix,
            publish_time_local = %publish_time_local,
            timezone = %config.output_timezone.label(),
            "price_snapshot"
        );
    }

    if let Some(eth_rpc_ws_url) = &config.eth_rpc_ws_url {
        info!("Connecting to Ethereum WebSocket RPC: {}", eth_rpc_ws_url);

        let block_number = rpc_ws_client::get_latest_block_number(eth_rpc_ws_url).await?;
        info!("Current Ethereum block via WebSocket: {}", block_number);
    } else if let Some(eth_rpc_url) = &config.eth_rpc_url {
        info!("Connecting to Ethereum RPC: {}", eth_rpc_url);

        let provider = ProviderBuilder::new().connect(eth_rpc_url).await?;

        let block_number = provider.get_block_number().await?;
        info!("Current Ethereum block: {}", block_number);

        let vitalik = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
        let balance = provider.get_balance(vitalik).await?;
        info!("Vitalik's balance: {} wei", balance);
    }

    info!(event = "stream_start", "starting real-time price stream");
    info!(
        event = "operator_hint",
        action = "ctrl_c_to_stop",
        "Press Ctrl+C to stop"
    );

    monitor.start_streaming().await?;

    Ok(())
}
