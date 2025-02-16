#![no_std]
#![no_main]

mod board;

use crate::board::BoardInitialize;
use blinds_controller::board::stm32::Board;
use blinds_controller::board::ConfigurableDriver;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut board = Board::init(spawner);
    board.configure_driver().await;

    blinds_controller::run(spawner, board).await;
}
