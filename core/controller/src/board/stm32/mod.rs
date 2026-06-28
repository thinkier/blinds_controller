pub mod bitbanged_uart;

use crate::board::{ConfigurableBoard, ControlLoopInvoke, ControllableBoard, StepStickBoard};
#[cfg(feature = "host-usb")]
use crate::rpc::{DriverType, UsbRpcHandle};
use crate::{DRIVERS, STOPS};
use core::sync::atomic::Ordering;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_usb::driver::Driver;
cfg_select! {
    feature = "uart_configurable_driver" => {
        use embedded_io::{Read, Write};
    }
}

pub struct DriverPins<'a> {
    pub enable: Output<'a>,
    // pub step: Output<'a>,
    pub dir: Output<'a>,
}

pub struct Board<'a, const N: usize, D, H, T> {
    pub end_stops: [Option<ExtiInput<'a, Async>>; N],
    pub drivers: [Option<DriverPins<'a>>; N],
    pub driver_serial: [D; N],
    pub host_rpc: H,
    pub board_state: T,
}

#[cfg(feature = "host-usb")]
impl<'a, const N: usize, D, H, T> ControllableBoard for Board<'a, N, D, H, T>
where
    H: DriverType,
    H::Driver: Driver<'a>,
{
    type Rpc = UsbRpcHandle<'a, 256, H::Driver>;

    fn get_host_rpc(&mut self) -> &mut Self::Rpc {
        todo!()
    }

    fn reset(&mut self) {
        todo!()
    }

    fn enter_bootloader(&mut self) {
        todo!()
    }
}

impl<'a, const N: usize, D, H, T> StepStickBoard for Board<'a, N, D, H, T> {
    fn get_enabled(&mut self, channel: usize) -> bool {
        todo!()
    }

    fn set_enabled(&mut self, channel: usize, enabled: bool) {
        todo!()
    }

    fn set_direction(&mut self, channel: usize, invert: bool) {
        todo!()
    }

    fn get_stopped(&mut self, channel: usize) -> bool {
        todo!()
    }

    fn get_ready_for_steps(&mut self, channel: usize) -> bool {
        todo!()
    }

    fn add_steps(&mut self, channel: usize, steps: u32) -> Option<bool> {
        todo!()
    }

    fn clear_steps(&mut self, channel: usize) {
        todo!()
    }
}

#[cfg(feature = "uart_configurable_driver")]
impl<'a, const N: usize, D, H, T> ConfigurableBoard<N> for Board<'a, N, D, H, T>
where
    D: Read + Write,
{
    type DriverSerial = D;

    fn driver_serial(&mut self, addr: u8) -> &mut Self::DriverSerial {
        &mut self.driver_serial[addr as usize]
    }
}

/// Not universally compatible
///
/// See: https://docs.embassy.dev/embassy-stm32/git/stm32g0b1re/exti/struct.ExtiInput.html#method.wait_for_rising_edge
#[embassy_executor::task(pool_size = DRIVERS)]
async fn stop_detector(i: usize, mut input: ExtiInput<'static, Async>) {
    loop {
        input.wait_for_high().await;
        STOPS.bit_set(i as u32, Ordering::Release);
        input.wait_for_low().await;
    }
}

impl<'a, const N: usize, D, H, T> ControlLoopInvoke for Board<'a, N, D, H, T>
where T: ControlLoopInvoke {
    async fn invoke(&mut self, spawner: &mut Spawner) {
        self.board_state.invoke(spawner).await
    }
}
