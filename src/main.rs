#![no_std]
#![no_main]

mod board;
use crate::board::*;
use defmt::*;
use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialise Peripherals
    info!("Initialising Peripherals");
    let p = embassy_rp::init(Default::default());
    let mut board = Board::init(p);
    board.configure_drivers();

    info!("Peripherals Initialised");

    board.driver[3].enable.set_low();
    let mut counter = 0;
    let mut start = Instant::now();
    loop {
        let period = Timer::after_micros(500); // Actual ~= 1500 half-steps per second

        board.driver[3].step.set_high();
        Timer::after_nanos(125).await; // $t_{sh}$ as per datasheet
        board.driver[3].step.set_low();

        counter += 1;
        if Instant::now().duration_since(start) > Duration::from_millis(1000) {
            trace!("Counter: {}", counter);
            counter = 0;
            start = Instant::now();
        }
        period.await;
    }
}
