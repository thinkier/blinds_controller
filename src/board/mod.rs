#[cfg(feature = "raspberry")]
pub mod raspberry;
#[cfg(feature = "tmc2209")]
pub mod tmc2209;

use embassy_executor::Spawner;
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

#[allow(clippy::wrong_self_convention)]
pub trait StepStickBoard {
    fn set_enabled(&mut self, channel: usize, enabled: bool);
    fn set_direction(&mut self, channel: usize, invert: bool);
    fn is_stopped(&mut self, channel: usize) -> bool;
    fn is_ready_for_steps(&mut self, channel: usize) -> bool;
    fn add_steps(&mut self, channel: usize, steps: u32) -> Option<bool>;
    fn clear_steps(&mut self, channel: usize);
}

pub trait EndStopBoard {
    fn bind_endstops(&mut self, spawner: Spawner);
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

#[cfg(feature = "stallguard")]
pub trait StallGuard<S, const N: usize> {
    /// StallGuard Threshold, scaled back to 8 bits
    async fn set_sg_threshold(&mut self, channel: usize, sgthrs: u8);
    /// StallGuard result, scaled back to 8 bits
    async fn get_sg_result(&mut self, channel: usize) -> Option<u8>;
}

trait SoftHalfDuplex {
    async fn flush_clear<const N: usize>(&mut self);
}

impl<S> SoftHalfDuplex for S
where
    S: Read + Write,
    S::Error: defmt::Format,
{
    /// If the hardware doesn't support blocking out the TX bytes,
    /// then this function consumes those bytes that got echoed back on the RX line.
    ///
    /// e.g.
    /// - `embassy-rp` does not support half duplex UART
    /// - `embassy-stm32` supports half duplex USART
    #[inline]
    async fn flush_clear<const N: usize>(&mut self) {
        #[cfg(feature = "software_half_duplex_uart")]
        {
            use embassy_time::Timer;

            Timer::after_millis(50).await;
            let _ = self.flush();
            let _ = self.read_exact(&mut [0u8; N]);
        }
    }
}
