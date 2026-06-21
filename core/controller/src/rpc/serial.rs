use crate::rpc::{AsyncRpc, IncomingRpcPacket, OutgoingRpcPacket};
use cortex_m::peripheral::SCB;
use defmt::{debug, error, info, write, Format, Formatter};
use embedded_io::{ErrorType, Read, ReadExactError, ReadReady, Write};

/// Trait implementer and wrapper of a text-based port over any simple hardware protocol implementing [`embedded_io`]
///
/// [`N`] should be the size of the buffer allocated in the stack to process a single message
#[allow(unused)]
pub struct SerialRpcHandle<const N: usize, IO> {
    read_buf: Option<IncomingRpcPacket>,
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
            SerialRpcError::IoError(e) => write!(fmt, "IoError({:?})", e),
            SerialRpcError::IoReadExactError(e) => write!(fmt, "IoReadExactError({:?})", e),
            SerialRpcError::ParseError(e) => write!(fmt, "ParseError({:?})", e),
            SerialRpcError::EncodeError(e) => write!(fmt, "EncodeError({:?})", e),
        }
    }
}

impl<const N: usize, IO> SerialRpcHandle<N, IO>
where
    IO: Read + ReadReady + Write,
{
    #[allow(unused)]
    pub fn new(serial: IO) -> Self {
        Self {
            read_buf: None,
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

    async fn peek(&mut self) -> Result<Option<IncomingRpcPacket>, Self::Error> {
        if self.read_buf.is_some() {
            return Ok(self.read_buf.clone());
        }

        self.read_buf = self.read().await?;

        Ok(self.read_buf.clone())
    }

    async fn read(&mut self) -> Result<Option<IncomingRpcPacket>, Self::Error> {
        if self.read_buf.is_some() {
            debug!("Returning cached results immediately");
            return Ok(self.read_buf.take());
        }

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
                        .map(|(item, _)| Some(item))
                        .map_err(|e| {
                            debug!(
                                "Incoming packet resulted in parse error: buf={:02x}",
                                incoming_packet_buf[0..=i]
                            );
                            SerialRpcError::ParseError(e)
                        });
                }
                _ => i += 1,
            }
        }

        error!("Incoming buffer saturated, discarding serial input until seeing newline...");
        let mut drain = [0u8];
        loop {
            self.serial.read_exact(&mut drain)?;
            match drain[0] {
                0x00 => SCB::sys_reset(),
                b'\n' => break,
                _ => {}
            }
        }
        info!("Recovered from buffer saturation, exiting back to caller...");

        Ok(None)
    }

    async fn write(&mut self, resp: &OutgoingRpcPacket) -> Result<(), Self::Error> {
        let mut outgoing_packet_buf = [b'\n'; N];

        let packet = serde_json_core::to_slice(resp, &mut outgoing_packet_buf)
            .map_err(|e| SerialRpcError::EncodeError(e))?;

        self.serial.write_all(&outgoing_packet_buf[0..=packet])?;

        Ok(())
    }

    /// Recommended that for each channel this controller supports, at least 128 bytes is allocated in the stack buffer
    async fn write_bulk(&mut self, packets: impl Iterator<Item = &OutgoingRpcPacket>) -> Result<(), Self::Error> {
        let mut outgoing_packet_buf = [b'\n'; N];

        let mut i = 0;
        for packet in packets {
            let j = serde_json_core::to_slice(packet, &mut outgoing_packet_buf[i..])
                .map_err(|e| SerialRpcError::EncodeError(e))?;

            i += j + 1;
        }
        self.serial.write_all(&outgoing_packet_buf[0..i])?;

        Ok(())
    }
}
