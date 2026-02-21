use crate::types::{HermesLatestResponse, PriceFeed, StreamUpdate};
use anyhow::{Context, Result};
use futures::stream::{Stream, StreamExt};
use reqwest::Client;
use std::pin::Pin;
use tracing::{debug, info, warn};

pub struct HermesClient {
    endpoint: String,
    client: Client,
}

impl HermesClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            client: Client::new(),
        }
    }

    pub async fn get_latest_price_updates(&self, price_ids: &[String]) -> Result<Vec<PriceFeed>> {
        let url = self.build_latest_url(price_ids);

        debug!("Fetching latest prices from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch latest prices")?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {}", response.status());
        }

        let data: HermesLatestResponse =
            response.json().await.context("Failed to parse response")?;

        data.parsed
            .ok_or_else(|| anyhow::anyhow!("No parsed data in response"))
    }

    pub async fn stream_price_updates(
        &self,
        price_ids: &[String],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Vec<PriceFeed>>> + Send>>> {
        let url = self.build_stream_url(price_ids);

        info!("Connecting to price stream: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to stream")?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {}", response.status());
        }

        let stream = response.bytes_stream();

        let price_stream = stream.filter_map(|chunk_result| async move {
            match chunk_result {
                Ok(chunk) => {
                    let text = String::from_utf8_lossy(&chunk);

                    for line in text.lines() {
                        if line.starts_with("data:") {
                            let json_str = line.trim_start_matches("data:");

                            match serde_json::from_str::<StreamUpdate>(json_str) {
                                Ok(update) => {
                                    if let Some(parsed) = update.parsed {
                                        return Some(Ok(parsed));
                                    }
                                }
                                Err(err) => {
                                    warn!("Failed to parse stream update: {}", err);
                                }
                            }
                        }
                    }

                    None
                }
                Err(err) => Some(Err(anyhow::anyhow!("Stream error: {}", err))),
            }
        });

        Ok(Box::pin(price_stream))
    }

    fn build_latest_url(&self, price_ids: &[String]) -> String {
        let ids_param = price_ids
            .iter()
            .map(|id| format!("ids[]={id}"))
            .collect::<Vec<_>>()
            .join("&");

        format!(
            "{}/v2/updates/price/latest?{}&parsed=true",
            self.endpoint, ids_param
        )
    }

    fn build_stream_url(&self, price_ids: &[String]) -> String {
        let ids_param = price_ids
            .iter()
            .map(|id| format!("ids[]={id}"))
            .collect::<Vec<_>>()
            .join("&");

        format!(
            "{}/v2/updates/price/stream?{}&parsed=true",
            self.endpoint, ids_param
        )
    }
}
