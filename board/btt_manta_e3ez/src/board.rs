use controller::board::stm32::bitbanged_uart::BitbangedHalfDuplexUart;
use controller::board::stm32::{Board, DriverPins};
use controller::board::ControlLoopInvoke;
use controller::rpc::UsbRpcHandle;
use core::cell::RefCell;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::exti::{self, Channel, ExtiInput};
use embassy_stm32::gpio::{Flex, Level, Output, Pull, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::{EXTI0, EXTI5, TIM14, USB};
use embassy_stm32::timer::low_level::Timer;
use embassy_stm32::usb::{self, Driver as Stm32UsbDriver};
use embassy_usb::UsbDevice;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    EXTI0_1 => exti::InterruptHandler<<EXTI0 as Channel>::IRQ>;
    EXTI4_15 => exti::InterruptHandler<<EXTI5 as Channel>::IRQ>;
    USB_UCPD1_2 => usb::InterruptHandler<USB>;
});

pub trait BoardInitialize {
    fn init(spawner: Spawner) -> Self;
}

impl BoardInitialize
    for Board<
        'static,
        4,
        [BitbangedHalfDuplexUart<'static, Timer<'static, TIM14>>; 4],
        UsbRpcHandle<1024, Stm32UsbDriver<'static, USB>>,
        BttMantaE3ez,
    >
{
    fn init(spawner: Spawner) -> Self {
        let p = embassy_stm32::init(Default::default());

        let end_stops: [Option<ExtiInput<'static, Async>>; 4] = [
            // X ENN and X STEP are shared with SWCLK and SWDIO, absolutely banned
            // Some(ExtiInput::new(p.PC4, p.EXTI4, Pull::Down, Irqs)),
            Some(ExtiInput::new(p.PB0, p.EXTI0, Pull::Down, Irqs)),
            Some(ExtiInput::new(p.PC6, p.EXTI6, Pull::Down, Irqs)),
            Some(ExtiInput::new(p.PC5, p.EXTI5, Pull::Down, Irqs)),
            Some(ExtiInput::new(p.PB1, p.EXTI1, Pull::Down, Irqs)),
        ];

        let drivers = [
            // X ENN and X STEP are shared with SWCLK and SWDIO, absolutely banned
            // DriverPins {
            //     enable: Output::new(p.PA13, Level::Low, Speed::Low),
            //     // step // PA14 // SWCLK, USART2_TX, EVENTOUT, LPUART2_TX
            //     dir: Output::new(p.PA10, Level::Low, Speed::Low),
            // },
            DriverPins {
                enable: Output::new(p.PC14, Level::Low, Speed::Low),
                // step // PC8 // UCPD2_FRSTX, TIM3_CH3, TIM1_CH1, LPUART2_CTS
                dir: Output::new(p.PA15, Level::Low, Speed::Low),
            },
            DriverPins {
                enable: Output::new(p.PD3, Level::Low, Speed::Low),
                // step // PD2 // USART3_RTS_DE_CK, TIM3_ETR, TIM1_CH1N, USART5_RX
                dir: Output::new(p.PD4, Level::Low, Speed::Low),
            },
            DriverPins {
                enable: Output::new(p.PB3, Level::Low, Speed::Low),
                // step // PD5 // USART2_TX, SPI1_MISO/I2S1_MCK, TIM1_BKIN, USART5_CTS
                dir: Output::new(p.PD6, Level::Low, Speed::Low),
            },
            DriverPins {
                enable: Output::new(p.PB4, Level::Low, Speed::Low),
                // step // PB7 // USART1_RX, SPI2_MOSI/I2S2_SD, TIM17_CH1N, USART4_CTS, LPTIM1_IN2, I2C1_SDA, EVENTOUT, TIM4_CH2, LPUART2_RX
                dir: Output::new(p.PB6, Level::Low, Speed::Low),
            },
        ];

        let driver_timer = {
            static DRIVER_TIMER: StaticCell<RefCell<Timer<'static, TIM14>>> = StaticCell::new();

            DRIVER_TIMER.init(RefCell::new(Timer::new(p.TIM14)))
        };
        let driver_serial = [
            // X ENN and X STEP are shared with SWCLK and SWDIO, absolutely banned
            // BitbangedHalfDuplexUart::new(Flex::new(p.PB8), driver_timer), // CEC, SPI2_SCK/I2S2_CK, TIM16_CH1, FDCAN1_RX, USART3_TX, TIM15_BKIN, I2C1_SCL, EVENTOUT, USART6_TX, TIM4_CH3
            BitbangedHalfDuplexUart::new(Flex::new(p.PC9), driver_timer), // I2S_CKIN, TIM3_CH4, TIM1_CH2, LPUART2_RTS_DE, USB_NOE
            BitbangedHalfDuplexUart::new(Flex::new(p.PD0), driver_timer), // EVENTOUT, SPI2_NSS/I2S2_WS, TIM16_CH1, FDCAN1_RX
            BitbangedHalfDuplexUart::new(Flex::new(p.PD1), driver_timer), // EVENTOUT, SPI2_SCK/I2S2_CK, TIM17_CH1, FDCAN1_TX
            BitbangedHalfDuplexUart::new(Flex::new(p.PB5), driver_timer), // SPI1_MOSI/I2S1_SD, TIM3_CH2, TIM16_BKIN, FDCAN2_RX, LPTIM1_IN1, I2C1_SMBA, COMP2_OUT, USART5_RTS_DE_CK, SPI3_MOSI
        ];

        let usb_driver = Stm32UsbDriver::new(p.USB, Irqs, p.PA12, p.PA11);
        let (device, host_rpc) = UsbRpcHandle::new(usb_driver);

        let _ = spawner.spawn(usb_task(device).expect("Failed to spawn USB task"));

        Board {
            end_stops,
            drivers,
            driver_serial,
            host_rpc,
            board_state: BttMantaE3ez {},
        }
    }
}

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, Stm32UsbDriver<'static, USB>>) {
    usb.run().await;
}

pub struct BttMantaE3ez {}

impl ControlLoopInvoke for BttMantaE3ez {
    async fn invoke(&mut self, _spawner: &mut Spawner) {}
}
