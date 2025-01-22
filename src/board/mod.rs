#[cfg(feature = "btt_skr_pico_v1_0")]
mod btt_skr_pico_v1_0;
/// Supporting module to emulate the square wave generation capability on STM ACT peripheral
/// using RP PIO State machines
mod rp_act;
#[cfg(feature = "tmc2209")]
mod tmc2209;

use embassy_rp::gpio::{Input, Output};
pub use rp_act::counted_sqr_wav_pio::*;

pub struct DriverPins<'a> {
    pub enable: Output<'a>,
    // pub step: Output<'a>,
    pub dir: Output<'a>,
}

pub struct SerialBuffers {
    driver_tx_buf: [u8; 16],
    driver_rx_buf: [u8; 16],
    host_tx_buf: [u8; 256],
    host_rx_buf: [u8; 256],
}

impl SerialBuffers {
    pub(crate) const fn default() -> Self {
        Self {
            driver_tx_buf: [0; 16],
            driver_rx_buf: [0; 16],
            host_tx_buf: [0; 256],
            host_rx_buf: [0; 256],
        }
    }
}

pub struct Board<'a, const N: usize, D, H> {
    pub end_stops: [Input<'a>; N],
    pub drivers: [DriverPins<'a>; N],
    pub driver_serial: D,
    pub host_serial: H,
}
