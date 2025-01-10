#![no_std]
#![no_main]

mod board;
mod comms;
mod driver;

use crate::board::*;
use crate::comms::{RpcHandle, RpcPacket};
use crate::driver::{all_pins, dir_hold, stp_fall, stp_rise};
use blinds_sequencer::{
    Direction, HaltingSequencer, SensingWindowDressingSequencer, WindowDressingInstruction,
    WindowDressingSequencer,
};
use core::sync::atomic::Ordering;
use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::peripherals::CORE1;
use embassy_rp::Peripherals;
use embassy_time::{Duration, Instant, Timer};
use portable_atomic::AtomicU8;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static CORE1_EXECUTOR: StaticCell<Executor> = StaticCell::new();
static mut CORE1_STACK: Stack<16384> = Stack::new();

static REVERSE_POLARITY: AtomicU8 = AtomicU8::new(0);

const DRIVERS: usize = 4;

// A shame that I can't use a const generic here to fit to the number of drivers according to the BSP
#[embassy_executor::task]
async fn main1(mut chs: [DriverPins<'static>; DRIVERS]) {
    loop {
        let period = Timer::after_micros(400); // Actual ~= 1625 half-steps per second

        let mut instr: Option<WindowDressingInstruction> = None;
        let pol = REVERSE_POLARITY.load(Ordering::Relaxed);

        all_pins(&mut chs, |ch| {
            dir_hold(ch, instr.as_ref().map(|i| i.quality));
        });
        Timer::after_nanos(20).await; // $t_{dsh}$ & $t_{dsu}$ as per datasheet
        stp_rise(&mut chs[3], &mut instr);
        all_pins(&mut chs, |ch| {
            stp_rise(ch, &mut instr);
        });
        Timer::after_nanos(100).await; // $t_{sh}$ as per datasheet
        all_pins(&mut chs, |ch| {
            stp_fall(ch);
        });
        Timer::after_nanos(100).await; // $t_{sl}$ as per datasheet

        period.await;
    }
}

static PERIPH: StaticCell<Peripherals> = StaticCell::new();
static mut SERIAL_BUFFERS: SerialBuffers = SerialBuffers::default();

#[embassy_executor::main]
async fn main0(_spawner: Spawner) {
    // Initialise Peripherals
    info!("Initialising Peripherals");
    let p = PERIPH.init(embassy_rp::init(Default::default()));

    // Once again, a single-purpose buffer that should not be allocated at runtime, so
    // it is allocated as a static mutable reference (unsafe)
    #[allow(static_mut_refs)]
    let serial_buffers = unsafe { &mut SERIAL_BUFFERS };

    let mut board = Board::init(p, serial_buffers);
    #[cfg(feature = "configurable_driver")]
    {
        board.configure_driver();
    }
    info!("Peripherals Initialised");

    {
        // Have to unsafely steal core1 because the spawner takes ownership it,
        // and by extension, all the peripherals that were meant to be references
        // so it will throw a whole spanner in the works on the BSP module I've just refactored out
        let core1 = unsafe { CORE1::steal() };
        // Not practical to use a StaticCell to allocate and reference the new stack safely due to
        // concerns around stack overflow with such a big chunk being thrown around, plus runtime
        // initialization of the stack provides no benefits as opposed to compile-time initialization
        // perhaps other than not needing the unsafe keyword
        #[allow(static_mut_refs)]
        let core1_stack = unsafe { &mut CORE1_STACK };

        spawn_core1(core1, core1_stack, || {
            let core1_executor = CORE1_EXECUTOR.init(Executor::new());
            core1_executor.run(|spawner| spawner.spawn(main1(board.driver)).unwrap())
        });
    }

    let mut rpc = RpcHandle::<256, _>::new(board.host_serial);

    let mut seq = [
        HaltingSequencer::new_roller(100_000),
        HaltingSequencer::new_roller(100_000),
        HaltingSequencer::new_roller(100_000),
        HaltingSequencer::new_roller(100_000),
    ];
    loop {
        match rpc.read() {
            Ok(Some(packet)) => match packet {
                RpcPacket::Home { channel } => {
                    seq[channel as usize].home_fully_opened();
                }
                RpcPacket::Setup {
                    channel,
                    init,
                    full_cycle_steps,
                    reverse,
                    full_tilt_steps,
                } => {
                    seq[channel as usize] =
                        HaltingSequencer::new(full_cycle_steps, full_tilt_steps);

                    seq[channel as usize].load_state(&init);

                    if reverse.unwrap_or(false) {
                        REVERSE_POLARITY.bit_set(channel as u32, Ordering::Relaxed);
                    } else {
                        REVERSE_POLARITY.bit_clear(channel as u32, Ordering::Relaxed);
                    }
                }
                RpcPacket::Position { channel, state } => {
                    seq[channel as usize].set_state(&state);
                }
                RpcPacket::GetPosition { channel } => {
                    if let Err(e) = rpc.write(&RpcPacket::Position {
                        channel,
                        state: *seq[channel as usize].get_current_state(),
                    }) {
                        error!("Failed to write Position: {:?}", e);
                    }
                }
            },
            Ok(None) => {
                Timer::after_millis(10).await;
            }
            Err(e) => {
                error!("Failed to read from host serial: {:?}", e);
            }
        }
    }
}
