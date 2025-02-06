use crate::comms::RpcHandle;
use crate::{DRIVERS, STOPS};
use core::sync::atomic::Ordering;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Output;
use embedded_io::{Read, Write};
use crate::board::{ConfigurableBoard, EndStopBoard, StepStickBoard};

#[cfg(feature = "btt_manta_e3ez")]
mod btt_manta_e3ez;
#[cfg(feature = "btt_octopus")]
mod btt_octopus;

pub struct DriverPins<'a> {
    pub enable: Output<'a>,
    // pub step: Output<'a>,
    pub dir: Output<'a>,
}

pub struct Board<'a, const N: usize, D, H> {
    pub end_stops: [Option<ExtiInput<'a>>; N],
    pub drivers: [Option<DriverPins<'a>>; N],
    pub driver_serial: [D; N],
    pub host_rpc: RpcHandle<256, H>,
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

impl<'a, const N: usize, D, H> EndStopBoard for Board<'a, N, D, H> {
    fn bind_endstops(&mut self, spawner: Spawner) {
        todo!()
    }
}


#[cfg(feature = "configurable_driver")]
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
