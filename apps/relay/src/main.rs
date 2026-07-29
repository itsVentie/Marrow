use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context;
use bytes::BytesMut;
use dashmap::DashMap;
use quinn::{Connection, Endpoint, ServerConfig};
use rand::Rng;
use tokio::sync::mpsc;

use r_protocol::{EncryptedMessagePayload, Frame, HandshakeInitPayload, HandshakeResponsePayload};

type PeerId = [u8; 32];
type PeerMap = Arc<DashMap<PeerId, mpsc::Sender<Frame>>>;
type OfflineBuffer = Arc<DashMap<PeerId, VecDeque<(Instant, Frame)>>>;

const RELAY_ADDR: &str = "0.0.0.0:9000";
const CHANNEL_BUFFER: usize = 512;
const OFFLINE_TTL: Duration = Duration::from_secs(300);

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

async fn handle_connection(
    conn: Connection,
    peer_map: PeerMap,
    offline_buffer: OfflineBuffer,
) -> anyhow::Result<()> {
    let (mut send_stream, mut recv_stream) =
        conn.accept_bi().await.context("Failed to accept stream")?;

    let reg_frame_bytes = read_frame_bytes(&mut recv_stream).await?;
    let reg_frame = Frame::decode(&reg_frame_bytes)?;

    let peer_id = match reg_frame {
        Frame::HandshakeInit(HandshakeInitPayload { sender_pubkey, .. }) => sender_pubkey,
        _ => anyhow::bail!("Invalid registration frame: expected HandshakeInit"),
    };

    let (tx, mut rx) = mpsc::channel::<Frame>(CHANNEL_BUFFER);
    peer_map.insert(peer_id, tx.clone());
    tracing::info!("Peer registered: {}", hex::encode(peer_id));

    if let Some((_, queue)) = offline_buffer.remove(&peer_id) {
        tracing::info!(
            "Delivering {} buffered messages to {}",
            queue.len(),
            hex::encode(peer_id)
        );
        for (_, frame) in queue {
            let _ = tx.send(frame).await;
        }
    }

    let write_task = tokio::spawn(async move {
        while let Some(frame) = rx.recv().await {
            let bytes = frame.encode_padded()?;
            let len = bytes.len() as u32;

            let jitter_ms = rand::thread_rng().gen_range(2..=15);
            tokio::time::sleep(Duration::from_millis(jitter_ms)).await;

            send_stream.write_all(&len.to_le_bytes()).await?;
            send_stream.write_all(&bytes).await?;
        }
        Ok::<_, anyhow::Error>(())
    });

    let read_result = read_loop(&mut recv_stream, &peer_map, &offline_buffer, &tx).await;

    peer_map.remove(&peer_id);
    let _ = write_task.await;
    tracing::info!("Peer disconnected: {}", hex::encode(peer_id));

    read_result
}

async fn read_loop(
    recv: &mut quinn::RecvStream,
    peer_map: &PeerMap,
    offline_buffer: &OfflineBuffer,
    self_tx: &mpsc::Sender<Frame>,
) -> anyhow::Result<()> {
    loop {
        let frame_bytes = match read_frame_bytes(recv).await {
            Ok(bytes) => bytes,
            Err(_) => break,
        };

        let frame = match Frame::decode(&frame_bytes) {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!("Failed to decode incoming frame: {e}");
                continue;
            }
        };

        match &frame {
            Frame::Dummy(_) => {}
            Frame::Ping => {
                let _ = self_tx.send(Frame::Pong).await;
            }
            Frame::Pong => {}
            _ => {
                if let Some(recipient) = extract_recipient(&frame) {
                    if let Some(sender) = peer_map.get(&recipient) {
                        if sender.send(frame.clone()).await.is_err() {
                            buffer_offline_message(offline_buffer, recipient, frame);
                        }
                    } else {
                        buffer_offline_message(offline_buffer, recipient, frame);
                    }
                }
            }
        }
    }
    Ok(())
}

fn buffer_offline_message(offline_buffer: &OfflineBuffer, recipient: PeerId, frame: Frame) {
    tracing::debug!(
        "Buffering offline frame for peer {}",
        hex::encode(recipient)
    );
    let mut entry = offline_buffer.entry(recipient).or_default();
    if entry.len() >= 100 {
        entry.pop_front();
    }
    entry.push_back((Instant::now(), frame));
}

fn extract_recipient(frame: &Frame) -> Option<PeerId> {
    match frame {
        Frame::HandshakeInit(HandshakeInitPayload { sender_pubkey, .. }) => Some(*sender_pubkey),
        Frame::HandshakeResponse(HandshakeResponsePayload {
            recipient_pubkey, ..
        }) => Some(*recipient_pubkey),
        Frame::Message(EncryptedMessagePayload {
            recipient_pubkey, ..
        }) => Some(*recipient_pubkey),
        _ => None,
    }
}

async fn read_frame_bytes(recv: &mut quinn::RecvStream) -> anyhow::Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    recv.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf) as usize;

    if len > r_protocol::MAX_FRAME_SIZE {
        anyhow::bail!("Frame size exceeds limit: {len}");
    }

    let mut frame_buf = BytesMut::with_capacity(len);
    frame_buf.resize(len, 0);
    recv.read_exact(&mut frame_buf).await?;

    Ok(frame_buf.to_vec())
}

fn make_server_config() -> anyhow::Result<ServerConfig> {
    let key_pair = rcgen::KeyPair::generate()?;
    let cert = rcgen::CertificateParams::new(vec!["localhost".to_string()])
        .context("Failed to build cert params")?
        .self_signed(&key_pair)
        .context("Failed to self-sign certificate")?;

    let cert_der = rustls_pki_types::CertificateDer::from(cert.der().to_vec());
    let key_der = rustls_pki_types::PrivateKeyDer::Pkcs8(
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
