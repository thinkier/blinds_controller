#[cfg(feature = "rp")]
mod rp;
#[cfg(feature = "stm32")]
mod stm32;
#[cfg(feature = "tmc2209_uart")]
pub mod tmc2209_uart;

use embedded_io::{Read, Write};

#[allow(unused)]
#[cfg(feature = "rp")]
pub use crate::board::rp::Board;

#[allow(unused)]
#[cfg(feature = "stm32")]
pub use crate::board::stm32::Board;

#[macro_export]
macro_rules! static_buffer {
    ($name:tt: $size:literal) => {
        static $name: static_cell::ConstStaticCell<[u8; $size]> =
            static_cell::ConstStaticCell::new([0; $size]);
    };
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

#[cfg(feature = "uart_configurable_driver")]
pub trait ConfigurableBoard<const N: usize> {
    type DriverSerial: Read + Write;

    fn driver_serial(&mut self, addr: u8) -> &mut Self::DriverSerial;
}
#[cfg(feature = "uart_configurable_driver")]
pub trait ConfigurableDriver<S, const N: usize> {
    async fn configure_driver(&mut self);
}

#[cfg(feature = "stallguard")]
pub trait StallGuard<S, const N: usize> {
    /// StallGuard Threshold, scaled back to 8 bits
    async fn set_sg_threshold(&mut self, channel: u8, sgthrs: u8);
    /// StallGuard result, scaled back to 8 bits
    async fn get_sg_result(&mut self, channel: u8) -> Option<u8>;
}

#[cfg(feature = "uart_soft_half_duplex")]
trait SoftHalfDuplex {
    async fn flush_clear<const N: usize>(&mut self);
}

#[cfg(feature = "uart_soft_half_duplex")]
impl<S> SoftHalfDuplex for S
where
    S: Read + Write,
    S::Error: defmt::Format,
{
    /// If the hardware doesn't support blocking out the TX bytes,
    /// then this function consumes those bytes that got echoed back on the RX line.
    ///
    /// e.g.
    /// - `embassy-rp` does not prevent half duplex read-back, so the bytes must be discarded manually
    /// - `embassy-stm32` has hardware support to prevent half duplex read-back
    #[inline]
    async fn flush_clear<const N: usize>(&mut self) {
        use embassy_time::Timer;

        Timer::after_millis(50).await;
        let _ = self.flush();
        let _ = self.read_exact(&mut [0u8; N]);
    }
}
