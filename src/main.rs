use std::net::{Ipv4Addr, UdpSocket};
use std::fs::File;
use std::io::{ Read,Error as E,ErrorKind};


pub struct BytePacketBuffer{
    pub buf:[u8;512],
    pub pos:usize
}

impl BytePacketBuffer{
    pub fn new()->BytePacketBuffer {
        BytePacketBuffer{
            buf:[0;512],
            pos:0
        }
    }

    pub fn pos(&self)->usize{
        self.pos        
    }

    fn step(&mut self, steps: usize) -> Result<(),E> {
        self.pos += steps;

        Ok(())
    }

    fn seek(&mut self, pos: usize) -> Result<(),E> {
        self.pos = pos;

        Ok(())
    }

    pub fn read(&mut self)->Result<u8,E>{
        if self.pos>=512{
            return Err(E::new(ErrorKind::InvalidInput, "End of buffer"));
        }
        let res = self.buf[self.pos];
        self.pos+=1;

        Ok(res)
    }

    pub fn get(&self,pos:usize)->Result<u8,E>{
        if pos >= 512 {
            return Err(E::new(ErrorKind::InvalidInput, "End of buffer"));
        }
        Ok(self.buf[pos])
    }

    pub fn get_range(&self , start :usize, len:usize)->Result<&[u8],E>{
        if start + len >= 512 {
            return Err(E::new(ErrorKind::InvalidInput, "End of buffer"));
        }
        Ok(&self.buf[start..start+len as usize])
    }

    pub fn read_u16(&mut self)->Result<u16,E>{
        let res= ((self.read()? as u16)<<8) | (self.read()? as  u16);
        Ok(res)        
    }

    pub fn read_u32(&mut self) -> Result<u32,E> {
        let res = ((self.read()? as u32) << 24)
            | ((self.read()? as u32) << 16)
            | ((self.read()? as u32) << 8)
            | ((self.read()? as u32) << 0);

        Ok(res)
    }

    fn read_qname(&mut self,outstr:&mut String)->Result<(),E>{
        let mut pos=self.pos();
        let mut delim="";
        
        let mut jumped = false;            //unaudited part
        let max_jumps = 5;
        let mut jumps_performed = 0;

        loop{
            if jumps_performed > max_jumps {
                return Err(E::new(ErrorKind::InvalidInput, "Limit of jumps exceeded"));
            }

            let len = self.get(pos)?;

            if (len & 0xC0) == 0xC0 {                 
                if !jumped {
                    self.seek(pos + 2)?;
                }

                
                let b2 = self.get(pos + 1)? as u16;
                let offset = (((len as u16) ^ 0xC0) << 8) | b2;
                pos = offset as usize;

                jumped = true;
                jumps_performed += 1;

                continue;
            }
            else{
                pos+=1;

                if len==0{
                    break;
                }

                outstr.push_str(delim);
                let str_buffer= self.get_range(pos,len as usize)?;
                outstr.push_str(&String::from_utf8_lossy(str_buffer).to_lowercase());
                delim=".";

                pos+=len as usize;
                
            }
        }
        if !jumped {
            self.seek(pos)?;
        }

        Ok(())
    }

    fn write(&mut self, val :u8) -> Result<(), E> {
        if self.pos >= 512{
            return Err(E::new(ErrorKind::InvalidInput, "Limit of jumps exceeded"));
        }
        self.buf[self.pos] = val;
        self.pos += 1;
        Ok(())
    }

    fn write_u8(&mut self , val: u8) -> Result<(),E> {
        self.write(val)?;
        Ok(())
    }

    fn write_u16(&mut self , val: u16) -> Result<(),E> {
        self.write((val >> 8) as u8)?;
        self.write((val & 0xFF) as u8)?;
        Ok(())
    }

    fn write_u32(&mut self, val: u32) -> Result<(),E> {
        self.write((val >> 24) as u8)?;
        self.write((val >> 16) as u8)?;
        self.write((val >> 8) as u8)?;
        self.write((val >> 0) as u8)?;

        Ok(())
    }

    fn write_qname(&mut self, qname: &str) -> Result<(),E> {
        for label in qname.split('.') {
            let len = label.len();
            if len > 0x3f {
                return Err(E::new(ErrorKind::InvalidInput, "Single label exceeds 63 characters of length"));
            }

            self.write_u8(len as u8)?;
            for b in label.as_bytes() {
                self.write_u8(*b)?;
            }
        }
            self.write_u8(0)?;

        Ok(())
    }

}


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ResultCode{
    NOERROR = 0,
    FORMERR = 1,
    SERVFAIL = 2,
    NXDOMAIN = 3,
    NOTIMP = 4,
    REFUSED = 5,
}

impl ResultCode{
    pub fn from_num(num:u8)->ResultCode{
        match num {
            1=>ResultCode::FORMERR,
            2 => ResultCode::SERVFAIL,
            3 => ResultCode::NXDOMAIN,
            4 => ResultCode::NOTIMP,
            5 => ResultCode::REFUSED,
            0 | _ => ResultCode::NOERROR,
        }
    }
}

//<------------------------------ DNS header -------------------------------------->

#[derive(Clone, Debug)]
pub struct DnsHeader {
    pub id: u16, // 16 bits

    pub recursion_desired: bool,    
    pub truncated_message: bool,    
    pub authoritative_answer: bool, 
    pub opcode: u8,                 
    pub response: bool,             

    pub rescode: ResultCode,       
    pub checking_disabled: bool,   
    pub authed_data: bool,         
    pub z: bool,                   
    pub recursion_available: bool, 

    pub questions: u16,             
    pub answers: u16,               
    pub authoritative_entries: u16, 
    pub resource_entries: u16,      
}

impl DnsHeader{
    pub fn new()->DnsHeader{
        DnsHeader {
            id: 0,

            recursion_desired: false,
            truncated_message: false,
            authoritative_answer: false,
            opcode: 0,
            response: false,

            rescode: ResultCode::NOERROR,
            checking_disabled: false,
            authed_data: false,
            z: false,
            recursion_available: false,

            questions: 0,
            answers: 0,
            authoritative_entries: 0,
            resource_entries: 0,
        }
    }
    pub fn read(&mut self,buffer:&mut BytePacketBuffer)->Result<(),E>{
        self.id=buffer.read_u16()?;
        let flags = buffer.read_u16()?;
        let a= (flags >> 8)as u8;
        let b = (flags & 0xFF) as u8;
        self.recursion_desired=(a & (1 <<0 )) > 0;
        self.truncated_message = (a & (1 << 1)) > 0;
        self.authoritative_answer = (a & (1 << 2)) > 0;
        self.opcode = (a >> 3) & 0x0F;
        self.response = (a & (1 << 7)) > 0;

        self.rescode=ResultCode::from_num(b & 0x0F);
        self.checking_disabled = (b & (1 << 4)) > 0;
        self.authed_data = (b & (1 << 5)) > 0;
        self.z = (b & (1 << 6)) > 0;
        self.recursion_available = (b & (1 << 7)) > 0;

        self.questions = buffer.read_u16()?;
        self.answers = buffer.read_u16()?;
        self.authoritative_entries = buffer.read_u16()?;
        self.resource_entries = buffer.read_u16()?;


        Ok(())
    }

    fn write(&mut self, buffer: &mut BytePacketBuffer) -> Result<(),E> {
        buffer.write_u16(self.id)?;

        buffer.write_u8(
            (self.recursion_desired as u8)
                | ((self.truncated_message as u8) << 1)
                | ((self.authoritative_answer as u8) << 2)
                | (self.opcode << 3)
                | ((self.response as u8) << 7) as u8,
        )?;

        buffer.write_u8(
            (self.rescode as u8)
                | ((self.checking_disabled as u8) << 4)
                | ((self.authed_data as u8) << 5)
                | ((self.z as u8) << 6)
                | ((self.recursion_available as u8) << 7),
        )?;     

        buffer.write_u16(self.questions)?;
        buffer.write_u16(self.answers)?;
        buffer.write_u16(self.authoritative_entries)?;
        buffer.write_u16(self.resource_entries)?;

        Ok(())
    }
}

//<-------------------------- Query Type ---------------------------------------------------------> 

#[derive(PartialEq, Eq, Debug, Clone, Hash, Copy)]
pub enum QueryType {
    UNKNOWN(u16),
    A, 
}

impl QueryType {
    pub fn to_num(&self) -> u16 {
        match *self {
            QueryType::UNKNOWN(x) => x,
            QueryType::A => 1,
        }
    }

    pub fn from_num(num: u16) -> QueryType {
        match num {
            1 => QueryType::A,
            _ => QueryType::UNKNOWN(num),
        }
    }
}

//<-------------------------------------- DNS Question --------------------------------------------------->

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct DnsQuestion{
    pub name : String,
    pub qtype : QueryType 
}

impl DnsQuestion{
    pub fn new(name :String , qtype :QueryType)->DnsQuestion{
        DnsQuestion{
            name : name,
            qtype : qtype
        }
    }

    pub fn read(&mut self, buffer :&mut BytePacketBuffer)->Result<(),E>{
        buffer.read_qname(&mut self.name)?;
        self.qtype=QueryType::from_num(buffer.read_u16()?);
        let _ = buffer.read_u16()?;

        Ok(())
    }

    pub fn write(&mut self, buffer: &mut BytePacketBuffer) -> Result<(),E> {
        buffer.write_qname(&self.name)?;

        let typenum = self.qtype.to_num();
        buffer.write_u16(typenum)?;
        buffer.write_u16(1)?;

        Ok(())        
    }
}

//<----------------------------------- Dns Record --------------------------------------->

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(dead_code)]

pub enum DnsRecord{
    Unknown{
        domain: String,
        qtype: u16,
        data_len: u16,
        ttl: u32,
    },
    A {
        domain: String,
        addr: Ipv4Addr,
        ttl: u32,
    }
}

impl DnsRecord{
    pub fn read(buffer : &mut BytePacketBuffer)->Result<DnsRecord,E>{
        let mut domain =String::new();
        buffer.read_qname(&mut domain)?;

        let qtype_num= buffer.read_u16()?;
        let qtype =QueryType::from_num(qtype_num);
        let _ = buffer.read_u16();
        let ttl =buffer.read_u32()?;
        let data_len=buffer.read_u16()?;

        match qtype{
            QueryType::A => {
                let raw_addr = buffer.read_u32()?;
                let addr = Ipv4Addr::new(
                    ((raw_addr >> 24 ) & 0xFF) as u8,
                    ((raw_addr >> 16) & 0xFF) as u8,
                    ((raw_addr >> 8) & 0xFF) as u8,
                    ((raw_addr >> 0) & 0xFF) as u8,
                );
                Ok(DnsRecord::A {
                    domain : domain,
                    addr : addr,
                    ttl : ttl,
                 })
            }
            QueryType::UNKNOWN(_)=>{
                buffer.step(data_len as usize);
                Ok(DnsRecord::Unknown{
                    domain: domain,
                    qtype: qtype_num,
                    data_len: data_len,
                    ttl: ttl,
                })
            }
        }
    }

    pub fn write(&mut self, buffer: &mut BytePacketBuffer) -> Result<usize,E> {
        let start_pos = buffer.pos();

        match *self {
            DnsRecord::A {
                ref domain,
                ref addr , ttl
            } => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QueryType::A.to_num())?;
                buffer.write_u16(1)?;
                buffer.write_u32(ttl)?;
                buffer.write_u16(4)?;

                let octets = addr.octets();
                buffer.write_u8(octets[0])?;
                buffer.write_u8(octets[1])?;
                buffer.write_u8(octets[2])?;
                buffer.write_u8(octets[3])?;                
            }
            DnsRecord::Unknown { 
                ..
            } => {
                println!("Skipping record: {:?}",self)
            }
        }

        
        Ok(buffer.pos() - start_pos)
    }
}
//<------------------------------------ Dns Packet ---------------------------------------->

#[derive(Clone,Debug)]
pub struct DnsPacket{
    pub header : DnsHeader,
    pub questions: Vec<DnsQuestion>,
    pub answers: Vec<DnsRecord>,
    pub authorities: Vec<DnsRecord>,
    pub resources: Vec<DnsRecord>,
}

impl  DnsPacket {
    pub fn new()->DnsPacket{
        DnsPacket{
            header : DnsHeader::new(),
            questions : Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            resources: Vec::new(),
        }      
    } 

    pub fn from_buffer(buffer :  &mut BytePacketBuffer ) -> Result<DnsPacket,E>{
        let mut result = DnsPacket::new();
        result.header.read(buffer);

        for _ in 0..result.header.questions{
            let mut question = DnsQuestion::new("".to_string(),QueryType::UNKNOWN(0));
            question.read(buffer)?;
            result.questions.push(question);
        }

        for _ in 0..result.header.answers{
            let rec = DnsRecord::read(buffer)?;
            result.answers.push(rec);
        }

        for _ in 0..result.header.authoritative_entries {
            let rec = DnsRecord::read(buffer)?;
            result.authorities.push(rec);
        }
        for _ in 0..result.header.resource_entries {
            let rec = DnsRecord::read(buffer)?;
            result.resources.push(rec);
        } 

        Ok(result)
    }

    fn write(&mut self,buffer :  &mut BytePacketBuffer) -> Result<(),E> {
        self.header.questions = self.questions.len() as u16;
        self.header.answers = self.answers.len() as u16;
        self.header.authoritative_entries = self.authorities.len() as u16;
        self.header.resource_entries = self.resources.len() as u16;

        self.header.write(buffer)?;

        for question in &mut self.questions {
            question.write(buffer)?;
        }

        for rec in &mut self.answers {
            rec.write(buffer)?;
        }
        for rec in &mut self.authorities {
            rec.write(buffer)?;
        }
        for rec in &mut self.resources {
            rec.write(buffer)?;
        }

        Ok(())
    }
}

//<-------------------------- Final part ----------------------------------------------->fn main() -> Result<()> {
fn main() -> Result<(),E> {

    let qname = "google.com";
    let qtype = QueryType::A;

    let server = ("8.8.8.8", 53);

    let socket = UdpSocket::bind(("0.0.0.0", 43210))?;

    let mut packet = DnsPacket::new();

    packet.header.id = 6666;
    packet.header.questions = 1;
    packet.header.recursion_desired = true;
    packet.questions.push(DnsQuestion::new(qname.to_string(), qtype));

    let mut req_buffer = BytePacketBuffer::new();
    packet.write(&mut req_buffer)?;

    socket.send_to(&req_buffer.buf[0..req_buffer.pos], server)?;

    let mut res_buffer = BytePacketBuffer::new();
    socket.recv_from(&mut res_buffer.buf)?;

    let res_packet = DnsPacket::from_buffer(&mut res_buffer)?;
    println!("{:#?}", res_packet.header);

    for q in res_packet.questions {
        println!("{:#?}", q);
    }
    for rec in res_packet.answers {
        println!("{:#?}", rec);
    }
    for rec in res_packet.authorities {
        println!("{:#?}", rec);
    }
    for rec in res_packet.resources {
        println!("{:#?}", rec);
    }

    Ok(())
}