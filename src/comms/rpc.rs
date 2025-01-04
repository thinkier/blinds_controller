use blinds_sequencer::WindowDressingState;
use embedded_io::{Read, Write};
use serde::{Deserialize, Serialize};

pub struct RpcBuffer<const N: usize, IO> {
    pub out_buf: [u8; N],
    pub in_buf: [u8; N],
    pub serial: IO,
}

impl<const N: usize, IO> RpcBuffer<N, IO>
where
    IO: Read + Write,
{
    pub fn new(serial: IO) -> Self {
        Self {
            out_buf: [0; N],
            in_buf: [0; N],
            serial,
        }
    }

    pub fn read(&mut self) -> Option<Request> {
        // self.serial.read()
        unimplemented!()
    }
}

#[derive(Deserialize, Serialize)]
pub enum Request {
    Home {
        channel: u8,
    },
    SetPosition {
        channel: u8,
        state: WindowDressingState,
    },
    GetPosition {
        channel: u8,
    },
}

#[derive(Serialize)]
pub enum Response {
    Position {
        channel: u8,
        state: WindowDressingState,
    },
}
