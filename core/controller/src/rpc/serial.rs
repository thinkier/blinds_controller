use crate::rpc::SerialRpcError::IoError;
use crate::rpc::{AsyncRpc, IncomingRpcPacket, OutgoingRpcPacket};
use cortex_m::peripheral::SCB;
use defmt::*;
use embedded_io::{ErrorType, Read, ReadExactError, ReadReady, Write};

#[allow(unused)]
pub struct SerialRpcHandle<const N: usize, IO> {
    pub serial: IO,
}

#[allow(unused)]
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
    #[allow(unused)]
    pub fn new(serial: IO) -> Self {
        Self { serial }
    }
}
impl<const N: usize, IO> AsyncRpc for SerialRpcHandle<N, IO>
where
    IO: Read + ReadReady + Write,
    <IO as ErrorType>::Error: defmt::Format,
{
    type Error = SerialRpcError<IO::Error>;

    async fn read(&mut self) -> Result<Option<IncomingRpcPacket>, Self::Error> {
        if !self.serial.read_ready()? {
            return Ok(None);
        }

        let mut i = 0;
        let mut incoming_packet_buf = [0u8; N];

        // Read one byte at a time, looking for special characters.
        // Rather than read normally into this struct's buffer,
        // we try to keep stuff in the upstream buffer.
        //
        // Yes this incurs performance penalty, but I don't want to implement or
        // find a no_alloc deque / circular buffer.
        //
        // It also keeps the memory for the buffer in the stack and not global.
        while i < N {
            self.serial.read_exact(&mut incoming_packet_buf[i..=i])?;

            match incoming_packet_buf[i] {
                0x00 => SCB::sys_reset(),
                b'\n' => {
                    return serde_json_core::from_slice(&mut incoming_packet_buf[0..=i])
                        .map_ok(|(item, _)| item)
                        .map_err(|e| SerialRpcError::ParseError(e));
                }
                _ => i += 1,
            }
        }

        error!("Incoming buffer saturated, discarding serial input until receiving newline...");
        let mut drain = [0u8];
        loop {
            self.serial.read_exact(&mut drain)?;
            match drain[0] {
                0x00 => SCB::sys_reset(),
                b'\n' => break,
            }
        }
        info!("Recovered from buffer saturation, exiting back to caller...");

        return Ok(None);
    }

    async fn write(&mut self, resp: &OutgoingRpcPacket) -> Result<(), Self::Error> {
        let mut outgoing_packet_buf = [0u8; N];

        let packet = serde_json_core::to_slice(resp, &mut outgoing_packet_buf)
            .map_err(|e| SerialRpcError::EncodeError(e))?;
        // LF terminates packet
        outgoing_packet_buf[packet] = b'\n';

        self.serial.write_all(&outgoing_packet_buf[0..=packet])?;

        Ok(())
    }
}
