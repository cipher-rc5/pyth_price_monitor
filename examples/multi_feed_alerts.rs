use anyhow::Result;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use pyth_price_monitor::{config::Config, hermes_client::HermesClient};
use std::collections::HashMap;
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

struct PriceAlert {
    threshold_percent: f64,
    last_price: Option<f64>,
}

impl PriceAlert {
    fn new(threshold_percent: f64) -> Self {
        Self {
            threshold_percent,
            last_price: None,
        }
    }

    fn check_alert(&mut self, current_price: f64) -> Option<String> {
        if let Some(last) = self.last_price {
            let percent_change = ((current_price - last) / last) * 100.0;

            if percent_change.abs() >= self.threshold_percent {
                let direction = if percent_change > 0.0 { "UP" } else { "DOWN" };
                let alert = format!(
                    "ALERT: Price moved {} by {:.2}% (${:.2} -> ${:.2})",
                    direction,
                    percent_change.abs(),
                    last,
                    current_price
                );
                self.last_price = Some(current_price);
                return Some(alert);
            }
        } else {
            self.last_price = Some(current_price);
        }

        None
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let config = Config::from_env()?;
    let client = HermesClient::new(config.hermes_endpoint);

    let eth_usd_id = "0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace";
    let price_feeds = vec![config.btc_usd_price_feed_id.clone(), eth_usd_id.to_string()];

    let feed_names: HashMap<String, &str> = [
        (config.btc_usd_price_feed_id.clone(), "BTC/USD"),
        (eth_usd_id.to_string(), "ETH/USD"),
    ]
    .into_iter()
    .collect();

    let mut alerts: HashMap<String, PriceAlert> = HashMap::new();
    for id in &price_feeds {
        alerts.insert(id.clone(), PriceAlert::new(1.0));
    }

    info!("Fetching initial prices for BTC/USD and ETH/USD...");
    let initial = client.get_latest_price_updates(&price_feeds).await?;

    for feed in initial {
        let parsed = feed.parse()?;
        let name = feed_names.get(&parsed.id).copied().unwrap_or("Unknown");
        info!(
            "{}: ${:.2} +/- ${:.2}",
            name, parsed.price.price, parsed.price.conf
        );

        if let Some(alert) = alerts.get_mut(&parsed.id) {
            let _ = alert.check_alert(parsed.price.price);
        }
    }

    info!("Starting real-time stream with 1% price alerts...");

    let mut stream = client.stream_price_updates(&price_feeds).await?;
    while let Some(result) = stream.next().await {
        match result {
            Ok(feeds) => {
                for feed in feeds {
                    if let Ok(parsed) = feed.parse() {
                        let name = feed_names.get(&parsed.id).copied().unwrap_or("Unknown");
                        let timestamp = DateTime::from_timestamp(parsed.price.publish_time, 0)
                            .unwrap_or_else(Utc::now);

                        info!(
                            "[{}] {}: ${:.2} +/- ${:.2} | EMA: ${:.2} | {}",
                            timestamp.format("%H:%M:%S"),
                            name,
                            parsed.price.price,
                            parsed.price.conf,
                            parsed.ema_price.price,
                            timestamp.format("%Y-%m-%d")
                        );

                        if let Some(alert) = alerts.get_mut(&parsed.id)
                            && let Some(alert_msg) = alert.check_alert(parsed.price.price)
                        {
                            info!("*** {} {} ***", name, alert_msg);
                        }
                    }
                }
            }
            Err(err) => {
                error!("Stream error: {}", err);
            }
        }
    }

    Ok(())
}
