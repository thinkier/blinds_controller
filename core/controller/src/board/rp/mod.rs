use crate::board::rp::utils::counted_sqr_wav_pio::CountedSqrWav;
use crate::board::{ConfigurableStepStickHost, ControllableBoard, StepStickHost};
#[cfg(feature = "host-uart")]
use crate::rpc::SerialRpcHandle;
#[cfg(feature = "host-usb")]
use crate::rpc::UsbRpcHandle;
use crate::{DRIVERS, STOPS};
use core::sync::atomic::Ordering;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output};
use embassy_rp::peripherals::PIO0;
#[cfg(any(feature = "driver-qty-5", feature = "driver-qty-8"))]
use embassy_rp::peripherals::PIO1;
use embassy_rp::watchdog::Watchdog;
use embassy_time::{Timer};
#[cfg(feature = "host-usb")]
use embassy_usb::driver::Driver;
#[cfg(feature = "host-uart")]
use embedded_io_async::{ErrorType, ReadReady};
use embedded_io_async::{Read, Write};

pub mod utils;

pub struct DriverPins<'a> {
    pub enable: Output<'a>,
    // pub step: Output<'a>,
    pub dir: Output<'a>,
}

pub struct Board<'a, const N: usize, D, H, T> {
    pub drivers: [DriverPins<'a>; N],
    pub driver_serial: D,
    pub host_rpc: H,
    pub wdr: Watchdog,
    // Implementer defined, useful for debugging or carrying any information that
    // the controller does not care about
    pub board_state: T,
    // State machines - alternative to an ACT timer on STM controllers
    pub pio0_0: Option<CountedSqrWav<'a, PIO0, 0>>,
    pub pio0_1: Option<CountedSqrWav<'a, PIO0, 1>>,
    pub pio0_2: Option<CountedSqrWav<'a, PIO0, 2>>,
    pub pio0_3: Option<CountedSqrWav<'a, PIO0, 3>>,
    #[cfg(any(feature = "driver-qty-5", feature = "driver-qty-8"))]
    pub pio1_0: Option<CountedSqrWav<'a, PIO1, 0>>,
    #[cfg(feature = "driver-qty-8")]
    pub pio1_1: Option<CountedSqrWav<'a, PIO1, 1>>,
    #[cfg(feature = "driver-qty-8")]
    pub pio1_2: Option<CountedSqrWav<'a, PIO1, 2>>,
    #[cfg(feature = "driver-qty-8")]
    pub pio1_3: Option<CountedSqrWav<'a, PIO1, 3>>,
}

#[cfg(feature = "host-uart")]
impl<'a, const N: usize, const BS: usize, D, IO, T> ControllableBoard
    for Board<'a, N, D, SerialRpcHandle<BS, IO>, T>
where
    IO: Read + ReadReady + Write,
    <IO as ErrorType>::Error: defmt::Format,
{
    type Rpc = SerialRpcHandle<BS, IO>;

    fn get_host_rpc(&mut self) -> &mut Self::Rpc {
        &mut self.host_rpc
    }

    fn reset(&mut self) {
        self.wdr.trigger_reset();
    }

    fn enter_bootloader(&mut self) {
        embassy_rp::rom_data::reset_to_usb_boot(0, 0);
    }

    fn watchdog_feed(&mut self) {
        // According to https://arduino-pico.readthedocs.io/en/latest/rp2040.html
        // The maximum value is 8.3 seconds.  Any higher values will be truncated by the hardware.
        self.wdr.feed(embassy_time::Duration::from_secs(2))
    }
}

#[cfg(feature = "host-usb")]
impl<const N: usize, const BS: usize, D, HD, T> ControllableBoard
    for Board<'static, N, D, UsbRpcHandle<BS, HD>, T>
where
    HD: Driver<'static>,
{
    type Rpc = UsbRpcHandle<BS, HD>;

    fn get_host_rpc(&mut self) -> &mut Self::Rpc {
        &mut self.host_rpc
    }

    fn reset(&mut self) {
        self.wdr.trigger_reset();
    }

    fn enter_bootloader(&mut self) {
        embassy_rp::rom_data::reset_to_usb_boot(0, 0);
    }
}

impl<'a, const N: usize, D, H, T> StepStickHost for Board<'a, N, D, H, T> {
    fn get_enabled(&mut self, channel: usize) -> bool {
        self.drivers[channel].enable.is_set_low()
    }

    fn set_enabled(&mut self, channel: usize, enabled: bool) {
        self.drivers[channel].dir.set_level(if enabled { Level::Low } else { Level::High });

    }

    fn set_direction(&mut self, channel: usize, invert: bool) {
        self.drivers[channel].dir.set_level(if invert { Level::High } else { Level::Low });
    }

    fn get_stopped(&mut self, channel: usize) -> bool {
        match channel {
            0 => self.pio0_0.as_mut().map(|p| p.stopped()).unwrap_or(true),
            1 => self.pio0_1.as_mut().map(|p| p.stopped()).unwrap_or(true),
            2 => self.pio0_2.as_mut().map(|p| p.stopped()).unwrap_or(true),
            3 => self.pio0_3.as_mut().map(|p| p.stopped()).unwrap_or(true),
            #[cfg(any(feature = "driver-qty-5", feature = "driver-qty-8"))]
            4 => self.pio1_0.as_mut().map(|p| p.stopped()).unwrap_or(true),
            #[cfg(feature = "driver-qty-8")]
            5 => self.pio1_1.as_mut().map(|p| p.stopped()).unwrap_or(true),
            #[cfg(feature = "driver-qty-8")]
            6 => self.pio1_2.as_mut().map(|p| p.stopped()).unwrap_or(true),
            #[cfg(feature = "driver-qty-8")]
            7 => self.pio1_3.as_mut().map(|p| p.stopped()).unwrap_or(true),
            _ => true,
        }
    }

    fn get_ready_for_steps(&mut self, channel: usize) -> bool {
        match channel {
            0 => self.pio0_0.as_mut().map(|p| p.ready()).unwrap_or(false),
            1 => self.pio0_1.as_mut().map(|p| p.ready()).unwrap_or(false),
            2 => self.pio0_2.as_mut().map(|p| p.ready()).unwrap_or(false),
            3 => self.pio0_3.as_mut().map(|p| p.ready()).unwrap_or(false),
            #[cfg(any(feature = "driver-qty-5", feature = "driver-qty-8"))]
            4 => self.pio1_0.as_mut().map(|p| p.ready()).unwrap_or(false),
            #[cfg(feature = "driver-qty-8")]
            5 => self.pio1_1.as_mut().map(|p| p.ready()).unwrap_or(false),
            #[cfg(feature = "driver-qty-8")]
            6 => self.pio1_2.as_mut().map(|p| p.ready()).unwrap_or(false),
            #[cfg(feature = "driver-qty-8")]
            7 => self.pio1_3.as_mut().map(|p| p.ready()).unwrap_or(false),
            _ => false,
        }
    }

    fn add_steps(&mut self, channel: usize, steps: u32) -> Option<bool> {
        if steps == 0 {
            return None;
        }

        match channel {
            0 => self.pio0_0.as_mut().map(|p| p.try_push(steps)),
            1 => self.pio0_1.as_mut().map(|p| p.try_push(steps)),
            2 => self.pio0_2.as_mut().map(|p| p.try_push(steps)),
            3 => self.pio0_3.as_mut().map(|p| p.try_push(steps)),
            #[cfg(any(feature = "driver-qty-5", feature = "driver-qty-8"))]
            4 => self.pio1_0.as_mut().map(|p| p.try_push(steps)),
            #[cfg(feature = "driver-qty-8")]
            5 => self.pio1_1.as_mut().map(|p| p.try_push(steps)),
            #[cfg(feature = "driver-qty-8")]
            6 => self.pio1_2.as_mut().map(|p| p.try_push(steps)),
            #[cfg(feature = "driver-qty-8")]
            7 => self.pio1_3.as_mut().map(|p| p.try_push(steps)),
            _ => None,
        }
    }

    fn clear_steps(&mut self, channel: usize) {
        match channel {
            0 => self.pio0_0.as_mut().map(|p| p.clear()),
            1 => self.pio0_1.as_mut().map(|p| p.clear()),
            2 => self.pio0_2.as_mut().map(|p| p.clear()),
            3 => self.pio0_3.as_mut().map(|p| p.clear()),
            #[cfg(any(feature = "driver-qty-5", feature = "driver-qty-8"))]
            4 => self.pio1_0.as_mut().map(|p| p.clear()),
            #[cfg(feature = "driver-qty-8")]
            5 => self.pio1_1.as_mut().map(|p| p.clear()),
            #[cfg(feature = "driver-qty-8")]
            6 => self.pio1_2.as_mut().map(|p| p.clear()),
            #[cfg(feature = "driver-qty-8")]
            7 => self.pio1_3.as_mut().map(|p| p.clear()),
            _ => None,
        };
    }
}

#[cfg(feature = "uart_configurable_driver_async")]
impl<'a, const N: usize, D, H, T> ConfigurableStepStickHost<N> for Board<'a, N, D, H, T>
where
    D: Read + Write,
{
    type DriverSerial = D;

    fn driver_serial(&mut self, _addr: u8) -> &mut Self::DriverSerial {
        &mut self.driver_serial
    }
}

pub fn bind_endstops<const N: usize>(spawner: Spawner, inputs: [Input<'static>; N]) {
    let mut i = 0;
    for stop in inputs {
        let _ = spawner.spawn(stop_detector(i, stop).unwrap());
        i += 1;
    }
}

/// Not universally compatible
///
/// See: https://docs.embassy.dev/embassy-rp/git/rp2040/gpio/struct.Input.html
#[embassy_executor::task(pool_size = DRIVERS)]
async fn stop_detector(i: usize, mut input: Input<'static>) {
    loop {
        debug!("Waiting for endstop event on {}", i);
        input.wait_for_high().await;
        debug!("Endstop HIGH detected for channel {}", i);
        STOPS.bit_set(i as u32, Ordering::Release);
        input.wait_for_low().await;
        debug!("Endstop LOW detected for channel {}", i);
        Timer::after_secs(1).await; // Dead Time Insertion
    }
}
