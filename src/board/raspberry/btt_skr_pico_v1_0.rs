use crate::board::raspberry::counted_sqr_wav_pio::{CountedSqrWav, CountedSqrWavProgram};
use crate::board::raspberry::{Board, DriverPins};
use crate::board::{ConfigurableBoard, SerialBuffers};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{PIO0, PIO1, UART0, UART1, WATCHDOG};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, Config, Uart};
use embassy_rp::Peripherals;
use embassy_rp::watchdog::Watchdog;
use embedded_io::{Read, Write};
use static_cell::StaticCell;
use crate::comms::RpcHandle;

pub const FREQUENCY: u16 = 1000;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    PIO1_IRQ_0 => InterruptHandler<PIO1>;
    UART0_IRQ => BufferedInterruptHandler<UART0>;
    UART1_IRQ => BufferedInterruptHandler<UART1>;
});

static PERIPHERALS: StaticCell<Peripherals> = StaticCell::new();
static PIO0: StaticCell<Pio<PIO0>> = StaticCell::new();
static PROG: StaticCell<CountedSqrWavProgram<PIO0>> = StaticCell::new();

impl Board<'static, 4, BufferedUart<'static, UART1>, BufferedUart<'static, UART0>> {
    pub fn init(serial_buffers: &'static mut SerialBuffers) -> Self {
        let p = PERIPHERALS.init(embassy_rp::init(Default::default()));
        let pio = PIO0.init(Pio::new(&mut p.PIO0, Irqs));
        let prog = PROG.init(CountedSqrWavProgram::new(&mut pio.common));

        let pio0_0 = CountedSqrWav::new(
            &mut pio.common,
            &mut pio.sm0,
            &mut p.PIN_11,
            prog,
            FREQUENCY,
        );
        let pio0_1 = CountedSqrWav::new(
            &mut pio.common,
            &mut pio.sm1,
            &mut p.PIN_19,
            prog,
            FREQUENCY,
        );

        let pio0_2 =
            CountedSqrWav::new(&mut pio.common, &mut pio.sm2, &mut p.PIN_6, prog, FREQUENCY);

        let pio0_3 = CountedSqrWav::new(
            &mut pio.common,
            &mut pio.sm3,
            &mut p.PIN_14,
            prog,
            FREQUENCY,
        );

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
        let host_rpc = RpcHandle::new(host_serial, Watchdog::new(unsafe { WATCHDOG::steal() }));

        let end_stops = [
            Some(Input::new(&mut p.PIN_4, Pull::Down)),
            Some(Input::new(&mut p.PIN_25, Pull::Down)),
            Some(Input::new(&mut p.PIN_3, Pull::Down)),
            Some(Input::new(&mut p.PIN_16, Pull::Down)),
        ];
        let driver = [
            DriverPins {
                enable: Output::new(&mut p.PIN_12, Level::High),
                // step: Output::new(&mut p.PIN_11, Level::Low),
                dir: Output::new(&mut p.PIN_10, Level::Low),
            },
            DriverPins {
                enable: Output::new(&mut p.PIN_2, Level::High),
                // step: Output::new(&mut p.PIN_19, Level::Low),
                dir: Output::new(&mut p.PIN_28, Level::Low),
            },
            DriverPins {
                enable: Output::new(&mut p.PIN_7, Level::High),
                // step: Output::new(&mut p.PIN_6, Level::Low),
                dir: Output::new(&mut p.PIN_5, Level::Low),
            },
            DriverPins {
                enable: Output::new(&mut p.PIN_15, Level::High),
                // step: Output::new(&mut p.PIN_14, Level::Low),
                dir: Output::new(&mut p.PIN_13, Level::Low),
            },
        ];

        Self {
            end_stops,
            drivers: driver,
            driver_serial,
            host_rpc,
            pio0_0: Some(pio0_0),
            pio0_1: Some(pio0_1),
            pio0_2: Some(pio0_2),
            pio0_3: Some(pio0_3),
            pio1_0: None,
            pio1_1: None,
            pio1_2: None,
            pio1_3: None,
        }
    }
}

impl<'a, const N: usize, D, H> ConfigurableBoard<N> for Board<'a, N, D, H>
where
    D: Read + Write,
{
    type DriverSerial = D;

    fn driver_serial(&mut self, _addr: u8) -> &mut Self::DriverSerial {
        &mut self.driver_serial
    }
}
