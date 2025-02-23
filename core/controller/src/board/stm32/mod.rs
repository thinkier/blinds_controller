use crate::board::{ConfigurableBoard, ControllableBoard, StepStickBoard};
#[cfg(feature = "host-usb")]
use crate::rpc::usb_cdc_acm::{DriverType, UsbRpcHandle};
use crate::{DRIVERS, STOPS};
use core::sync::atomic::Ordering;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Output;
use embassy_usb::driver::Driver;
use embedded_io::{Read, Write};

pub struct DriverPins<'a> {
    pub enable: Output<'a>,
    // pub step: Output<'a>,
    pub dir: Output<'a>,
}

pub struct Board<'a, const N: usize, D, H> {
    pub end_stops: [Option<ExtiInput<'a>>; N],
    pub drivers: [Option<DriverPins<'a>>; N],
    pub driver_serial: [D; N],
    pub host_rpc: H,
}

#[cfg(feature = "host-usb")]
impl<'a, const N: usize, D, H> ControllableBoard for Board<'a, N, D, H>
where
    H: DriverType,
    H::Driver: Driver<'a>,
{
    type Rpc = UsbRpcHandle<'a, 256, H::Driver>;

    fn get_host_rpc(&mut self) -> &mut Self::Rpc {
        todo!()
    }
}

impl<'a, const N: usize, D, H> StepStickBoard for Board<'a, N, D, H> {
    fn set_enabled(&mut self, channel: usize, enabled: bool) {
        todo!()
    }

    fn set_direction(&mut self, channel: usize, invert: bool) {
        todo!()
    }

    fn is_stopped(&mut self, channel: usize) -> bool {
        todo!()
    }

    fn is_ready_for_steps(&mut self, channel: usize) -> bool {
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
impl<'a, const N: usize, D, H> ConfigurableBoard<N> for Board<'a, N, D, H>
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
async fn stop_detector(i: usize, mut input: ExtiInput<'static>) {
    loop {
        input.wait_for_high().await;
        STOPS.bit_set(i as u32, Ordering::Release);
        input.wait_for_low().await;
    }
}
