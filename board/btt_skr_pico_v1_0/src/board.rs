use controller::board::rp::utils::counted_sqr_wav_pio::{CountedSqrWav, CountedSqrWavProgram};
use controller::board::rp::{bind_endstops, Board, DriverPins};
use controller::board::ControlLoopInvoke;
#[cfg(feature = "host-uart")]
use controller::rpc::SerialRpcHandle;
#[cfg(feature = "host-usb")]
use controller::rpc::UsbRpcHandle;
use controller::static_buffer;
use defmt::{debug, error};
use embassy_executor::Spawner;
use embassy_rp::adc::{self, Adc};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{PIO0, UART0, UART1, USB};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_rp::uart::{self, BufferedInterruptHandler, BufferedUart};
#[cfg(feature = "host-usb")]
use embassy_rp::usb::Driver;
use embassy_rp::usb::InterruptHandler as UsbInterruptHandler;
use embassy_rp::watchdog::Watchdog;
use embassy_rp::Peripherals;
use embassy_time::Duration;
#[cfg(feature = "host-usb")]
use embassy_usb::UsbDevice;
use static_cell::StaticCell;
use thermistor::NtcThermistor;

pub const FREQUENCY: u16 = 1000;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    UART0_IRQ => BufferedInterruptHandler<UART0>;
    UART1_IRQ => BufferedInterruptHandler<UART1>;
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
});

static_buffer!(DRIVER_BUFFER_TX: 128);
static_buffer!(DRIVER_BUFFER_RX: 512);
#[cfg(feature = "host-uart")]
static_buffer!(HOST_BUFFER_TX: 1024);
#[cfg(feature = "host-uart")]
static_buffer!(HOST_BUFFER_RX: 2048);

static PERIPHERALS: StaticCell<Peripherals> = StaticCell::new();
static PIO0: StaticCell<Pio<PIO0>> = StaticCell::new();
static PROG: StaticCell<CountedSqrWavProgram<PIO0>> = StaticCell::new();

pub trait BoardInitialize {
    fn init(spawner: Spawner) -> Self;
}

#[cfg(feature = "host-uart")]
pub type HD = SerialRpcHandle<512, BufferedUart>;
#[cfg(feature = "host-usb")]
pub type HD = UsbRpcHandle<2048, Driver<'static, USB>>;

pub struct BttSkrPicoV1_0 {
    thermistor: NtcThermistor,
    adc: Adc<'static, adc::Blocking>,
    thermistor_pin: adc::Channel<'static>,
}

impl BoardInitialize for Board<'static, 4, BufferedUart, HD, BttSkrPicoV1_0> {
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

        let mut uart_cfg = uart::Config::default();
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
            let (usb_device, host_rpc) = UsbRpcHandle::new(usb_driver);
            let _ = spawner.spawn(usb_task(usb_device).unwrap());

            host_rpc
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

        let mut wdr = Watchdog::new(p.WATCHDOG.reborrow());
        #[cfg(test)]
        wdr.pause_on_debug(true);
        wdr.start(Duration::from_secs(2));

        Self {
            drivers: driver,
            driver_serial,
            host_rpc,
            wdr,
            board_state: BttSkrPicoV1_0 {
                thermistor: thermistor::ERT_J1VGXXA, // ERT-J1VG103FA from PBLS-1.0/27 EDLC (Supercapacitor)
                adc: Adc::new_blocking(p.ADC.reborrow(), adc::Config::default()),
                thermistor_pin: adc::Channel::new_pin(p.PIN_27.reborrow(), Pull::None),
            },
            pio0_0: Some(pio0_0),
            pio0_1: Some(pio0_1),
            pio0_2: Some(pio0_2),
            pio0_3: Some(pio0_3),
        }
    }
}

impl ControlLoopInvoke for BttSkrPicoV1_0 {
    async fn invoke(&mut self, _spawner: &mut Spawner) {
        let temp = self.measure_temp();
        debug!("Thermistor is at {}C", temp);
    }
}

impl BttSkrPicoV1_0 {
    fn read_thermistor_voltage(&mut self) -> u16 {
        match self.adc.blocking_read(&mut self.thermistor_pin) {
            Ok(thermistor) => thermistor,
            Err(e) => {
                error!("Failed to read thermistor: {}", e);
                1
            }
        }
    }

    /// The standard voltage divider formula is $V_{R_2} = \frac{V_{src}R_2}{R_1 + R_2}$. Transposed,
    /// $$
    /// R_2 = \frac{V_{R_2}R_1}{V_{src} - V_{R_2}}
    /// $$
    ///
    /// ADC measures voltage, and we want the resistance for our thermistor calculation, so let's move a few things...
    ///
    /// Mapping the thermistor ports to the standard voltage divider formula,
    /// you'll see that the thermistor is in the position of $R_2$, and we read $V_{R_2}$ from ADC
    /// based on <https://github.com/bigtreetech/SKR-Pico/blob/master/Hardware/BTT%20SKR%20Pico%20V1.0-SCH.pdf>
    ///
    /// Also from the same datasheet, $R_1$ is 4.7KOhms
    ///
    /// From the RP2040 specifications, the ADC is 12-bit (&therefore; value range $[0, 4096)$), which makes $V_{src} = 4095$
    // The intermediates for the $R_2$ value should be on the scale of ~24 bit, scaling up to u32 to prevent overflows
    // u32 maths is a lot faster than f32 non-hf maths
    fn measure_temp(&mut self) -> f32 {
        let v_src: u32 = 4095;
        let r_1: u32 = 4700;
        let v_r_2 = self.read_thermistor_voltage() as u32;

        let r_2 = (v_r_2 * r_1) / (v_src - v_r_2);

        self.thermistor.get_temp_celsius(r_2 as f32)
    }
}

#[cfg(feature = "host-usb")]
#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, Driver<'static, USB>>) {
    usb.run().await;
}
