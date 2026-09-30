pub mod connection;
pub mod relay;

use anyhow::Context;
use quinn::Connection;
use rand::Rng;
use std::time::Instant;
use tokio::sync::mpsc;

use r_protocol::{Frame, HandshakeInitPayload};

use self::relay::{buffer_offline_message, extract_recipient, read_frame_bytes};
use crate::config::CHANNEL_BUFFER;
use crate::state::{OfflineBuffer, PeerMap};

pub async fn handle_connection(
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
            tokio::time::sleep(std::time::Duration::from_millis(jitter_ms)).await;

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
