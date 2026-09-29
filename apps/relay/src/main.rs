mod config;
mod network;
mod state;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context;
use dashmap::DashMap;
use quinn::Endpoint;

use config::{OFFLINE_TTL, RELAY_ADDR};
use network::connection::make_server_config;
use network::handle_connection;
use state::{OfflineBuffer, PeerMap};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let peer_map: PeerMap = Arc::new(DashMap::new());
    let offline_buffer: OfflineBuffer = Arc::new(DashMap::new());

    let server_config = make_server_config().context("Failed to build TLS config")?;
    let addr: SocketAddr = RELAY_ADDR.parse()?;
    let endpoint = Endpoint::server(server_config, addr)?;

    tracing::info!("Stateless Blind Relay engine running on {RELAY_ADDR}");

    let cleanup_buffer = Arc::clone(&offline_buffer);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            let now = Instant::now();
            cleanup_buffer.retain(|_, queue| {
                queue.retain(|(created, _)| now.duration_since(*created) < OFFLINE_TTL);
                !queue.is_empty()
            });
        }
    });

    while let Some(incoming) = endpoint.accept().await {
        let peer_map = Arc::clone(&peer_map);
        let offline_buffer = Arc::clone(&offline_buffer);

        tokio::spawn(async move {
            match incoming.await {
                Ok(conn) => {
                    if let Err(e) = handle_connection(conn, peer_map, offline_buffer).await {
                        tracing::warn!("Connection terminated: {e:#}");
                    }
                }
                Err(e) => tracing::warn!("Handshake error: {e}"),
            }
        });
    }

    Ok(())
}