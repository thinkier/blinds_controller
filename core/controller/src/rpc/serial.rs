use crate::rpc::{AsyncRpc, IncomingRpcPacket, OutgoingRpcPacket};
use cortex_m::peripheral::SCB;
use defmt::*;
use embedded_io::{ErrorType, Read, ReadExactError, ReadReady, Write};

pub struct SerialRpcHandle<const N: usize, IO> {
    pub packet_buf: [u8; N],
    pub serial: IO,
}

pub enum SerialRpcError<E: embedded_io::Error> {
    IoError(E),
    IoReadExactError(ReadExactError<E>),
    ParseError(serde_json_core::de::Error),
    EncodeError(serde_json_core::ser::Error),
}

impl<E: embedded_io::Error> From<E> for SerialRpcError<E> {
    fn from(value: E) -> Self {
        SerialRpcError::IoError(value)
    }
}

impl<E: embedded_io::Error> From<ReadExactError<E>> for SerialRpcError<E> {
    fn from(value: ReadExactError<E>) -> Self {
        SerialRpcError::IoReadExactError(value)
    }
}

impl<E: embedded_io::Error + Format> Format for SerialRpcError<E> {
    fn format(&self, fmt: Formatter) {
        match self {
            SerialRpcError::IoError(e) => defmt::write!(fmt, "IoError({:?})", e),
            SerialRpcError::IoReadExactError(e) => defmt::write!(fmt, "IoReadExactError({:?})", e),
            SerialRpcError::ParseError(e) => defmt::write!(fmt, "ParseError({:?})", e),
            SerialRpcError::EncodeError(e) => defmt::write!(fmt, "EncodeError({:?})", e),
        }
    }
}

impl<const N: usize, IO> SerialRpcHandle<N, IO>
where
    IO: Read + ReadReady + Write,
{
    pub fn new(serial: IO) -> Self {
        Self {
            packet_buf: [0; N],
            serial,
        }
    }
}
impl<const N: usize, IO> AsyncRpc for SerialRpcHandle<N, IO>
where
    IO: Read + ReadReady + Write,
    <IO as ErrorType>::Error: defmt::Format,
{
    type Error = SerialRpcError<IO::Error>;

    async fn read(&mut self) -> Result<Option<IncomingRpcPacket>, Self::Error> {
        if self.serial.read_ready()? == false {
            return Ok(None);
        }

        let mut len_buf = [0u8];
        self.serial.read(&mut len_buf)?;
        let len = len_buf[0] as usize;
        if len == 0 {
            SCB::sys_reset();
        }
        self.serial.read_exact(&mut self.packet_buf[0..len])?;
        let packet = serde_json_core::from_slice(&mut self.packet_buf[0..len])
            .map_err(|e| SerialRpcError::ParseError(e))?
            .0;

        Ok(Some(packet))
    }

    async fn write(&mut self, resp: &OutgoingRpcPacket) -> Result<(), Self::Error> {
        let packet = serde_json_core::to_slice(resp, &mut self.packet_buf[1..])
            .map_err(|e| SerialRpcError::EncodeError(e))?;
        self.packet_buf[0] = packet as u8 + 2;
        // CRLF ensures that minicom will display the packet correctly
        self.packet_buf[packet + 1] = b'\r';
        self.packet_buf[packet + 2] = b'\n';

        self.serial.write_all(&self.packet_buf[0..packet + 3])?;

        Ok(())
    }
}
