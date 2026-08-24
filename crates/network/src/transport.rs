use crate::codec::{FrameCodec, NetworkCodecError};
use futures::{SinkExt, StreamExt};
use r_protocol::Frame;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub struct FrameTransport {
    framed: Framed<TcpStream, FrameCodec>,
}

impl FrameTransport {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            framed: Framed::new(stream, FrameCodec::new()),
        }
    }

    pub async fn send(&mut self, frame: Frame) -> Result<(), NetworkCodecError> {
        self.framed.send(frame).await
    }

    pub async fn recv(&mut self) -> Result<Option<Frame>, NetworkCodecError> {
        self.framed.next().await.transpose()
    }

    pub fn into_split(
        self,
    ) -> (
        futures::stream::SplitSink<Framed<TcpStream, FrameCodec>, Frame>,
        futures::stream::SplitStream<Framed<TcpStream, FrameCodec>>,
    ) {
        self.framed.split()
    }
}
