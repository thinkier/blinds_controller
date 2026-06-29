pub mod bitbanged_uart;

use crate::board::{ConfigurableStepStickHost, ControllableBoard, StepStickHost};
#[cfg(feature = "host-usb")]
use crate::rpc::UsbRpcHandle;
use crate::{DRIVERS, STOPS};
use core::sync::atomic::Ordering;
use cortex_m::peripheral::SCB;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output};
use embassy_stm32::mode::Async;
#[cfg(feature = "host-usb")]
use embassy_usb::driver::Driver as UsbDriver;

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
    pub drivers: [DriverPins<'a>; N],
    pub driver_serial: D,
    pub host_rpc: H,
    pub board_state: T,
}

#[cfg(feature = "host-usb")]
impl<const N: usize, const B: usize, D, HD, T> ControllableBoard
    for Board<'static, N, D, UsbRpcHandle<B, HD>, T>
where
    HD: UsbDriver<'static>,
{
    type Rpc = UsbRpcHandle<B, HD>;

    fn get_host_rpc(&mut self) -> &mut Self::Rpc {
        &mut self.host_rpc
    }

    fn reset(&mut self) {
        SCB::sys_reset();
    }

    fn enter_bootloader(&mut self) {
        self.reset() // No native first stage bootloader on STM32
    }
}

impl<'a, const N: usize, D, H, T> StepStickHost for Board<'a, N, D, H, T> {
    fn get_enabled(&mut self, channel: usize) -> bool {
        self.drivers[channel].enable.is_set_low()
    }

    fn set_enabled(&mut self, channel: usize, enabled: bool) {
        self.drivers[channel].enable.set_level(if enabled { Level::Low } else { Level::High });
    }

    fn set_direction(&mut self, channel: usize, invert: bool) {
        self.drivers[channel].dir.set_level(if invert { Level::High } else { Level::Low });
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
impl<'a, const N: usize, D, H, T> ConfigurableStepStickHost<N> for Board<'a, N, [D; N], H, T>
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
