
pub struct BytePacketBuffer{
    pub buf:[u8;512],
    pub pos:usize
}

impl BytePacketBuffer{
    pub fn new()->BytePacketBuffer{
        BytePacketBuffer{
            buf:[0;512],
            pos:0
        }
    }

    pub fn pos(&self)->usize{
        self.pos        
    }

    fn step(&mut self, steps: usize) -> Result<()> {
        self.pos += steps;

        Ok(())
    }

    fn seek(&mut self, pos: usize) -> Result<()> {
        self.pos = pos;

        Ok(())
    }

    pub fn read(&mut self)->Result<u8>{
        if self.pos>512{
            return Err("End of buffer".into());
        }
        let res = self.buf[self.pos];
        self.pos+=1;

        Ok(res)
    }

    pub fn get(&self,pos:usize)->Result<u8>{
        if pos >= 512 {
            return Err("End of buffer".into());
        }
        Ok(self.buf[pos])
    }

    pub fn get_range(&self , start :usize, len:usize)->Result<&[u8]>{
        if start + len >= 512 {
            return Err("End of buffer".into());
        }
        Ok(&self.buf[start..start+len as usize])
    }

    pub fn read_u16(&mut self)->Result<u16>{
        let res= ((self.read()? as u16)<<8) | (self.read() as  u16);
        Ok(res)        
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let res = ((self.read()? as u32) << 24)
            | ((self.read()? as u32) << 16)
            | ((self.read()? as u32) << 8)
            | ((self.read()? as u32) << 0);

        Ok(res)
    }

    fn read_qname(&mut self,outstr:&mut String)->Result<()>{
        let mut pos=self.pos();
        let mut delim="";
        
        let mut jumped = false;            //unaudited part
        let max_jumps = 5;
        let mut jumps_performed = 0;

        loop{
            if jumps_performed > max_jumps {
                return Err(format!("Limit of {} jumps exceeded", max_jumps).into());
            }

            let len = self.get(pos)?;

            if (len & 0xC0) == 0xC0 {                 //unaudited part
                // Update the buffer position to a point past the current
                // label. We don't need to touch it any further.
                if !jumped {
                    self.seek(pos + 2)?;
                }

                // Read another byte, calculate offset and perform the jump by
                // updating our local position variable
                let b2 = self.get(pos + 1)? as u16;
                let offset = (((len as u16) ^ 0xC0) << 8) | b2;
                pos = offset as usize;

                // Indicate that a jump was performed.
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
                let str_buffer= self.get_range(pos,len as usize);
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

#[derive(Clone, Debug)]
pub struct DnsHeader {
    pub id: u16, // 16 bits

    pub recursion_desired: bool,    // 1 bit
    pub truncated_message: bool,    // 1 bit
    pub authoritative_answer: bool, // 1 bit
    pub opcode: u8,                 // 4 bits
    pub response: bool,             // 1 bit

    pub rescode: ResultCode,       // 4 bits
    pub checking_disabled: bool,   // 1 bit
    pub authed_data: bool,         // 1 bit
    pub z: bool,                   // 1 bit
    pub recursion_available: bool, // 1 bit

    pub questions: u16,             // 16 bits
    pub answers: u16,               // 16 bits
    pub authoritative_entries: u16, // 16 bits
    pub resource_entries: u16,      // 16 bits
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
    pub fn read(&mut self,buffer:&mut BytePacketBuffer)->Result<()>{
        self.id=buffer.read_u16()?;
        let flags = buffer.read_u16()?;
        let a= (flags >> 8)as u8;
        let b = (flags & 0xFF) as u8;


        Ok(())
    }
}







fn main(){
    println!("Helooooo");
}