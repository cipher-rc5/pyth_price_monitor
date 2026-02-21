use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFeed {
    pub id: String,
    pub price: Price,
    pub ema_price: Price,
    pub metadata: Option<FeedMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    pub price: String,
    pub conf: String,
    pub expo: i32,
    pub publish_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedMetadata {
    pub slot: Option<u64>,
    pub proof_available_time: Option<i64>,
    pub prev_publish_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedPriceFeed {
    pub id: String,
    pub price: ParsedPrice,
    pub ema_price: ParsedPrice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedPrice {
    pub price: f64,
    pub conf: f64,
    pub expo: i32,
    pub publish_time: i64,
}

impl PriceFeed {
    pub fn parse(&self) -> Result<ParsedPriceFeed> {
        let price_val = self.price.price.parse::<f64>()?;
        let conf_val = self.price.conf.parse::<f64>()?;
        let expo = self.price.expo;

        let divisor = 10_f64.powi(-expo);

        let ema_price_val = self.ema_price.price.parse::<f64>()?;
        let ema_conf_val = self.ema_price.conf.parse::<f64>()?;

        Ok(ParsedPriceFeed {
            id: self.id.clone(),
            price: ParsedPrice {
                price: price_val / divisor,
                conf: conf_val / divisor,
                expo,
                publish_time: self.price.publish_time,
            },
            ema_price: ParsedPrice {
                price: ema_price_val / divisor,
                conf: ema_conf_val / divisor,
                expo,
                publish_time: self.ema_price.publish_time,
            },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesLatestResponse {
    pub binary: Option<BinaryData>,
    pub parsed: Option<Vec<PriceFeed>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryData {
    pub encoding: String,
    pub data: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamUpdate {
    pub binary: Option<BinaryData>,
    pub parsed: Option<Vec<PriceFeed>>,
}
