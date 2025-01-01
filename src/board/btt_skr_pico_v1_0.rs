use crate::config::DriverPinN;
use embassy_rp::peripherals::UART1;
use embassy_rp::uart::{Blocking, Config, Uart};
use embassy_rp::Peripherals;

pub fn get_driver_serial(p: &mut Peripherals) -> Uart<UART1, Blocking> {
    let mut cfg = Config::default();
    cfg.baudrate = 115200;

    Uart::new_blocking(&mut p.UART1, &mut p.PIN_8, &mut p.PIN_9, cfg)
}

pub static DRIVERS: [DriverPinN; 4] = [
    DriverPinN {
        interrupt: 4,
        enable: 12,
        step: 11,
        dir: 10,
    },
    DriverPinN {
        interrupt: 25,
        enable: 2,
        step: 19,
        dir: 28,
    },
    DriverPinN {
        interrupt: 3,
        enable: 7,
        step: 6,
        dir: 5,
    },
    DriverPinN {
        interrupt: 16,
        enable: 15,
        step: 14,
        dir: 13,
    },
];
