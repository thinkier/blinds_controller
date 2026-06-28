use controller::board::stm32::bitbanged_uart::BitbangedHalfDuplexUart;
use controller::board::stm32::{Board, DriverPins};
use controller::board::ControlLoopInvoke;
use controller::rpc::{UsbCdcAcmStream, UsbRpcHandle};
use core::marker::PhantomData;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::exti::{self, Channel, ExtiInput};
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::{EXTI0, EXTI5, USB};
use embassy_stm32::usb::{self, Driver as UsbDriver};
use embassy_usb::UsbDevice;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    EXTI0_1 => exti::InterruptHandler<<EXTI0 as Channel>::IRQ>;
    EXTI4_15 => exti::InterruptHandler<<EXTI5 as Channel>::IRQ>;
    USB_UCPD1_2 => usb::InterruptHandler<USB>;
});

static USB_HANDLE: StaticCell<UsbCdcAcmStream<'static, UsbDriver<'static, USB>>> = StaticCell::new();

pub trait BoardInitialize {
    fn init(spawner: Spawner) -> Self;
}

impl BoardInitialize
    for Board<
        'static,
        4,
        BitbangedHalfDuplexUart<'static, ()>,
        UsbRpcHandle<'static, 1024, UsbCdcAcmStream<'static, UsbDriver<'static, USB>>>,
        BttMantaE3ez,
    >
{
    fn init(spawner: Spawner) -> Self {
        let mut p = embassy_stm32::init(Default::default());

        let end_stops: [Option<ExtiInput<'static, Async>>; 4] = [
            // X ENN and X STEP are shared with SWCLK and SWDIO, absolutely banned
            // Some(ExtiInput::new(p.PC4, p.EXTI4, Pull::Down, Irqs)),
            Some(ExtiInput::new(p.PB0, p.EXTI0, Pull::Down, Irqs)),
            Some(ExtiInput::new(p.PC6, p.EXTI6, Pull::Down, Irqs)),
            Some(ExtiInput::new(p.PC5, p.EXTI5, Pull::Down, Irqs)),
            Some(ExtiInput::new(p.PB1, p.EXTI1, Pull::Down, Irqs)),
        ];

        let drivers: [Option<DriverPins<'static>>; 4] = [
            // X ENN and X STEP are shared with SWCLK and SWDIO, absolutely banned
            // Some(DriverPins {
            //     enable: Output::new(p.PA13, Level::Low, Speed::Low),
            //     dir: Output::new(p.PA10, Level::Low, Speed::Low),
            // }),
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

        let driver_serial = [
            // X ENN and X STEP are shared with SWCLK and SWDIO, absolutely banned
            // BitbangedHalfDuplexUart {
            //     pin: PhantomData::default(), // PB8
            // },
            BitbangedHalfDuplexUart {
                pin: PhantomData::default(), // PC9
            },
            BitbangedHalfDuplexUart {
                pin: PhantomData::default(), // PD0
            },
            BitbangedHalfDuplexUart {
                pin: PhantomData::default(), // PD1
            },
            BitbangedHalfDuplexUart {
                pin: PhantomData::default(), // PB5
            },
        ];

        let usb_driver = UsbDriver::new(p.USB, Irqs, p.PA12, p.PA11);
        let (usb_device, host_serial) = UsbCdcAcmStream::init(usb_driver);
        let _ = spawner.spawn(usb_task(usb_device).expect("Failed to spawn USB task"));

        Board {
            end_stops,
            drivers,
            driver_serial,
            host_rpc: UsbRpcHandle::new(host_serial),
            board_state: BttMantaE3ez {},
        }
    }
}

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, UsbDriver<'static, USB>>) {
    usb.run().await;
}

pub struct BttMantaE3ez {}

impl ControlLoopInvoke for BttMantaE3ez {
    async fn invoke(&mut self, _spawner: &mut Spawner) {}
}
