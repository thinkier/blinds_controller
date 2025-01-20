#![no_std]
#![no_main]

mod board;
mod checks;
mod comms;
mod driver;

use crate::board::*;
use crate::checks::all_checks;
use crate::comms::{IncomingRpcPacket, InstructionBuffer, OutgoingRpcPacket, RpcHandle};
use crate::driver::{dir_hold, stp_fall, stp_rise};
use blinds_sequencer::{
    HaltingSequencer, SensingWindowDressingSequencer, WindowDressingInstruction,
    WindowDressingSequencer,
};
use core::sync::atomic::Ordering;
use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::peripherals::{CORE1, WATCHDOG};
use embassy_rp::watchdog::Watchdog;
use embassy_rp::Peripherals;
use embassy_time::{Duration, Ticker, Timer};
use portable_atomic::AtomicU8;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static CORE1_EXECUTOR: StaticCell<Executor> = StaticCell::new();
static mut CORE1_STACK: Stack<8192> = Stack::new();
static REVERSALS: AtomicU8 = AtomicU8::new(0);
static STOPS: AtomicU8 = AtomicU8::new(0);
type IBuf = InstructionBuffer<WindowDressingInstruction, DRIVERS>;
static LOOK_AHEAD_BUFFER: IBuf = IBuf::new();
static PERIPH: StaticCell<Peripherals> = StaticCell::new();
static mut SERIAL_BUFFERS: SerialBuffers = SerialBuffers::default();
static SEQUENCERS: StaticCell<[HaltingSequencer<1024>; 4]> = StaticCell::new();

pub const DRIVERS: usize = 4;

// A shame that I can't use a const generic here to fit to the number of drivers according to the BSP
#[embassy_executor::task]
async fn main1(mut chs: [DriverPins<'static>; DRIVERS]) {
    let mut ticker = Ticker::every(Duration::from_micros(400)); // Actual ~= 1625 half-steps per second

    let mut cur_buf: [Option<WindowDressingInstruction>; DRIVERS] = [None; DRIVERS];

    loop {
        ticker.next().await;
        let reversal = REVERSALS.load(Ordering::Relaxed);
        for i in 0..DRIVERS {
            if cur_buf[i].is_none() {
                cur_buf[i] = LOOK_AHEAD_BUFFER.take(i);
                cur_buf[i].iter_mut().for_each(|instr| {
                    if (reversal >> i) & 1 != 0 {
                        instr.quality = instr.quality.reverse();
                    }
                });
            }
        }

        let stops = STOPS.swap(0, Ordering::Relaxed);
        for i in 0..DRIVERS {
            if (stops << i) & 1 != 0 {
                cur_buf[i] = None;
            }
        }

        chs.iter_mut().enumerate().for_each(|(i, ch)| {
            dir_hold(ch, cur_buf[i].as_ref().map(|i| i.quality));
        });
        // Realistically though, that's 3 CPU cycles...
        Timer::after_nanos(20).await; // $t_{dsh}$ & $t_{dsu}$ as per datasheet

        chs.iter_mut().enumerate().for_each(|(i, ch)| {
            stp_rise(ch, &mut cur_buf[i]);
        });
        Timer::after_nanos(100).await; // $t_{sh}$ as per datasheet

        chs.iter_mut().for_each(|ch| {
            stp_fall(ch);
        });
        Timer::after_nanos(100).await; // $t_{sl}$ as per datasheet
    }
}

#[embassy_executor::main]
async fn main0(_spawner: Spawner) {
    all_checks();
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
            core1_executor.run(|spawner| spawner.spawn(main1(board.drivers)).unwrap())
        });
    }

    let mut wdt = Watchdog::new(unsafe { WATCHDOG::steal() });
    wdt.pause_on_debug(true);
    wdt.start(Duration::from_secs(1));
    let mut rpc = RpcHandle::<256, _>::new(board.host_serial, wdt);
    let _ = rpc.write(&OutgoingRpcPacket::Ready {});

    let seq = SEQUENCERS.init([
        HaltingSequencer::new_roller(100_000),
        HaltingSequencer::new_roller(100_000),
        HaltingSequencer::new_roller(100_000),
        HaltingSequencer::new_roller(100_000),
    ]);
    loop {
        match rpc.read() {
            Ok(Some(packet)) => match packet {
                IncomingRpcPacket::Home { channel } => {
                    seq[channel as usize].home_fully_opened();
                }
                IncomingRpcPacket::Setup {
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
                        REVERSALS.bit_set(channel as u32, Ordering::Relaxed);
                    } else {
                        REVERSALS.bit_clear(channel as u32, Ordering::Relaxed);
                    }
                }
                IncomingRpcPacket::Set {
                    channel,
                    position,
                    tilt,
                } => {
                    position.map(|p| seq[channel as usize].set_position(p));
                    tilt.map(|t| seq[channel as usize].set_tilt(t));
                }
                IncomingRpcPacket::Get { channel } => {
                    if let Err(e) = rpc.write(&OutgoingRpcPacket::Position {
                        channel,
                        current: *seq[channel as usize].get_current_state(),
                        desired: *seq[channel as usize].get_desired_state(),
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

        let mut stops = 0;
        for i in 0..DRIVERS {
            if !LOOK_AHEAD_BUFFER.has(i) {
                if let Some(instr) = seq[i].get_next_instruction() {
                    defmt::info!("Sending instruction to driver {}", i);
                    LOOK_AHEAD_BUFFER.put(i, instr);
                }
            }

            if board.end_stops[i].is_high() {
                stops |= 1 << i;
            }
        }
        STOPS.or(stops, Ordering::Relaxed);
    }
}
