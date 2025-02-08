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

pub struct UsbCdcAcmStream<'a, D> {
    class: CdcAcmClass<'a, D>,
}

impl<D: Driver<'static>> UsbCdcAcmStream<'static, D> {
    /// This function is basically copied from the Embassy example
    ///
    /// https://github.com/embassy-rs/embassy/blob/a3d35216d4649fbadd3e78fe240b736258b7befe/examples/rp/src/bin/usb_serial.rs
    pub fn make_acm<D: Driver<'static>>(driver: D) -> (UsbDevice<'static, D>, Self) {
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
