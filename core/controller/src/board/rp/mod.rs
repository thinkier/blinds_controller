use crate::board::rp::utils::counted_sqr_wav_pio::CountedSqrWav;
use crate::board::{ConfigurableBoard, ControllableBoard, StepStickBoard};
use crate::rpc::SerialRpcHandle;
use crate::{DRIVERS, STOPS};
use core::sync::atomic::Ordering;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Output};
use embassy_rp::peripherals::PIO0;
#[cfg(feature = "driver-qty-ge-5")]
use embassy_rp::peripherals::PIO1;
use embassy_time::Timer;
use embedded_io::{ErrorType, Read, ReadReady, Write};

pub mod utils;

pub struct DriverPins<'a> {
    pub enable: Output<'a>,
    // pub step: Output<'a>,
    pub dir: Output<'a>,
}

pub struct Board<'a, const N: usize, D, H> {
    pub drivers: [DriverPins<'a>; N],
    pub driver_serial: D,
    pub host_rpc: SerialRpcHandle<256, H>,
    // State machines - alternative to an ACT timer on STM controllers
    pub pio0_0: Option<CountedSqrWav<'a, PIO0, 0>>,
    pub pio0_1: Option<CountedSqrWav<'a, PIO0, 1>>,
    pub pio0_2: Option<CountedSqrWav<'a, PIO0, 2>>,
    pub pio0_3: Option<CountedSqrWav<'a, PIO0, 3>>,
    #[cfg(feature = "driver-qty-ge-5")]
    pub pio1_0: Option<CountedSqrWav<'a, PIO1, 0>>,
    #[cfg(feature = "driver-qty-ge-8")]
    pub pio1_1: Option<CountedSqrWav<'a, PIO1, 1>>,
    #[cfg(feature = "driver-qty-ge-8")]
    pub pio1_2: Option<CountedSqrWav<'a, PIO1, 2>>,
    #[cfg(feature = "driver-qty-ge-8")]
    pub pio1_3: Option<CountedSqrWav<'a, PIO1, 3>>,
}

impl<'a, const N: usize, D, H> ControllableBoard for Board<'a, N, D, H>
where
    H: Read + ReadReady + Write,
    <H as ErrorType>::Error: defmt::Format,
{
    type Rpc = SerialRpcHandle<256, H>;

    fn get_host_rpc(&mut self) -> &mut Self::Rpc {
        &mut self.host_rpc
    }
}

impl<'a, const N: usize, D, H> StepStickBoard for Board<'a, N, D, H> {
    fn set_enabled(&mut self, channel: usize, enabled: bool) {
        if enabled {
            self.drivers[channel].enable.set_low()
        } else {
            self.drivers[channel].enable.set_high()
        }
    }

    fn set_direction(&mut self, channel: usize, invert: bool) {
        if invert {
            self.drivers[channel].dir.set_high()
        } else {
            self.drivers[channel].dir.set_low()
        }
    }

    fn is_stopped(&mut self, channel: usize) -> bool {
        match channel {
            0 => self.pio0_0.as_mut().map(|p| p.stopped()).unwrap_or(true),
            1 => self.pio0_1.as_mut().map(|p| p.stopped()).unwrap_or(true),
            2 => self.pio0_2.as_mut().map(|p| p.stopped()).unwrap_or(true),
            3 => self.pio0_3.as_mut().map(|p| p.stopped()).unwrap_or(true),
            #[cfg(feature = "driver-qty-ge-5")]
            4 => self.pio1_0.as_mut().map(|p| p.stopped()).unwrap_or(true),
            #[cfg(feature = "driver-qty-ge-8")]
            5 => self.pio1_1.as_mut().map(|p| p.stopped()).unwrap_or(true),
            #[cfg(feature = "driver-qty-ge-8")]
            6 => self.pio1_2.as_mut().map(|p| p.stopped()).unwrap_or(true),
            #[cfg(feature = "driver-qty-ge-8")]
            7 => self.pio1_3.as_mut().map(|p| p.stopped()).unwrap_or(true),
            _ => true,
        }
    }

    fn is_ready_for_steps(&mut self, channel: usize) -> bool {
        match channel {
            0 => self.pio0_0.as_mut().map(|p| p.ready()).unwrap_or(false),
            1 => self.pio0_1.as_mut().map(|p| p.ready()).unwrap_or(false),
            2 => self.pio0_2.as_mut().map(|p| p.ready()).unwrap_or(false),
            3 => self.pio0_3.as_mut().map(|p| p.ready()).unwrap_or(false),
            #[cfg(feature = "driver-qty-ge-5")]
            4 => self.pio1_0.as_mut().map(|p| p.ready()).unwrap_or(false),
            #[cfg(feature = "driver-qty-ge-8")]
            5 => self.pio1_1.as_mut().map(|p| p.ready()).unwrap_or(false),
            #[cfg(feature = "driver-qty-ge-8")]
            6 => self.pio1_2.as_mut().map(|p| p.ready()).unwrap_or(false),
            #[cfg(feature = "driver-qty-ge-8")]
            7 => self.pio1_3.as_mut().map(|p| p.ready()).unwrap_or(false),
            _ => false,
        }
    }

    fn add_steps(&mut self, channel: usize, steps: u32) -> Option<bool> {
        if steps == 0 {
            return None;
        }

        match channel {
            0 => self.pio0_0.as_mut().map(|p| p.try_push(steps)),
            1 => self.pio0_1.as_mut().map(|p| p.try_push(steps)),
            2 => self.pio0_2.as_mut().map(|p| p.try_push(steps)),
            3 => self.pio0_3.as_mut().map(|p| p.try_push(steps)),
            #[cfg(feature = "driver-qty-ge-5")]
            4 => self.pio1_0.as_mut().map(|p| p.try_push(steps)),
            #[cfg(feature = "driver-qty-ge-8")]
            5 => self.pio1_1.as_mut().map(|p| p.try_push(steps)),
            #[cfg(feature = "driver-qty-ge-8")]
            6 => self.pio1_2.as_mut().map(|p| p.try_push(steps)),
            #[cfg(feature = "driver-qty-ge-8")]
            7 => self.pio1_3.as_mut().map(|p| p.try_push(steps)),
            _ => None,
        }
    }

    fn clear_steps(&mut self, channel: usize) {
        match channel {
            0 => self.pio0_0.as_mut().map(|p| p.clear()),
            1 => self.pio0_1.as_mut().map(|p| p.clear()),
            2 => self.pio0_2.as_mut().map(|p| p.clear()),
            3 => self.pio0_3.as_mut().map(|p| p.clear()),
            #[cfg(feature = "driver-qty-ge-5")]
            4 => self.pio1_0.as_mut().map(|p| p.clear()),
            #[cfg(feature = "driver-qty-ge-8")]
            5 => self.pio1_1.as_mut().map(|p| p.clear()),
            #[cfg(feature = "driver-qty-ge-8")]
            6 => self.pio1_2.as_mut().map(|p| p.clear()),
            #[cfg(feature = "driver-qty-ge-8")]
            7 => self.pio1_3.as_mut().map(|p| p.clear()),
            _ => None,
        };
    }
}

#[cfg(feature = "uart_configurable_driver")]
impl<'a, const N: usize, D, H> ConfigurableBoard<N> for Board<'a, N, D, H>
where
    D: Read + Write,
{
    type DriverSerial = D;

    fn driver_serial(&mut self, _addr: u8) -> &mut Self::DriverSerial {
        &mut self.driver_serial
    }
}

pub fn bind_endstops<const N: usize>(spawner: Spawner, inputs: [Input<'static>; N]) {
    let mut i = 0;
    for stop in inputs {
        let _ = spawner.spawn(stop_detector(i, stop));
        i += 1;
    }
}

/// Not universally compatible
///
/// See: https://docs.embassy.dev/embassy-rp/git/rp2040/gpio/struct.Input.html
#[embassy_executor::task(pool_size = DRIVERS)]
async fn stop_detector(i: usize, mut input: Input<'static>) {
    loop {
        input.wait_for_high().await;
        STOPS.bit_set(i as u32, Ordering::Release);
        Timer::after_secs(1).await; // Dead Time Insertion
        input.wait_for_low().await;
    }
}
