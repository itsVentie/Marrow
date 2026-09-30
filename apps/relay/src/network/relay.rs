use bytes::BytesMut;
use r_protocol::{EncryptedMessagePayload, Frame, HandshakeResponsePayload};
use std::time::Instant;

use crate::config::MAX_OFFLINE_QUEUE_LEN;
use crate::state::{OfflineBuffer, PeerId};

pub fn extract_recipient(frame: &Frame) -> Option<PeerId> {
    match frame {
        Frame::HandshakeInit(payload) => Some(payload.recipient_pubkey),
        Frame::HandshakeResponse(HandshakeResponsePayload {
            recipient_pubkey, ..
        }) => Some(*recipient_pubkey),
        Frame::Message(EncryptedMessagePayload {
            recipient_pubkey, ..
        }) => Some(*recipient_pubkey),
        _ => None,
    }
}

pub fn buffer_offline_message(offline_buffer: &OfflineBuffer, recipient: PeerId, frame: Frame) {
    tracing::debug!(
        "Buffering offline frame for peer {}",
        hex::encode(recipient)
    );
    let mut entry = offline_buffer.entry(recipient).or_default();
    if entry.len() >= MAX_OFFLINE_QUEUE_LEN {
        entry.pop_front();
    }
    entry.push_back((Instant::now(), frame));
}

pub async fn read_frame_bytes(recv: &mut quinn::RecvStream) -> anyhow::Result<Vec<u8>> {
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
