use std::convert::TryFrom;
use std::io::{self, Read};
use std::net::TcpStream;

use crate::config::HEADER_SIZE;

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum OpCode {
    Hello = 0x01,       // Worker -> Controller: "Ready"
    TaskDef = 0x02,     // Controller -> Worker: "Compile this file path"
    TaskResult = 0x03,  // Worker -> Controller: "Success/Fail + Output"
    SubmitFile = 0x04,  // Client -> Server: "Here's a .c file to compile"
    FileResult = 0x05,  // Server -> Client: "Here's your .o file"
    Shutdown = 0xFF,    // Controller -> Worker: "Exit"
}

impl TryFrom<u8> for OpCode {
    type Error = ();
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0x01 => Ok(OpCode::Hello),
            0x02 => Ok(OpCode::TaskDef),
            0x03 => Ok(OpCode::TaskResult),
            0x04 => Ok(OpCode::SubmitFile),
            0x05 => Ok(OpCode::FileResult),
            0xFF => Ok(OpCode::Shutdown),
            _ => Err(()),
        }
    }
}

pub struct Message {
    pub op: OpCode,
    pub payload: Vec<u8>,
}

impl Message {
    pub fn new(op: OpCode, payload: Vec<u8>) -> Self {
        Self { op, payload }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.op as u8);
        buf.extend_from_slice(&(self.payload.len() as u32).to_be_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }

    pub fn read(stream: &mut TcpStream) -> io::Result<Message> {
        let mut header = [0u8; HEADER_SIZE];
        stream.read_exact(&mut header)?;

        let op = OpCode::try_from(header[0])
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid Op"))?;
        let len = u32::from_be_bytes(header[1..5].try_into().unwrap()) as usize;

        let mut payload = vec![0u8; len];
        stream.read_exact(&mut payload)?;

        Ok(Message { op, payload })
    }
}
