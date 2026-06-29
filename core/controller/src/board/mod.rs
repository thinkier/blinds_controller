#[cfg(feature = "rp")]
pub mod rp;
#[cfg(feature = "stm32")]
pub mod stm32;
#[cfg(any(feature = "tmc2209_uart", feature = "tmc2209_uart_async"))]
pub mod tmc2209_uart;

cfg_select! {
    feature = "rp" => { use rp::Board; },
    feature = "stm32" => { use stm32::Board; }
    _ => {}
}

use crate::rpc::AsyncRpc;
use embassy_executor::Spawner;
cfg_select! {
    feature = "uart_configurable_driver" => {
        use embedded_io::{Read, Write};
    }
    feature = "uart_configurable_driver_async" => {
        use embedded_io_async::{Read, Write};
    }
    _ => {}
}

#[macro_export]
macro_rules! static_buffer {
    ($name:tt: $size:literal) => {
        static $name: static_cell::ConstStaticCell<[u8; $size]> =
            static_cell::ConstStaticCell::new([0; $size]);
    };
}

pub trait StepStickHost {
    fn get_enabled(&mut self, channel: usize) -> bool;
    fn set_enabled(&mut self, channel: usize, enabled: bool);
    fn set_direction(&mut self, channel: usize, invert: bool);
    fn get_stopped(&mut self, channel: usize) -> bool;
    fn get_ready_for_steps(&mut self, channel: usize) -> bool;
    fn add_steps(&mut self, channel: usize, steps: u32) -> Option<bool>;
    fn clear_steps(&mut self, channel: usize);
}

pub trait ControllableBoard {
    type Rpc: AsyncRpc;

    fn get_host_rpc(&mut self) -> &mut Self::Rpc;

    fn reset(&mut self);

    fn enter_bootloader(&mut self);
    /// Feed the board's watchdog, should it be implemented.
    ///
    /// Implementers of Watchdog should be much more generous than the timer in the run loop.
    fn watchdog_feed(&mut self) {}
}

#[allow(async_fn_in_trait)]
pub trait ControlLoopInvoke {
    async fn invoke(&mut self, _spawner: &mut Spawner);
}

#[cfg(any(
    feature = "uart_configurable_driver",
    feature = "uart_configurable_driver_async"
))]
pub trait ConfigurableStepStickHost<const N: usize> {
    type DriverSerial: Read + Write;

    fn driver_serial(&mut self, addr: u8) -> &mut Self::DriverSerial;
}

#[allow(async_fn_in_trait)]
pub trait ConfigurableStepStickDriver<S, const N: usize> {
    async fn configure_driver(&mut self);
}

#[cfg(all(
    feature = "stallguard",
    any(
        feature = "uart_configurable_driver",
        feature = "uart_configurable_driver_async"
    )
))]
#[allow(async_fn_in_trait)]
pub trait StallGuard<S, const N: usize> {
    /// StallGuard Threshold, scaled back to 8 bits
    async fn set_sg_threshold(&mut self, channel: u8, sgthrs: u8);
    /// StallGuard result, scaled back to 8 bits
    async fn get_sg_result_halved(&mut self, channel: u8) -> Option<u8>;
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

#[cfg(any(feature = "rp", feature = "stm32"))]
impl<'a, const N: usize, D, H, T> ControlLoopInvoke for Board<'a, N, D, H, T>
where
    T: ControlLoopInvoke,
{
    async fn invoke(&mut self, spawner: &mut Spawner) {
        self.board_state.invoke(spawner).await
    }
}
