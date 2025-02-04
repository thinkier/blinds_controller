use crate::board::raspberry::counted_sqr_wav_pio::CountedSqrWav;
use crate::board::{EndStopBoard, StepStickBoard};
use crate::comms::RpcHandle;
use crate::{DRIVERS, STOPS};
use core::mem;
use core::sync::atomic::Ordering;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Output};
use embassy_rp::peripherals::{PIO0, PIO1};

#[cfg(feature = "btt_skr_pico_v1.0")]
mod btt_skr_pico_v1_0;
pub mod counted_sqr_wav_pio;

pub struct DriverPins<'a> {
    pub enable: Output<'a>,
    // pub step: Output<'a>,
    pub dir: Output<'a>,
}

pub struct Board<'a, const N: usize, D, H> {
    pub end_stops: [Option<Input<'a>>; N],
    pub drivers: [DriverPins<'a>; N],
    pub driver_serial: D,
    pub host_rpc: RpcHandle<256, H>,
    // State machines - alternative to an ACT timer on STM controllers
    pub pio0_0: Option<CountedSqrWav<'a, PIO0, 0>>,
    pub pio0_1: Option<CountedSqrWav<'a, PIO0, 1>>,
    pub pio0_2: Option<CountedSqrWav<'a, PIO0, 2>>,
    pub pio0_3: Option<CountedSqrWav<'a, PIO0, 3>>,
    pub pio1_0: Option<CountedSqrWav<'a, PIO1, 0>>,
    pub pio1_1: Option<CountedSqrWav<'a, PIO1, 1>>,
    pub pio1_2: Option<CountedSqrWav<'a, PIO1, 2>>,
    pub pio1_3: Option<CountedSqrWav<'a, PIO1, 3>>,
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
            4 => self.pio1_0.as_mut().map(|p| p.stopped()).unwrap_or(true),
            5 => self.pio1_1.as_mut().map(|p| p.stopped()).unwrap_or(true),
            6 => self.pio1_2.as_mut().map(|p| p.stopped()).unwrap_or(true),
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
            4 => self.pio1_0.as_mut().map(|p| p.ready()).unwrap_or(false),
            5 => self.pio1_1.as_mut().map(|p| p.ready()).unwrap_or(false),
            6 => self.pio1_2.as_mut().map(|p| p.ready()).unwrap_or(false),
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
            4 => self.pio1_0.as_mut().map(|p| p.try_push(steps)),
            5 => self.pio1_1.as_mut().map(|p| p.try_push(steps)),
            6 => self.pio1_2.as_mut().map(|p| p.try_push(steps)),
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
            4 => self.pio1_0.as_mut().map(|p| p.clear()),
            5 => self.pio1_1.as_mut().map(|p| p.clear()),
            6 => self.pio1_2.as_mut().map(|p| p.clear()),
            7 => self.pio1_3.as_mut().map(|p| p.clear()),
            _ => None,
        };
    }
}

impl<const N: usize, D, H> EndStopBoard for Board<'static, N, D, H> {
    fn bind_endstops(&mut self, spawner: Spawner) {
        let mut i = 0;
        for stop in mem::replace(&mut self.end_stops, [const { None }; N]) {
            if let Some(stop) = stop {
                let _ = spawner.spawn(stop_detector(i, stop));
                i += 1;
            }
        }
    }
}

/// Not universally compatible
///
/// See: https://docs.embassy.dev/embassy-stm32/git/stm32g0b1re/exti/struct.ExtiInput.html#method.wait_for_rising_edge
#[embassy_executor::task(pool_size = DRIVERS)]
async fn stop_detector(i: usize, mut input: Input<'static>) {
    loop {
        input.wait_for_high().await;
        STOPS.bit_set(i as u32, Ordering::Release);
        input.wait_for_low().await;
    }
}
