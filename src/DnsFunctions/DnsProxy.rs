use std::net::UdpSocket;
use Dnspacket::{BytePacketBuffer, DnsPacket};
//
//bind socket to port 2053 using UDP protocol
//accept packet from the user through the port
//parse the packet to lookup fn
//give the returned answer back to the user
//error handling

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("127.0.0.1:2053")?;

    let mut buf = BytePacketBuffer::new();
    let (no_bytes, src_address) = socket.recv_from(&mut buf.buf)?;
    let packet = DnsPacket::from_buffer(&mut buf);

    println!("{:#?} ", packet);
    Ok(())
}
