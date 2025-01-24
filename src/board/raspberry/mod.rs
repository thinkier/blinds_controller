use embassy_rp::gpio::{Input, Output};
use embassy_rp::peripherals::{PIO0, PIO1};
use crate::board::raspberry::counted_sqr_wav_pio::CountedSqrWav;
use crate::board::{EndStopBoard, StepStickBoard};

#[cfg(feature = "btt_skr_pico_v1_0")]
mod btt_skr_pico_v1_0;
pub mod counted_sqr_wav_pio;

pub struct DriverPins<'a> {
    pub enable: Output<'a>,
    // pub step: Output<'a>,
    pub dir: Output<'a>,
}

pub struct Board<'a, const N: usize, D, H> {
    pub end_stops: Option<[Input<'a>; N]>,
    pub drivers: [DriverPins<'a>; N],
    pub driver_serial: D,
    pub host_serial: H,
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
    fn set_ena(&mut self, channel: usize, enabled: bool) {
        if enabled {
            self.drivers[channel].enable.set_low()
        } else {
            self.drivers[channel].enable.set_high()
        }
    }

    fn set_dir(&mut self, channel: usize, invert: bool) {
        if invert {
            self.drivers[channel].dir.set_high()
        } else {
            self.drivers[channel].dir.set_low()
        }
    }

    fn ready(&mut self, channel: usize) -> bool {
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

impl<'a, const N: usize, D, H> EndStopBoard for Board<'a, N, D, H> {
    async fn end_stop(&mut self, channel: usize) {
        if let Some(end_stops) = &mut self.end_stops {
            end_stops[channel].wait_for_high().await;


        }
    }
}