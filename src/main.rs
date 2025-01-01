#![no_std]
#![no_main]

mod board;
mod config;

use crate::board::get_driver_serial;
use cortex_m::prelude::_embedded_hal_blocking_serial_Write;
use defmt::*;
use embassy_embedded_hal;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_time::{Duration, Instant, Timer};
use gpio::{Level, Output};
use tmc2209::reg::CHOPCONF;
use tmc2209::{send_write_request};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialise Peripherals
    info!("Initialising Peripherals");
    let mut p = embassy_rp::init(Default::default());

    {
        let mut driver_serial = get_driver_serial(&mut p);
        let mut chop = CHOPCONF::default();
        chop.set_vsense(false); // Essential for using the 0R11 external sense resistors on the board, which will program the driver to run at approximately ~1.7A
        for addr in 0..4 {
            if let Err(e) = send_write_request(addr, chop, &mut driver_serial) {
                info!("Failed to program CHOPCONF on addr {}: {:?}", addr, e);
            }
        }
    }

    let _enable = Output::new(p.PIN_15, Level::Low);
    let mut step = Output::new(p.PIN_14, Level::Low);

    info!("Peripherals Initialised");

    let mut counter = 0;
    let mut start = Instant::now();
    loop {
        let period = Timer::after_micros(50);

        step.set_high();
        Timer::after_nanos(125).await; // $t_{sh}$ as per datasheet
        step.set_low();

        counter += 1;
        if Instant::now().duration_since(start) > Duration::from_millis(1000) {
            info!("Counter: {}", counter);
            counter = 0;
            start = Instant::now();
        }
        period.await;
    }
}
