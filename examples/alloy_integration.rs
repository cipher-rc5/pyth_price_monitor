use alloy::{
    primitives::{U256, address},
    providers::{Provider, ProviderBuilder},
};
use anyhow::{Context, Result};
use pyth_price_monitor::{config::Config, hermes_client::HermesClient};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let config = Config::from_env()?;

    let eth_rpc_url = config
        .eth_rpc_url
        .as_deref()
        .context("ETH_RPC_URL must be set in .env for this example")?;

    let client = HermesClient::new(config.hermes_endpoint);
    let provider = ProviderBuilder::new().connect(eth_rpc_url).await?;

    let block_number = provider.get_block_number().await?;
    info!("Connected to Ethereum - Block: {}", block_number);

    info!("Fetching BTC/USD price from Pyth...");
    let price_feeds = vec![config.btc_usd_price_feed_id.clone()];
    let feeds = client.get_latest_price_updates(&price_feeds).await?;

    if let Some(feed) = feeds.first() {
        let parsed = feed.parse()?;
        info!("BTC/USD Price: ${:.2}", parsed.price.price);
        info!("Confidence: +/- ${:.2}", parsed.price.conf);
        info!("EMA Price: ${:.2}", parsed.ema_price.price);

        let price_scaled = (parsed.price.price * 100.0) as u64;
        let price_u256 = U256::from(price_scaled);
        info!("Price scaled for on-chain use: {}", price_u256);

        info!("In a production scenario, you would:");
        info!("1. Create a signed transaction to update the on-chain price");
        info!("2. Use the Pyth contract's updatePriceFeeds method");
        info!("3. Pass the binary price update data from Hermes");
        info!("4. Pay the required update fee");

        let vitalik = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
        let balance = provider.get_balance(vitalik).await?;
        info!("Example - Vitalik's balance: {} wei", balance);
    }

    Ok(())
}
