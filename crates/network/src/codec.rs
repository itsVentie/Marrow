use async_trait::async_trait;
use futures::prelude::*;
use libp2p::request_response::Codec;
use std::io;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarrowProtocol;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarrowRequest(pub Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarrowResponse(pub Vec<u8>);

#[derive(Clone, Default)]
pub struct MarrowCodec;

#[async_trait]
impl Codec for MarrowCodec {
    type Protocol = MarrowProtocol;
    type Request = MarrowRequest;
    type Response = MarrowResponse;

    async fn read_request<T>(&mut self, _: &MarrowProtocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut len_buf = [0u8; 4];
        io.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > 10 * 1024 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Frame exceeds size limit",
            ));
        }

        let mut buf = vec![0u8; len];
        io.read_exact(&mut buf).await?;
        Ok(MarrowRequest(buf))
    }

    async fn read_response<T>(&mut self, _: &MarrowProtocol, io: &mut T) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut len_buf = [0u8; 4];
        io.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > 10 * 1024 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Frame exceeds size limit",
            ));
        }

        let mut buf = vec![0u8; len];
        io.read_exact(&mut buf).await?;
        Ok(MarrowResponse(buf))
    }

    async fn write_request<T>(
        &mut self,
        _: &MarrowProtocol,
        io: &mut T,
        MarrowRequest(data): Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let len = data.len() as u32;
        io.write_all(&len.to_be_bytes()).await?;
        io.write_all(&data).await?;
        io.flush().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &MarrowProtocol,
        io: &mut T,
        MarrowResponse(data): Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let len = data.len() as u32;
        io.write_all(&len.to_be_bytes()).await?;
        io.write_all(&data).await?;
        io.flush().await?;
        Ok(())
    }
}

impl AsRef<str> for MarrowProtocol {
    fn as_ref(&self) -> &str {
        "/marrow/p2p/1.0.0"
    }
}
