use controller::board::stm32::{Board, DriverPins};
use controller::rpc::usb_cdc_acm::{UsbCdcAcmStream, UsbRpcHandle};
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::peripherals::USB;
use embassy_stm32::usart::BufferedUart;
use embassy_stm32::usb::{Driver, InterruptHandler};
use embassy_usb::UsbDevice;

bind_interrupts!(struct Irqs {
    USB_UCPD1_2 => InterruptHandler<USB>;
});

pub trait BoardInitialize {
    fn init(spawner: Spawner) -> Self;
}

impl BoardInitialize for Board<'static, 5, BufferedUart<'static>, UsbCdcAcmStream<'static, Driver<'static, USB>>> {
    fn init(spawner: Spawner) -> Self {
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
            None,
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

        let mut driver_serial = [
            // PB8,
            // PC9,
            // PD0,
            // PD1,
            // PB5
        ];

        let usb_driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);
        let (usb_device, host_rpc) = UsbCdcAcmStream::init(usb_driver);
        let _ = spawner.spawn(usb_task(usb_device));

        Board {
            end_stops,
            drivers,
            driver_serial,
            host_rpc,
        }
    }
}

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, Driver<'static, USB>>) {
    usb.run().await;
}
