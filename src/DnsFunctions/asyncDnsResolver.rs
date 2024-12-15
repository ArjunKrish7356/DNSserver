mod Dnspacket;
use tokio::net::UdpSocket;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let in_socket = Arc::new(UdpSocket::bind("127.0.0.1:2053").await?);
    println!("Listening on {}", in_socket.local_addr()?);

    let out_socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
    println!("Bound to {}", out_socket.local_addr()?);

    let mut root_query_buffer = Dnspacket::BytePacketBuffer::new();

    loop{
        let (size, src) = in_socket.recv_from(&mut root_query_buffer.buf).await?;
        println!("Received {} bytes from {}", size, src);
        
        let buffer = root_query_buffer.clone();
        let out_socket = out_socket.clone();
        let in_socket = in_socket.clone();

        tokio::spawn(async move{
            if let Err(e) = handle_request(buffer, src, in_socket, out_socket).await{
                eprintln!("Error: {}", e);
            };
        });
    }
}

async fn handle_request(
    buffer: Dnspacket::BytePacketBuffer,
    src: std::net::SocketAddr,
    in_socket: Arc<UdpSocket>,
    out_socket: Arc<UdpSocket>,
) -> Result<(), Box<dyn std::error::Error>> {

   Ok(())
}


