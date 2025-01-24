#[cfg(feature = "raspberry")]
pub mod raspberry;
#[cfg(feature = "tmc2209")]
pub mod tmc2209;

use embedded_io::{Read, Write};

pub struct SerialBuffers {
    driver_tx_buf: [u8; 32],
    driver_rx_buf: [u8; 32],
    host_tx_buf: [u8; 256],
    host_rx_buf: [u8; 256],
}

impl SerialBuffers {
    pub(crate) const fn default() -> Self {
        Self {
            driver_tx_buf: [0; 32],
            driver_rx_buf: [0; 32],
            host_tx_buf: [0; 256],
            host_rx_buf: [0; 256],
        }
    }
}

pub trait StepStickBoard {
    fn set_ena(&mut self, channel: usize, enabled: bool);
    fn set_dir(&mut self, channel: usize, invert: bool);
    fn ready(&mut self, channel: usize) -> bool;
    fn add_steps(&mut self, channel: usize, steps: u32) -> Option<bool>;
    fn clear_steps(&mut self, channel: usize);
}

pub trait EndStopBoard: StepStickBoard {
    async fn end_stop(&mut self, channel: usize);
}

#[cfg(feature = "configurable_driver")]
pub trait ConfigurableBoard<const N: usize> {
    type DriverSerial: Read + Write;

    fn driver_serial(&mut self) -> &mut Self::DriverSerial;
}

#[cfg(feature = "configurable_driver")]
pub trait ConfigurableDriver<S, const N: usize> {
    async fn configure_driver(&mut self);
}
