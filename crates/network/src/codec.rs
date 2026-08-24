use bytes::{Buf, BufMut, BytesMut};
use r_protocol::{Frame, ProtocolError, MAX_FRAME_SIZE};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Error, Debug)]
pub enum NetworkCodecError {
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Frame length {0} exceeds maximum allowed limit {MAX_FRAME_SIZE}")]
    FrameTooLarge(usize),
}

pub struct FrameCodec;

impl FrameCodec {
    pub fn new() -> Self {
        Self
    }
}

impl Decoder for FrameCodec {
    type Item = Frame;
    type Error = NetworkCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let frame_len = u32::from_be_bytes(length_bytes) as usize;

        if frame_len > MAX_FRAME_SIZE {
            return Err(NetworkCodecError::FrameTooLarge(frame_len));
        }

        if src.len() < 4 + frame_len {
            src.reserve((4 + frame_len) - src.len());
            return Ok(None);
        }

        src.advance(4);
        let frame_bytes = src.split_to(frame_len);
        let frame = Frame::decode(&frame_bytes)?;

        Ok(Some(frame))
    }
}

impl Encoder<Frame> for FrameCodec {
    type Error = NetworkCodecError;

    fn encode(&mut self, item: Frame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let encoded = item.encode_padded()?;
        let len = encoded.len() as u32;

        dst.reserve(4 + encoded.len());
        dst.put_u32(len);
        dst.put_slice(&encoded);

        Ok(())
    }
}
