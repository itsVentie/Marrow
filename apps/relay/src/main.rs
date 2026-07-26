use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use bytes::Bytes;
use dashmap::DashMap;
use quinn::{Connection, Endpoint, ServerConfig};
use tokio::sync::mpsc;

use r_protocol::MAX_FRAME_SIZE;

type PeerId = [u8; 32];
type PeerMap = Arc<DashMap<PeerId, mpsc::Sender<Bytes>>>;

const RELAY_ADDR: &str = "0.0.0.0:9000";
const CHANNEL_BUFFER: usize = 512;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let peer_map: PeerMap = Arc::new(DashMap::new());
    let server_config = make_server_config().context("Failed to build TLS config")?;
    let addr: SocketAddr = RELAY_ADDR.parse()?;
    let endpoint = Endpoint::server(server_config, addr)?;

    tracing::info!("Relay listening on {RELAY_ADDR}");

    while let Some(incoming) = endpoint.accept().await {
        let peer_map = Arc::clone(&peer_map);
        tokio::spawn(async move {
            match incoming.await {
                Ok(conn) => {
                    if let Err(e) = handle_connection(conn, peer_map).await {
                        tracing::warn!("Connection closed: {e:#}");
                    }
                }
                Err(e) => tracing::warn!("Incoming handshake failed: {e}"),
            }
        });
    }

    Ok(())
}

async fn handle_connection(conn: Connection, peer_map: PeerMap) -> anyhow::Result<()> {
    let (mut send_stream, mut recv_stream) = conn
        .accept_bi()
        .await
        .context("Failed to accept bi-directional stream")?;

    let mut peer_id = PeerId::default();
    recv_stream
        .read_exact(&mut peer_id)
        .await
        .context("Failed to read peer ID during registration")?;

    let (tx, mut rx) = mpsc::channel::<Bytes>(CHANNEL_BUFFER);
    peer_map.insert(peer_id, tx);
    tracing::info!("Peer registered: {}", hex::encode(peer_id));

    let write_task = tokio::spawn(async move {
        while let Some(frame) = rx.recv().await {
            let len = frame.len() as u32;
            send_stream.write_all(&len.to_le_bytes()).await?;
            send_stream.write_all(&frame).await?;
        }
        Ok::<_, anyhow::Error>(())
    });

    let read_result = read_loop(&mut recv_stream, &peer_map).await;

    peer_map.remove(&peer_id);
    let _ = write_task.await;
    tracing::info!("Peer disconnected: {}", hex::encode(peer_id));

    read_result
}

async fn read_loop(
    recv: &mut quinn::RecvStream,
    peer_map: &PeerMap,
) -> anyhow::Result<()> {
    loop {
        let mut recipient = PeerId::default();
        match recv.read_exact(&mut recipient).await {
            Ok(_) => {}
            Err(_) => break,
        }

        let mut len_buf = [0u8; 4];
        recv.read_exact(&mut len_buf)
            .await
            .context("Failed to read frame length")?;
        let len = u32::from_le_bytes(len_buf) as usize;

        if len > MAX_FRAME_SIZE {
            anyhow::bail!("Frame too large: {len} bytes (max {MAX_FRAME_SIZE})");
        }

        let mut frame_buf = vec![0u8; len];
        recv.read_exact(&mut frame_buf)
            .await
            .context("Failed to read frame body")?;

        match peer_map.get(&recipient) {
            Some(sender) => {
                if sender.send(Bytes::from(frame_buf)).await.is_err() {
                    tracing::debug!(
                        "Recipient {} disconnected mid-send",
                        hex::encode(recipient)
                    );
                }
            }
            None => {
                tracing::debug!("No route to peer: {}", hex::encode(recipient));
            }
        }
    }
    Ok(())
}

fn make_server_config() -> anyhow::Result<ServerConfig> {
    let key_pair = rcgen::KeyPair::generate()?;
    let cert = rcgen::CertificateParams::new(vec!["localhost".to_string()])
        .context("Failed to build cert params")?
        .self_signed(&key_pair)
        .context("Failed to self-sign certificate")?;

    let cert_der = cert.der().clone();
    let key_der = rustls_pki_types::PrivateKeyDer::from(
        rustls_pki_types::PrivatePkcs8KeyDer::from(key_pair.serialize_der()),
    );

    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der)
        .context("Failed to build rustls ServerConfig")?;

    Ok(ServerConfig::with_crypto(Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(tls_config)
            .context("Failed to build QUIC server config")?,
    )))
}
