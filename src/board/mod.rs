#[cfg(feature = "btt_skr_pico_v1_0")]
mod btt_skr_pico_v1_0;
#[cfg(feature = "driver_tmc2209")]
mod tmc2209;

use embassy_rp::gpio::{Input, Output};

pub struct DriverPins<'a> {
    pub stop: Input<'a>,
    pub enable: Output<'a>,
    pub step: Output<'a>,
    pub dir: Output<'a>,
}

impl DriverPins<'_> {
    pub fn reset(&mut self) {
        self.enable.set_high();
        self.step.set_low();
        self.dir.set_low();
    }
}

pub struct Board<'a, const N: usize, D, H> {
    pub driver: [DriverPins<'a>; N],
    pub driver_serial: D,
    pub host_serial: H,
}
