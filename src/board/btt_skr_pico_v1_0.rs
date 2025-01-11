use crate::board::{Board, DriverPins, SerialBuffers};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{UART0, UART1};
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, Config, Uart};
use embassy_rp::Peripherals;

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
    UART1_IRQ => BufferedInterruptHandler<UART1>;
});

impl<'a> Board<'a, 4, BufferedUart<'a, UART1>, BufferedUart<'a, UART0>> {
    pub fn init(p: &'a mut Peripherals, serial_buffers: &'a mut SerialBuffers) -> Self {
        let mut uart_cfg = Config::default();
        uart_cfg.baudrate = 115200;

        let driver_serial = Uart::new_blocking(&mut p.UART1, &mut p.PIN_8, &mut p.PIN_9, uart_cfg)
            .into_buffered(
                Irqs,
                &mut serial_buffers.driver_tx_buf,
                &mut serial_buffers.driver_rx_buf,
            );
        let host_serial = Uart::new_blocking(&mut p.UART0, &mut p.PIN_0, &mut p.PIN_1, uart_cfg)
            .into_buffered(
                Irqs,
                &mut serial_buffers.host_tx_buf,
                &mut serial_buffers.host_rx_buf,
            );

        let end_stops = [
            Input::new(&mut p.PIN_4, Pull::Down),
            Input::new(&mut p.PIN_25, Pull::Down),
            Input::new(&mut p.PIN_3, Pull::Down),
            Input::new(&mut p.PIN_16, Pull::Down),
        ];
        let driver = [
            DriverPins {
                enable: Output::new(&mut p.PIN_12, Level::High),
                step: Output::new(&mut p.PIN_11, Level::Low),
                dir: Output::new(&mut p.PIN_10, Level::Low),
            },
            DriverPins {
                enable: Output::new(&mut p.PIN_2, Level::High),
                step: Output::new(&mut p.PIN_19, Level::Low),
                dir: Output::new(&mut p.PIN_28, Level::Low),
            },
            DriverPins {
                enable: Output::new(&mut p.PIN_7, Level::High),
                step: Output::new(&mut p.PIN_6, Level::Low),
                dir: Output::new(&mut p.PIN_5, Level::Low),
            },
            DriverPins {
                enable: Output::new(&mut p.PIN_15, Level::High),
                step: Output::new(&mut p.PIN_14, Level::Low),
                dir: Output::new(&mut p.PIN_13, Level::Low),
            },
        ];

        Self {
            end_stops,
            drivers: driver,
            driver_serial,
            host_serial,
        }
    }
}
