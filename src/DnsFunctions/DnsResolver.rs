mod Dnspacket;
mod DnsErrors;

use DnsErrors::DnsResolverError;
use std::net::{IpAddr, Ipv4Addr,UdpSocket};
use Dnspacket::DnsRecord;


fn recursive_resolver(
    buffer: Dnspacket::BytePacketBuffer,
) -> Result<Dnspacket::BytePacketBuffer,DnsResolverError> {
    let mut buffer = buffer;
    let mut packet = Dnspacket::DnsPacket::from_buffer(&mut buffer)
        .map_err(|e|DnsResolverError::ParseError(e.to_string()))?;

    let domain = packet.questions.get(0)
        .map(|q| &q.name)
        .ok_or(DnsResolverError::NoQuestionFound)?;

    let mut ip = String::from("192.203.230.10");
    let out_socket = UdpSocket::bind("0.0.0.0:0")?;

    for _ in 0..13 {
        let (ns_domain, ns_ip ) = fetch_ns(&buffer, &ip, &out_socket)
            .ok_or(DnsResolverError::NoNameserverFound)?;

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

fn fetch_ns(
    buffer: &Dnspacket::BytePacketBuffer,
    ip_addr: &str,
    out_socket: &UdpSocket,
) -> Option<(String, Ipv4Addr)> {
    let root_server = (IpAddr::V4(ip_addr.parse::<Ipv4Addr>().unwrap()), 53);

    out_socket
        .send_to(
            &buffer.buf[0..buffer.pos], // Later adjust to send only relevant part
            root_server,
        )
        .unwrap();

    let mut root_answer_buffer = Dnspacket::BytePacketBuffer::new();
    let (size, _) = out_socket.recv_from(&mut root_answer_buffer.buf).unwrap();

    let packet2 = Dnspacket::DnsPacket::from_buffer(&mut root_answer_buffer).unwrap();
    if packet2.header.answers != 0 {
        return random_ns(&packet2.answers);
    }
    let ns = random_ns(&packet2.resources)?;
    Some(ns)
}

fn random_ns<'a>(list: &'a Vec<Dnspacket::DnsRecord>) -> Option<(String, Ipv4Addr)> {
    for record in list {
        match record {
            Dnspacket::DnsRecord::A { domain, addr, ttl } => return Some((domain.clone(), *addr)),
            _ => (),
        }
    }
    None
}

fn main() -> Result<(), DnsResolverError> {
    let in_socket = UdpSocket::bind("127.0.0.1:2054")?;

    let mut root_query_buffer = Dnspacket::BytePacketBuffer::new();

    let (_, src) = in_socket.recv_from(&mut root_query_buffer.buf)?;
    
    let mut buffer = recursive_resolver(root_query_buffer.clone())?;

    in_socket.send_to(&buffer.buf[0..buffer.pos], src)?;

    Ok(())
}