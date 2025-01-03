use crate::board::{Board, DriverPins};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{UART0, UART1};
use embassy_rp::uart::{Blocking, Config, Uart};
use embassy_rp::Peripherals;

impl<'a> Board<'a, 4, Uart<'a, UART1, Blocking>, Uart<'a, UART0, Blocking>> {
    pub fn init(p: &'a mut Peripherals) -> Self {
        let mut uart_cfg = Config::default();
        uart_cfg.baudrate = 115200;

        let driver_serial = Uart::new_blocking(&mut p.UART1, &mut p.PIN_8, &mut p.PIN_9, uart_cfg);
        let host_serial = Uart::new_blocking(&mut p.UART0, &mut p.PIN_0, &mut p.PIN_1, uart_cfg);

        Self {
            driver: [
                DriverPins {
                    stop: Input::new(&mut p.PIN_4, Pull::Down),
                    enable: Output::new(&mut p.PIN_12, Level::High),
                    step: Output::new(&mut p.PIN_11, Level::Low),
                    dir: Output::new(&mut p.PIN_10, Level::Low),
                },
                DriverPins {
                    stop: Input::new(&mut p.PIN_25, Pull::Down),
                    enable: Output::new(&mut p.PIN_2, Level::High),
                    step: Output::new(&mut p.PIN_19, Level::Low),
                    dir: Output::new(&mut p.PIN_28, Level::Low),
                },
                DriverPins {
                    stop: Input::new(&mut p.PIN_3, Pull::Down),
                    enable: Output::new(&mut p.PIN_7, Level::High),
                    step: Output::new(&mut p.PIN_6, Level::Low),
                    dir: Output::new(&mut p.PIN_5, Level::Low),
                },
                DriverPins {
                    stop: Input::new(&mut p.PIN_16, Pull::Down),
                    enable: Output::new(&mut p.PIN_15, Level::High),
                    step: Output::new(&mut p.PIN_14, Level::Low),
                    dir: Output::new(&mut p.PIN_13, Level::Low),
                },
            ],
            driver_serial,
            host_serial,
        }
    }
}
