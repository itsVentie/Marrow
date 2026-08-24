use r_network::FrameTransport;
use r_protocol::Frame;
use tokio::net::{TcpListener, TcpStream};

#[tokio::test]
async fn test_framed_transport_loopback() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_handle = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let mut transport = FrameTransport::new(socket);

        if let Some(Frame::Ping) = transport.recv().await.unwrap() {
            transport.send(Frame::Pong).await.unwrap();
        }
    });

    let client_socket = TcpStream::connect(addr).await.unwrap();
    let mut client_transport = FrameTransport::new(client_socket);

    client_transport.send(Frame::Ping).await.unwrap();
    let response = client_transport.recv().await.unwrap();

    assert_eq!(response, Some(Frame::Pong));
    server_handle.await.unwrap();
}
