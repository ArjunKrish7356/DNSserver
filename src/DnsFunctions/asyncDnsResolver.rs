mod Dnspacket;
mod DnsErrors;

use Dnspacket::DnsRecord;
use tokio::net::UdpSocket;
use std::{net::{Ipv4Addr,IpAddr}, sync::Arc};
use DnsErrors::DnsResolverError;

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
        print!("\n");
    }
}

async fn handle_request(
    buffer: Dnspacket::BytePacketBuffer,
    src: std::net::SocketAddr,
    in_socket: Arc<UdpSocket>,
    out_socket: Arc<UdpSocket>,
) -> Result<(), Box<dyn std::error::Error>> {
    let response_buffer = recursive_resolver(out_socket,buffer).await?;
    in_socket.send_to(&response_buffer.buf[0..response_buffer.pos], src).await?;
    Ok(())
}

async fn recursive_resolver(
    out_socket: Arc<UdpSocket>,
    buffer: Dnspacket::BytePacketBuffer,
) -> Result<Dnspacket::BytePacketBuffer,DnsResolverError> {
    let mut buffer = buffer;
    let mut packet = Dnspacket::DnsPacket::from_buffer(&mut buffer)
        .map_err(|e|DnsResolverError::ParseError(e.to_string()))?;

    let domain = packet.questions.get(0)
        .map(|q| &q.name)
        .ok_or(DnsResolverError::NoQuestionFound)?;

    let mut ip = String::from("192.203.230.10");

    for _ in 0..13 {
        let (ns_domain, ns_ip ) = fetch_ns(&buffer, &ip, &out_socket).await?;

        if ns_domain == *domain{
            break;
        }
        ip = ns_ip.to_string();
    }

    let answer_record = DnsRecord::A {
        domain: domain.clone(),
        addr: ip.parse::<Ipv4Addr>().unwrap(),
        ttl: 1000,
    };
    packet.answers.push(answer_record);
    let mut response_buffer = Dnspacket::BytePacketBuffer::new();
    packet.write(&mut response_buffer).unwrap();
    Ok(response_buffer)
}

async fn fetch_ns(
    buffer: &Dnspacket::BytePacketBuffer,
    ip_addr: &str,
    out_socket: &Arc<UdpSocket>,
) -> Result<(String,Ipv4Addr),DnsResolverError> {
    let root_server = (IpAddr::V4(ip_addr.parse::<Ipv4Addr>().unwrap()), 53);

    out_socket
        .send_to(
            &buffer.buf[0..buffer.pos],
            root_server,
        )
        .await
        .map_err(|e| DnsResolverError::NetworkError(e))?;

    let mut root_answer_buffer = Dnspacket::BytePacketBuffer::new();
    let (size, _) = out_socket.recv_from(&mut root_answer_buffer.buf).await
        .map_err(|e| DnsResolverError::NetworkError(e))?;

    let packet2 = Dnspacket::DnsPacket::from_buffer(&mut root_answer_buffer).unwrap();
    if packet2.header.answers != 0 {
        return random_ns(&packet2.answers);
    }
    let ns = random_ns(&packet2.resources)?;
    Ok(ns)
}
fn random_ns<'a>(list: &'a Vec<Dnspacket::DnsRecord>) -> Result<(String, Ipv4Addr),DnsResolverError> {
    for record in list {
        match record {
            Dnspacket::DnsRecord::A { domain, addr, ttl } => return Ok((domain.clone(), *addr)),
            _ => (),
        }
    }
    Err(DnsResolverError::NoNameserverFound)
}
