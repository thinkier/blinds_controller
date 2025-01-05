use blinds_sequencer::WindowDressingState;
use defmt::*;
use embedded_io::{Read, ReadExactError, ReadReady, Write};
use serde::{Deserialize, Serialize};

pub struct RpcHandle<const N: usize, IO> {
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

impl<E: embedded_io::Error + Format> Format for RpcError<E> {
    fn format(&self, fmt: Formatter) {
        match self {
            RpcError::IoError(e) => defmt::write!(fmt, "IoError({:?})", e),
            RpcError::IoReadExactError(e) => defmt::write!(fmt, "IoReadExactError({:?})", e),
            RpcError::ParseError(e) => defmt::write!(fmt, "ParseError({:?})", e),
            RpcError::EncodeError(e) => defmt::write!(fmt, "EncodeError({:?})", e),
        }
    }
}

impl<const N: usize, IO> RpcHandle<N, IO>
where
    IO: Read + ReadReady + Write,
{
    pub fn new(serial: IO) -> Self {
        Self {
            packet_buf: [0; N],
            serial,
        }
    }

    pub fn read(&mut self) -> Result<Option<RpcPacket>, RpcError<IO::Error>> {
        if self.serial.read_ready()? == false {
            return Ok(None);
        }

        let mut len_buf = [0u8];
        self.serial.read(&mut len_buf)?;
        let len = len_buf[0] as usize;
        self.serial.read_exact(&mut self.packet_buf[0..len])?;
        let packet = serde_json_core::from_slice(&mut self.packet_buf[0..len])
            .map_err(|e| RpcError::ParseError(e))?
            .0;

        Ok(Some(packet))
    }

    pub fn write(&mut self, resp: &RpcPacket) -> Result<(), RpcError<IO::Error>> {
        let packet = serde_json_core::to_slice(resp, &mut self.packet_buf)
            .map_err(|e| RpcError::EncodeError(e))?;
        self.serial.write_all(&self.packet_buf[0..packet])?;

        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub enum RpcPacket {
    Home {
        channel: u8,
    },
    Setup {
        channel: u8,
        init: WindowDressingState,
        full_cycle_steps: u32,
        full_tilt_steps: Option<u32>,
    },
    Position {
        channel: u8,
        state: WindowDressingState,
    },
    GetPosition {
        channel: u8,
    },
}
