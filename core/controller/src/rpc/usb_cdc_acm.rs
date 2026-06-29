use crate::rpc::{AsyncRpc, AsyncRpcError, IncomingRpcPacket, OutgoingRpcPacket};
use circ_buffer::RingBuffer;
use core::cmp::min;
use defmt::{Format, Formatter};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::Driver as UsbDriver;
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

pub struct UsbCdcAcmStream<'a, D: UsbDriver<'a>> {
    class: CdcAcmClass<'a, D>,
}

impl<D: UsbDriver<'static>> UsbCdcAcmStream<'static, D> {
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

pub struct UsbRpcHandle<const N: usize, D: UsbDriver<'static>> {
    pub rx_buf: RingBuffer<u8, N>,
    read_buf: Option<IncomingRpcPacket>,
    pub stream: UsbCdcAcmStream<'static, D>,
}

impl<const N: usize, D: UsbDriver<'static>> UsbRpcHandle<N, D> {
    pub fn new(driver: D) -> (UsbDevice<'static, D>, Self) {
        let (device, stream) = UsbCdcAcmStream::init(driver);

        (
            device,
            Self {
                rx_buf: RingBuffer::new(),
                read_buf: None,
                stream,
            },
        )
    }
}

impl<const N: usize, D> AsyncRpc for UsbRpcHandle<N, D>
where
    D: UsbDriver<'static>,
{
    type Error = UsbRpcError;

    async fn peek(&mut self) -> Result<Option<&IncomingRpcPacket>, Self::Error> {
        if self.read_buf.is_some() {
            return Ok(self.read_buf.as_ref());
        }

        self.read_buf = self.read().await?;

        Ok(self.read_buf.as_ref())
    }

    async fn read(&mut self) -> Result<Option<IncomingRpcPacket>, Self::Error> {
        todo!()
    }

    async fn write(&mut self, packet: &OutgoingRpcPacket) -> Result<(), Self::Error> {
        let mut tx_packet_buf = [b'\n'; N];
        let len = serde_json_core::to_slice(&packet, &mut tx_packet_buf)
            .map_err(|e| UsbRpcError::EncodeError(e))?
            + 1;

        let size = self.stream.class.max_packet_size() as usize;
        for range in 0..=(len / size) {
            let window = range * size..min(range * size + size, len);

            self.stream
                .class
                .write_packet(&tx_packet_buf[window])
                .await
                .map_err(|_| UsbRpcError::IoError)?;
        }

        Ok(())
    }
}
