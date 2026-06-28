use crate::rpc::{AsyncRpc, AsyncRpcError, IncomingRpcPacket, OutgoingRpcPacket};
use defmt::{Format, Formatter};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::Driver;
use embassy_usb::{Config, UsbDevice};
use static_cell::StaticCell;

const fn config() -> Config<'static> {
    let mut config = Config::new(0xdead, 0xc0de);
    config.manufacturer = Some(env!("CARGO_PKG_AUTHORS"));
    config.product = Some(concat!(
        env!("CARGO_PKG_NAME"),
        " v",
        env!("CARGO_PKG_VERSION")
    ));
    config.serial_number = Some("12345678");
    config.self_powered = true;
    config.max_power = 0;
    config.max_packet_size_0 = 64;

    config
}

pub trait DriverType {
    type Driver;
}

impl<'a, D: Driver<'a>> DriverType for UsbCdcAcmStream<'a, D> {
    type Driver = D;
}

pub struct UsbCdcAcmStream<'a, D: Driver<'a>> {
    class: CdcAcmClass<'a, D>,
}

impl<D: Driver<'static>> UsbCdcAcmStream<'static, D> {
    /// This function is basically copied from the Embassy example
    ///
    /// https://github.com/embassy-rs/embassy/blob/a3d35216d4649fbadd3e78fe240b736258b7befe/examples/rp/src/bin/usb_serial.rs
    pub fn init(driver: D) -> (UsbDevice<'static, D>, Self) {
        let config = config();
        // Create embassy-usb DeviceBuilder using the driver and config.
        // It needs some buffers for building the descriptors.
        let mut builder = {
            static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
            static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
            static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

            let builder = embassy_usb::Builder::new(
                driver,
                config,
                CONFIG_DESCRIPTOR.init([0; 256]),
                BOS_DESCRIPTOR.init([0; 256]),
                &mut [], // no msos descriptors
                CONTROL_BUF.init([0; 64]),
            );
            builder
        };

        // Create classes on the builder.
        let class = {
            static STATE: StaticCell<State> = StaticCell::new();
            let state = STATE.init(State::new());
            CdcAcmClass::new(&mut builder, state, 64)
        };

        // Build the builder.
        (builder.build(), Self { class })
    }
}

pub enum UsbRpcError {
    IoError,
    ParseError(serde_json_core::de::Error),
    EncodeError(serde_json_core::ser::Error),
}

impl Format for UsbRpcError {
    fn format(&self, _fmt: Formatter) {
        todo!()
    }
}

impl AsyncRpcError for UsbRpcError {
    fn is_broken_input(&self) -> bool {
        false
    }
}

pub struct UsbRpcHandle<'a, const N: usize, D: Driver<'a>> {
    pub packet_buf: [u8; N],
    pub packet_cursor: usize,
    pub stream: UsbCdcAcmStream<'a, D>,
}

impl<'a, const N: usize, D: Driver<'a>> UsbRpcHandle<'a, N, D> {
    pub fn new(acm: UsbCdcAcmStream<'a, D>) -> Self {
        Self {
            packet_buf: [0u8; N],
            packet_cursor: 0,
            stream: acm,
        }
    }
}

impl<'a, const N: usize, D> AsyncRpc for UsbRpcHandle<'a, N, D>
where
    D: Driver<'a>,
{
    type Error = UsbRpcError;

    async fn peek(&mut self) -> Result<Option<IncomingRpcPacket>, Self::Error> {
        todo!()
    }

    async fn read(&mut self) -> Result<Option<IncomingRpcPacket>, Self::Error> {
        if self.stream.class.line_coding().data_rate() <= 1200 {
            return Ok(Some(IncomingRpcPacket::Bootloader));
        }

        while self.packet_buf[self.packet_cursor - 1] != 0x00 {
            let len = self
                .stream
                .class
                .read_packet(&mut self.packet_buf[self.packet_cursor..])
                .await
                .map_err(|_| UsbRpcError::IoError)?;

            self.packet_cursor += len;
        }

        let packet = serde_json_core::from_slice(&mut self.packet_buf[0..self.packet_cursor])
            .map_err(|e| UsbRpcError::ParseError(e))?
            .0;

        Ok(Some(packet))
    }

    async fn write(&mut self, packet: &OutgoingRpcPacket) -> Result<(), Self::Error> {
        let len = serde_json_core::to_slice(&packet, &mut self.packet_buf)
            .map_err(|e| UsbRpcError::EncodeError(e))?;

        // Guaranteed null byte termination
        self.packet_buf[len] = 0x00;
        let len = len + 1;

        let size = self.stream.class.max_packet_size() as usize;
        for range in 0..=(len / size) {
            let window = range * size..(range * size + size);

            self.stream
                .class
                .write_packet(&self.packet_buf[window])
                .await
                .map_err(|_| UsbRpcError::IoError)?;
        }

        Ok(())
    }
}
