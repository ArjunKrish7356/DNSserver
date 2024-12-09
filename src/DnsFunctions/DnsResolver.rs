mod Dnspacket;
use std::{net::{IpAddr, Ipv4Addr, UdpSocket,SocketAddr}};
use std::io::Error as E;

use Dnspacket::DnsRecord;

fn test_main() -> Result<(), Box<dyn std::error::Error>>{
    let mut packet = Dnspacket::DnsPacket::new();

    //create a socket and recieve the packet
    let in_socket = UdpSocket::bind("127.0.0.1:2053").unwrap();
    let mut root_query_buffer = Dnspacket::BytePacketBuffer::new();
    let (_, src) = in_socket.recv_from(&mut root_query_buffer.buf).unwrap();
    let mut packet1 = Dnspacket::DnsPacket::from_buffer(&mut root_query_buffer).unwrap();

    println!("stage2");
    let root_server = (
        IpAddr::V4("192.203.230.10".parse::<Ipv4Addr>().unwrap()),
        53,
    );

    let out_socket = UdpSocket::bind("0.0.0.0:0").unwrap(); // Use any available local address and port
    let no = out_socket.send_to(
        &root_query_buffer.buf[0..root_query_buffer.pos],
        root_server,
    ).unwrap();
    //get the data from the root server
    println!("stage3");
    let mut root_answer_buffer = Dnspacket::BytePacketBuffer::new();
    out_socket.recv_from(&mut root_answer_buffer.buf).unwrap();
    let packet2 = Dnspacket::DnsPacket::from_buffer(&mut root_answer_buffer).unwrap();
    let ns = random_ns(&packet2.resources).unwrap();
    println!("{:#?}",ns);
    Ok(())
}


//main fn will listen in the socket out of pc
//when a query comes it gives data to recursive resolver
    //inside a while loop it will call the name server untill the query matches
//rescursive resolver will give back the finished dns packet




fn main() -> Result<(), Box<dyn std::error::Error>>{
     let in_socket = UdpSocket::bind("127.0.0.1:2053").unwrap();
     let mut root_query_buffer = Dnspacket::BytePacketBuffer::new();
   


        let (_,src) = in_socket.recv_from(&mut root_query_buffer.buf).unwrap();
        let buffer = recursive_resolver(src,root_query_buffer.clone()).unwrap();
        in_socket.send_to(&buffer.buf[0..buffer.pos],src);

    Ok(())
}

fn recursive_resolver(src: SocketAddr, buffer: Dnspacket::BytePacketBuffer) -> Option<Dnspacket::BytePacketBuffer> {
    let mut buffer = buffer.clone();
    let mut buffer2 = buffer.clone();
    let packet = Dnspacket::DnsPacket::from_buffer(&mut buffer).unwrap();
    println!("{:#?}", packet);

    let domain = if let Some(Dnspacket::DnsQuestion { name, .. }) = packet.questions.get(0) {
        name
    } else {
        eprintln!("No valid DNS question found at index 0.");
        return None;
    };

    let mut ip = String::from("192.203.230.10");
    let out_socket = UdpSocket::bind("0.0.0.0:0").unwrap(); // Persistent socket    

    for _ in 0..13 {
        let Some(ns_response) = fetch_ns(&buffer, &ip, &out_socket) else {
            eprintln!("Failed to fetch NS response.");
            return None;
        };
        println!("{:#?}", ns_response);
        if ns_response.0 == *domain {
            break;
        }
        ip = ns_response.1.to_string();
    }

    let mut packet = Dnspacket::DnsPacket::from_buffer(&mut buffer2).unwrap();
    let answer_record = DnsRecord::A { domain: domain.clone(), addr: ip.parse::<Ipv4Addr>().unwrap(), ttl: 1000 };
    packet.answers.push(answer_record);
    println!("{:#?} --- {}", packet,src);
    let mut response_buffer = Dnspacket::BytePacketBuffer::new();
    packet.write(&mut response_buffer).unwrap();
    Some(response_buffer)

}

fn fetch_ns<'a>(
    buffer: &'a Dnspacket::BytePacketBuffer,
    ip_addr: &str,
    out_socket: &UdpSocket,
) -> Option<(String, Ipv4Addr)> {
    let root_server = (
        IpAddr::V4(ip_addr.parse::<Ipv4Addr>().unwrap()),
        53,
    );

    out_socket.send_to(
        &buffer.buf[0..buffer.pos], // Later adjust to send only relevant part
        root_server,
    ).unwrap();

    let mut root_answer_buffer = Dnspacket::BytePacketBuffer::new();
    let (size, _) = out_socket.recv_from(&mut root_answer_buffer.buf).unwrap();

    let packet2 = Dnspacket::DnsPacket::from_buffer(&mut root_answer_buffer).unwrap();
    if packet2.header.answers !=0{
        return random_ns( &packet2.answers);
    }
    let ns = random_ns(&packet2.resources)?;
    Some(ns)
}


fn random_ns<'a>(list: &'a Vec<Dnspacket::DnsRecord>) -> Option<(String, Ipv4Addr)> {
    for record in list {
        match record{
            Dnspacket::DnsRecord::A {domain,addr,ttl} => {
                return Some((domain.clone(),*addr))
            }
            _ => ()
        }
    }
    None
}