#![no_std]
#![no_main]

mod board;
mod checks;
mod comms;

use crate::board::*;
use crate::board::{CountedSqrWav, CountedSqrWavProgram};
use crate::checks::all_checks;
use crate::comms::{IncomingRpcPacket, InstructionBuffer, OutgoingRpcPacket, RpcHandle};
use blinds_sequencer::{
    Direction, HaltingSequencer, SensingWindowDressingSequencer, WindowDressingInstruction,
    WindowDressingSequencer,
};
use core::mem;
use core::sync::atomic::Ordering;
use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::peripherals::{CORE1, PIN_11, PIN_14, PIN_19, PIN_6, PIO0, WATCHDOG};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::watchdog::Watchdog;
use embassy_rp::{bind_interrupts, Peripherals};
use embassy_time::{Duration, Instant, Ticker, Timer};
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
static PIO0: StaticCell<Pio<PIO0>> = StaticCell::new();
static PROG: StaticCell<CountedSqrWavProgram<PIO0>> = StaticCell::new();

pub const DRIVERS: usize = 4;
pub const FREQUENCY: u16 = 2000;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

// A shame that I can't use a const generic here to fit to the number of drivers according to the BSP
#[embassy_executor::task]
async fn main1(mut chs: [DriverPins<'static>; DRIVERS]) {
    let pio = PIO0.init(Pio::new(unsafe { PIO0::steal() }, Irqs));
    let prog = PROG.init(CountedSqrWavProgram::new(&mut pio.common));
    let mut ch0 = CountedSqrWav::new(
        &mut pio.common,
        &mut pio.sm0,
        unsafe { PIN_11::steal() },
        prog,
        FREQUENCY,
    );
    let mut ch1 = CountedSqrWav::new(
        &mut pio.common,
        &mut pio.sm1,
        unsafe { PIN_19::steal() },
        prog,
        FREQUENCY,
    );
    let mut ch2 = CountedSqrWav::new(
        &mut pio.common,
        &mut pio.sm2,
        unsafe { PIN_6::steal() },
        prog,
        FREQUENCY,
    );
    let mut ch3 = CountedSqrWav::new(
        &mut pio.common,
        &mut pio.sm3,
        unsafe { PIN_14::steal() },
        prog,
        FREQUENCY,
    );

    macro_rules! run_on_channel {
        ($ch:expr, $run:path $(,$args:expr)*) => {
            match $ch {
                0 => $run(&mut ch0 $(,$args)*),
                1 => $run(&mut ch1 $(,$args)*),
                2 => $run(&mut ch2 $(,$args)*),
                3 => $run(&mut ch3 $(,$args)*),
                _ => defmt::unreachable!(
                    "Is there more than {} channels for this board?",
                    DRIVERS
                ),
            }
        };
    }

    // Limit the ticks to prevent lock starvation
    let mut ticker = Ticker::every(Duration::from_micros(100));

    let mut direction: [Direction; DRIVERS] = [Direction::Hold; DRIVERS];
    let mut ready_at: [Instant; DRIVERS] = [Instant::now(); DRIVERS];
    let mut cur_buf: [Option<WindowDressingInstruction>; DRIVERS] = [None; DRIVERS];

    loop {
        ticker.next().await;
        // for i in 0..DRIVERS {
        //     let i = 0;
        //     defmt::info!(
        //         "Channel {}: stopped={}, ready={}",
        //         i,
        //         run_on_channel!(i, CountedSqrWav::stopped),
        //         run_on_channel!(i, CountedSqrWav::ready)
        //     );
        // }

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
        let now = Instant::now();
        for i in 0..DRIVERS {
            if (stops << i) & 1 != 0 {
                direction[i] = Direction::Hold;
                cur_buf[i] = None;
                run_on_channel!(i, CountedSqrWav::kill);
            }

            // Skip if the channel isn't ready
            if now < ready_at[i] {
                continue;
            }

            if run_on_channel!(i, CountedSqrWav::ready) {
                if let Some(instr) = mem::replace(&mut cur_buf[i], None) {
                    chs[i].enable.set_low();
                    if run_on_channel!(i, CountedSqrWav::stopped) {
                        // Direction changes may only occur when the channel is no longer producing phases
                        direction[i] = instr.quality;

                        match instr.quality {
                            Direction::Hold => {
                                // Hold instructions are handled by CPU clock instants
                                let ready = Duration::from_micros(
                                    1_000_000 * instr.quantity as u64 / FREQUENCY as u64,
                                );
                                ready_at[i] = now + ready;

                                // Stop further commands on the PIO SMs & move on to the next channel
                                // Also stops the instruction being placed back into the buffer (as this block handles it)
                                continue;
                            }
                            Direction::Retract => chs[i].dir.set_high(),
                            Direction::Extend => chs[i].dir.set_low(),
                        }
                    }

                    if direction[i] == instr.quality {
                        // If the direction has not changed, it may be pushed without interruption
                        // try_push will always succeed because we just checked CountedSqrWav::ready
                        run_on_channel!(i, CountedSqrWav::try_push, instr.quantity);
                    } else {
                        // Place the instruction back to the buffer if it was not executed
                        let _ = mem::replace(&mut cur_buf[i], Some(instr));
                    }
                } else if run_on_channel!(i, CountedSqrWav::stopped) {
                    // If there are no instructions, and the remaining instruction is completed, the channel will be disabled
                    chs[i].enable.set_high();
                }
            }
        }
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

    let wdt = Watchdog::new(unsafe { WATCHDOG::steal() });
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
                if let Some(instr) = seq[i].get_next_instruction_grouped(FREQUENCY as u32) {
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
