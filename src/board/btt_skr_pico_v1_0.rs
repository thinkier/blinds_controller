use crate::board::{Board, DriverPins};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{UART0, UART1};
use embassy_rp::uart::{Blocking, Config, Uart};
use embassy_rp::Peripherals;

impl<'a> Board<'a, 4, Uart<'a, UART1, Blocking>, Uart<'a, UART0, Blocking>> {
    pub fn init(p: Peripherals) -> Self {
        let mut uart_cfg = Config::default();
        uart_cfg.baudrate = 115200;

        let driver_serial = Uart::new_blocking(p.UART1, p.PIN_8, p.PIN_9, uart_cfg);
        let host_serial = Uart::new_blocking(p.UART0, p.PIN_0, p.PIN_1, uart_cfg);

        Self {
            driver: [
                DriverPins {
                    diag: Input::new(p.PIN_4, Pull::Down),
                    enable: Output::new(p.PIN_12, Level::High),
                    step: Output::new(p.PIN_11, Level::Low),
                    dir: Output::new(p.PIN_10, Level::Low),
                },
                DriverPins {
                    diag: Input::new(p.PIN_25, Pull::Down),
                    enable: Output::new(p.PIN_2, Level::High),
                    step: Output::new(p.PIN_19, Level::Low),
                    dir: Output::new(p.PIN_28, Level::Low),
                },
                DriverPins {
                    diag: Input::new(p.PIN_3, Pull::Down),
                    enable: Output::new(p.PIN_7, Level::High),
                    step: Output::new(p.PIN_6, Level::Low),
                    dir: Output::new(p.PIN_5, Level::Low),
                },
                DriverPins {
                    diag: Input::new(p.PIN_16, Pull::Down),
                    enable: Output::new(p.PIN_15, Level::High),
                    step: Output::new(p.PIN_14, Level::Low),
                    dir: Output::new(p.PIN_13, Level::Low),
                },
            ],
            driver_serial,
            host_serial,
        }
    }
}
