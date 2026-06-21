use controller::board::rp::utils::counted_sqr_wav_pio::{CountedSqrWav, CountedSqrWavProgram};
use controller::board::rp::{bind_endstops, Board, DriverPins};
#[cfg(feature = "host-uart")]
use controller::rpc::SerialRpcHandle;
#[cfg(feature = "host-usb")]
use controller::rpc::{UsbCdcAcmStream, UsbRpcHandle};
use controller::static_buffer;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{PIO0, UART0, UART1, USB};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, Config};
#[cfg(feature = "host-usb")]
use embassy_rp::usb::Driver;
use embassy_rp::usb::InterruptHandler as UsbInterruptHandler;
use embassy_rp::Peripherals;
#[cfg(feature = "host-usb")]
use embassy_usb::UsbDevice;
use static_cell::StaticCell;

pub const FREQUENCY: u16 = 1000;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    UART0_IRQ => BufferedInterruptHandler<UART0>;
    UART1_IRQ => BufferedInterruptHandler<UART1>;
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
});

static_buffer!(DRIVER_BUFFER_TX: 32);
static_buffer!(DRIVER_BUFFER_RX: 128);
static_buffer!(HOST_BUFFER_TX: 256);
static_buffer!(HOST_BUFFER_RX: 1024);

static PERIPHERALS: StaticCell<Peripherals> = StaticCell::new();
static PIO0: StaticCell<Pio<PIO0>> = StaticCell::new();
static PROG: StaticCell<CountedSqrWavProgram<PIO0>> = StaticCell::new();

pub trait BoardInitialize {
    fn init(spawner: Spawner) -> Self;
}

#[cfg(feature = "host-uart")]
pub type HD = SerialRpcHandle<128, BufferedUart>;
#[cfg(feature = "host-usb")]
pub type HD = UsbRpcHandle<'static, 128, Driver<'static, USB>>;

impl BoardInitialize for Board<'static, 4, BufferedUart, HD> {
    fn init(spawner: Spawner) -> Self {
        let p = PERIPHERALS.init(embassy_rp::init(Default::default()));
        let pio = PIO0.init(Pio::new(p.PIO0.reborrow(), Irqs));
        let prog = PROG.init(CountedSqrWavProgram::new(&mut pio.common));

        let pio0_0 = CountedSqrWav::new(
            &mut pio.common,
            &mut pio.sm0,
            p.PIN_11.reborrow(),
            prog,
            FREQUENCY,
        );
        let pio0_1 = CountedSqrWav::new(
            &mut pio.common,
            &mut pio.sm1,
            p.PIN_19.reborrow(),
            prog,
            FREQUENCY,
        );

        let pio0_2 = CountedSqrWav::new(
            &mut pio.common,
            &mut pio.sm2,
            p.PIN_6.reborrow(),
            prog,
            FREQUENCY,
        );

        let pio0_3 = CountedSqrWav::new(
            &mut pio.common,
            &mut pio.sm3,
            p.PIN_14.reborrow(),
            prog,
            FREQUENCY,
        );

        let mut uart_cfg = Config::default();
        uart_cfg.baudrate = 115200;

        let driver_serial = BufferedUart::new(
            p.UART1.reborrow(),
            p.PIN_8.reborrow(),
            p.PIN_9.reborrow(),
            Irqs,
            DRIVER_BUFFER_TX.take(),
            DRIVER_BUFFER_RX.take(),
            uart_cfg,
        );

        #[cfg(feature = "host-uart")]
        let host_rpc = {
            let host_serial = BufferedUart::new(
                p.UART0.reborrow(),
                p.PIN_0.reborrow(),
                p.PIN_1.reborrow(),
                Irqs,
                HOST_BUFFER_TX.take(),
                HOST_BUFFER_RX.take(),
                uart_cfg,
            );
            SerialRpcHandle::new(host_serial)
        };

        #[cfg(feature = "host-usb")]
        let host_rpc = {
            let usb_driver = Driver::new(p.USB.reborrow(), Irqs);
            let (usb_device, host_rpc) = UsbCdcAcmStream::init(usb_driver);
            let _ = spawner.spawn(usb_task(usb_device).unwrap());
            UsbRpcHandle::new(host_rpc)
        };

        bind_endstops(
            spawner,
            [
                Input::new(p.PIN_4.reborrow(), Pull::Down),
                Input::new(p.PIN_25.reborrow(), Pull::Down),
                Input::new(p.PIN_3.reborrow(), Pull::Down),
                Input::new(p.PIN_16.reborrow(), Pull::Down),
            ],
        );
        let driver = [
            DriverPins {
                enable: Output::new(p.PIN_12.reborrow(), Level::High),
                // step: Output::new(p.PIN_11.reborrow(), Level::Low),
                dir: Output::new(p.PIN_10.reborrow(), Level::Low),
            },
            DriverPins {
                enable: Output::new(p.PIN_2.reborrow(), Level::High),
                // step: Output::new(p.PIN_19.reborrow(), Level::Low),
                dir: Output::new(p.PIN_28.reborrow(), Level::Low),
            },
            DriverPins {
                enable: Output::new(p.PIN_7.reborrow(), Level::High),
                // step: Output::new(p.PIN_6.reborrow(), Level::Low),
                dir: Output::new(p.PIN_5.reborrow(), Level::Low),
            },
            DriverPins {
                enable: Output::new(p.PIN_15.reborrow(), Level::High),
                // step: Output::new(p.PIN_14.reborrow(), Level::Low),
                dir: Output::new(p.PIN_13.reborrow(), Level::Low),
            },
        ];

        Self {
            drivers: driver,
            driver_serial,
            host_rpc,
            pio0_0: Some(pio0_0),
            pio0_1: Some(pio0_1),
            pio0_2: Some(pio0_2),
            pio0_3: Some(pio0_3),
        }
    }
}

#[cfg(feature = "host-usb")]
#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, Driver<'static, USB>>) {
    usb.run().await;
}
