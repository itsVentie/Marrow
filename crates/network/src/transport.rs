use crate::codec::{FrameCodec, NetworkCodecError};
use futures::{SinkExt, StreamExt};
use r_protocol::Frame;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub type TxStream = futures::stream::SplitSink<Framed<TcpStream, FrameCodec>, Frame>;
pub type RxStream = futures::stream::SplitStream<Framed<TcpStream, FrameCodec>>;

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

    pub fn into_split(self) -> (TxStream, RxStream) {
        self.framed.split()
    }
}
