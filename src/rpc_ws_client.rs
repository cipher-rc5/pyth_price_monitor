use anyhow::{Context, Result};
use fastwebsockets::{Frame, OpCode, handshake};
use http_body_util::Empty;
use hyper::header::{CONNECTION, UPGRADE};
use hyper::{Request, body::Bytes};
use reqwest::Url;
use serde::Deserialize;
use serde_json::json;
use std::future::Future;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::{self, RootCertStore};

pub async fn get_latest_block_number(endpoint: &str) -> Result<u64> {
    let url = Url::parse(endpoint).context("Invalid ETH_RPC_WS_URL")?;
    let scheme = url.scheme();

    if scheme != "ws" && scheme != "wss" {
        anyhow::bail!("ETH_RPC_WS_URL must use ws:// or wss://");
    }

    let host = url
        .host_str()
        .context("ETH_RPC_WS_URL must include a host")?;
    let port = url
        .port_or_known_default()
        .context("ETH_RPC_WS_URL must include a valid port")?;
    let socket_addr = format!("{host}:{port}");

    let mut http_url = url.clone();
    let mapped_scheme = if scheme == "wss" { "https" } else { "http" };
    http_url
        .set_scheme(mapped_scheme)
        .map_err(|_| anyhow::anyhow!("Failed to normalize WS URL"))?;

    let host_header = if let Some(explicit_port) = url.port() {
        format!("{host}:{explicit_port}")
    } else {
        host.to_string()
    };

    let request = Request::builder()
        .method("GET")
        .uri(http_url.as_str())
        .header("Host", host_header)
        .header(UPGRADE, "websocket")
        .header(CONNECTION, "upgrade")
        .header("Sec-WebSocket-Key", handshake::generate_key())
        .header("Sec-WebSocket-Version", "13")
        .body(Empty::<Bytes>::new())?;

    let tcp_stream = TcpStream::connect(socket_addr)
        .await
        .context("Failed to connect to WebSocket endpoint")?;

    let (mut ws, _) = if scheme == "wss" {
        let server_name = rustls::pki_types::ServerName::try_from(host.to_string())
            .context("Invalid TLS server name for ETH_RPC_WS_URL")?;
        let tls_stream = tls_connector().connect(server_name, tcp_stream).await?;
        handshake::client(&TokioExecutor, request, tls_stream).await?
    } else {
        handshake::client(&TokioExecutor, request, tcp_stream).await?
    };

    let rpc_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_blockNumber",
        "params": [],
    });

    ws.write_frame(Frame::text(serde_json::to_vec(&rpc_request)?.into()))
        .await
        .context("Failed to send eth_blockNumber over WebSocket")?;

    loop {
        let frame = ws
            .read_frame()
            .await
            .context("Failed reading WebSocket frame")?;

        match frame.opcode {
            OpCode::Text | OpCode::Binary => {
                let response: JsonRpcResponse = serde_json::from_slice(&frame.payload)
                    .context("Invalid JSON-RPC payload from WebSocket endpoint")?;

                if let Some(error) = response.error {
                    anyhow::bail!("WebSocket JSON-RPC error: {error}");
                }

                let hex = response
                    .result
                    .context("Missing `result` in eth_blockNumber response")?;

                let hex = hex.trim_start_matches("0x");
                let block_number =
                    u64::from_str_radix(hex, 16).context("Invalid hex block number")?;

                return Ok(block_number);
            }
            OpCode::Close => anyhow::bail!("WebSocket closed before eth_blockNumber response"),
            _ => {}
        }
    }
}

fn tls_connector() -> TlsConnector {
    // rustls can be built with multiple crypto backends in this dependency graph.
    // Install a process-level default provider explicitly to avoid runtime panics.
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }

    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();

    TlsConnector::from(Arc::new(config))
}

struct TokioExecutor;

impl<F> hyper::rt::Executor<F> for TokioExecutor
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    result: Option<String>,
    error: Option<serde_json::Value>,
}
