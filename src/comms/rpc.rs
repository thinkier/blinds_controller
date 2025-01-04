use blinds_sequencer::WindowDressingState;
use embedded_io::{Read, ReadExactError, ReadReady, Write};
use serde::{Deserialize, Serialize};

pub struct RpcBuffer<const N: usize, IO> {
    pub packet_buf: [u8; N],
    pub serial: IO,
}

pub enum RpcError<E: embedded_io::Error> {
    IoError(E),
    IoReadExactError(ReadExactError<E>),
    ParseError(serde_json_core::de::Error),
    EncodeError(serde_json_core::ser::Error),
}

impl<E: embedded_io::Error> From<E> for RpcError<E> {
    fn from(value: E) -> Self {
        RpcError::IoError(value)
    }
}

impl<E: embedded_io::Error> From<ReadExactError<E>> for RpcError<E> {
    fn from(value: ReadExactError<E>) -> Self {
        RpcError::IoReadExactError(value)
    }
}

impl<const N: usize, IO> RpcBuffer<N, IO>
where
    IO: Read + ReadReady + Write,
{
    pub fn new(serial: IO) -> Self {
        Self {
            packet_buf: [0; N],
            serial,
        }
    }

    pub fn read(&mut self) -> Result<Option<Request>, RpcError<IO::Error>> {
        if self.serial.read_ready()? == false {
            return Ok(None);
        }

        let mut len_buf = [0u8];
        self.serial.read(&mut len_buf)?;
        let len = len_buf[0] as usize;
        self.serial.read_exact(&mut self.packet_buf[0..len])?;
        Ok(Some(
            serde_json_core::from_slice(&mut self.packet_buf[0..len])
                .map_err(|e| RpcError::ParseError(e))?
                .0,
        ))
    }

    pub fn write(&mut self, resp: &Response) -> Result<(), RpcError<IO::Error>> {
        let packet = serde_json_core::to_slice(resp, &mut self.packet_buf)
            .map_err(|e| RpcError::EncodeError(e))?;
        self.serial.write_all(&self.packet_buf[0..packet])?;

        Ok(())
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
