use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset, Utc};
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub hermes_endpoint: String,
    pub btc_usd_price_feed_id: String,
    pub eth_rpc_url: Option<String>,
    pub eth_rpc_ws_url: Option<String>,
    pub output_timezone: OutputTimezone,
}

#[derive(Debug, Clone)]
pub struct OutputTimezone {
    label: String,
    offset_seconds: i32,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let hermes_endpoint = env::var("HERMES_ENDPOINT")
            .unwrap_or_else(|_| "https://hermes.pyth.network".to_string());

        let btc_usd_price_feed_id =
            env::var("BTC_USD_PRICE_FEED_ID").context("BTC_USD_PRICE_FEED_ID must be set")?;

        let eth_rpc_api_key = env::var("ETH_RPC_API_KEY").ok();
        let eth_rpc_url = resolve_rpc_endpoint(
            env::var("ETH_RPC_URL").ok(),
            eth_rpc_api_key.as_deref(),
            "ETH_RPC_URL",
        )?;
        let eth_rpc_ws_url = resolve_rpc_endpoint(
            env::var("ETH_RPC_WS_URL").ok(),
            eth_rpc_api_key.as_deref(),
            "ETH_RPC_WS_URL",
        )?;
        let output_timezone = OutputTimezone::parse(
            env::var("OUTPUT_TIMEZONE")
                .unwrap_or_else(|_| "EST".to_string())
                .as_str(),
        )?;

        Ok(Self {
            hermes_endpoint,
            btc_usd_price_feed_id,
            eth_rpc_url,
            eth_rpc_ws_url,
            output_timezone,
        })
    }

    pub fn get_price_feed_ids(&self) -> Vec<String> {
        vec![self.btc_usd_price_feed_id.clone()]
    }
}

impl OutputTimezone {
    pub fn parse(value: &str) -> Result<Self> {
        let normalized = value.trim().to_uppercase();
        let parsed = match normalized.as_str() {
            "UTC" => Self::new("UTC", 0),
            "EST" => Self::new("EST", -5 * 60 * 60),
            "CST" => Self::new("CST", -6 * 60 * 60),
            "MST" => Self::new("MST", -7 * 60 * 60),
            "PST" => Self::new("PST", -8 * 60 * 60),
            _ => parse_utc_offset(&normalized),
        };

        parsed.with_context(|| {
            "Invalid OUTPUT_TIMEZONE. Supported values: UTC, EST, CST, MST, PST, UTC+HH:MM, UTC-HH:MM"
        })
    }

    pub fn format_unix_timestamp(&self, unix_seconds: i64) -> String {
        let timestamp = DateTime::from_timestamp(unix_seconds, 0).unwrap_or_else(Utc::now);
        let offset =
            FixedOffset::east_opt(self.offset_seconds).expect("valid fixed timezone offset");
        timestamp
            .with_timezone(&offset)
            .format(&format!("%Y-%m-%d %H:%M:%S {}", self.label))
            .to_string()
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    fn new(label: impl Into<String>, offset_seconds: i32) -> Result<Self> {
        if FixedOffset::east_opt(offset_seconds).is_none() {
            anyhow::bail!("invalid timezone offset");
        }

        Ok(Self {
            label: label.into(),
            offset_seconds,
        })
    }
}

fn parse_utc_offset(value: &str) -> Result<OutputTimezone> {
    let rest = value.strip_prefix("UTC").context("missing UTC prefix")?;

    let (sign, hh_mm) = match rest.chars().next() {
        Some('+') => (1_i32, &rest[1..]),
        Some('-') => (-1_i32, &rest[1..]),
        _ => anyhow::bail!("missing + or - UTC offset sign"),
    };

    let mut parts = hh_mm.split(':');
    let hours: i32 = parts
        .next()
        .context("missing UTC offset hours")?
        .parse()
        .context("invalid UTC offset hours")?;
    let minutes: i32 = parts
        .next()
        .context("missing UTC offset minutes")?
        .parse()
        .context("invalid UTC offset minutes")?;

    if parts.next().is_some() {
        anyhow::bail!("too many components in UTC offset");
    }

    if !(0..=23).contains(&hours) || !(0..=59).contains(&minutes) {
        anyhow::bail!("UTC offset out of range");
    }

    let offset_seconds = sign * (hours * 3600 + minutes * 60);
    OutputTimezone::new(value, offset_seconds)
}

fn resolve_rpc_endpoint(
    endpoint: Option<String>,
    api_key: Option<&str>,
    field_name: &str,
) -> Result<Option<String>> {
    let Some(mut endpoint) = endpoint else {
        return Ok(None);
    };

    let contains_api_placeholder =
        endpoint.contains("{$API_KEY}") || endpoint.contains("${API_KEY}");

    if contains_api_placeholder {
        let api_key = api_key.with_context(|| {
            format!("{field_name} uses an API key placeholder but ETH_RPC_API_KEY is not set")
        })?;

        endpoint = endpoint
            .replace("{$API_KEY}", api_key)
            .replace("${API_KEY}", api_key);
    }

    Ok(Some(endpoint))
}
