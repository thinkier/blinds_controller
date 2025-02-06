use crate::board::stm32::{Board, DriverPins};
use crate::board::SerialBuffers;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::usart::BufferedUart;

impl Board<'static, 5, BufferedUart<'static>, BufferedUart<'static>> {
    pub fn init(serial_buffers: &'static mut SerialBuffers<5>) -> Self {
        let mut p = embassy_stm32::init(Default::default());

        let end_stops: [Option<ExtiInput<'static>>; 5] = [
            Some(ExtiInput::new(p.PC4, p.EXTI4, Pull::Down)),
            Some(ExtiInput::new(p.PB0, p.EXTI0, Pull::Down)),
            Some(ExtiInput::new(p.PC6, p.EXTI6, Pull::Down)),
            Some(ExtiInput::new(p.PC5, p.EXTI5, Pull::Down)),
            Some(ExtiInput::new(p.PB1, p.EXTI1, Pull::Down)),
        ];

        let drivers: [Option<DriverPins<'static>>; 5] = [
            // X ENN and X STEP are shared with SWCLK and SWDIO, no go...
            // DriverPins {
            //     enable: Output::new(p.PA13, Level::Low, Speed::Low),
            //     dir: Output::new(p.PA10, Level::Low, Speed::Low),
            // },
            None,
            Some(DriverPins {
                enable: Output::new(p.PC14, Level::Low, Speed::Low),
                dir: Output::new(p.PA15, Level::Low, Speed::Low),
            }),
            Some(DriverPins {
                enable: Output::new(p.PD3, Level::Low, Speed::Low),
                dir: Output::new(p.PD4, Level::Low, Speed::Low),
            }),
            Some(DriverPins {
                enable: Output::new(p.PB3, Level::Low, Speed::Low),
                dir: Output::new(p.PD6, Level::Low, Speed::Low),
            }),
            Some(DriverPins {
                enable: Output::new(p.PB4, Level::Low, Speed::Low),
                dir: Output::new(p.PB6, Level::Low, Speed::Low),
            }),
        ];

        let driver_serial;
        let host_rpc;

        Board {
            end_stops,
            drivers,
            driver_serial,
            host_rpc
        }
    }
}
