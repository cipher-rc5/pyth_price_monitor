use crate::config::OutputTimezone;
use crate::hermes_client::HermesClient;
use crate::types::ParsedPriceFeed;
use anyhow::Result;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

pub struct PriceMonitor {
    client: HermesClient,
    price_feeds: Vec<String>,
    output_timezone: OutputTimezone,
    latest_prices: Arc<RwLock<Vec<ParsedPriceFeed>>>,
}

impl PriceMonitor {
    pub fn new(
        hermes_endpoint: String,
        price_feeds: Vec<String>,
        output_timezone: OutputTimezone,
    ) -> Self {
        Self {
            client: HermesClient::new(hermes_endpoint),
            price_feeds,
            output_timezone,
            latest_prices: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn fetch_latest_once(&self) -> Result<Vec<ParsedPriceFeed>> {
        let feeds = self
            .client
            .get_latest_price_updates(&self.price_feeds)
            .await?;

        let parsed_feeds: Result<Vec<_>> = feeds.iter().map(|feed| feed.parse()).collect();
        let parsed_feeds = parsed_feeds?;

        let mut latest = self.latest_prices.write().await;
        *latest = parsed_feeds.clone();

        Ok(parsed_feeds)
    }

    pub async fn start_streaming(&self) -> Result<()> {
        let mut stream = self.client.stream_price_updates(&self.price_feeds).await?;

        info!("Starting price stream monitor");

        while let Some(result) = stream.next().await {
            match result {
                Ok(feeds) => {
                    for feed in feeds {
                        match feed.parse() {
                            Ok(parsed) => {
                                self.process_price_update(parsed).await;
                            }
                            Err(err) => {
                                error!("Failed to parse price feed: {}", err);
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

    pub async fn get_latest_prices(&self) -> Vec<ParsedPriceFeed> {
        self.latest_prices.read().await.clone()
    }

    async fn process_price_update(&self, parsed_feed: ParsedPriceFeed) {
        let publish_time_unix = parsed_feed.price.publish_time;
        let publish_time_local = self
            .output_timezone
            .format_unix_timestamp(publish_time_unix);

        info!(
            event = "price_update",
            feed_id = %parsed_feed.id,
            price = parsed_feed.price.price,
            confidence = parsed_feed.price.conf,
            ema_price = parsed_feed.ema_price.price,
            publish_time_unix,
            publish_time_local = %publish_time_local,
            timezone = %self.output_timezone.label(),
            "price_update"
        );

        let mut latest = self.latest_prices.write().await;

        if let Some(existing) = latest.iter_mut().find(|feed| feed.id == parsed_feed.id) {
            *existing = parsed_feed;
        } else {
            latest.push(parsed_feed);
        }
    }
}
