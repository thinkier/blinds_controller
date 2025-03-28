#![no_std]

pub mod board;
pub mod rpc;

use crate::board::*;
use crate::rpc::{AsyncRpc, IncomingRpcPacket, OutgoingRpcPacket};
use core::mem;
use core::sync::atomic::Ordering;
use defmt::*;
use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use portable_atomic::AtomicU16;
use sequencer::{
    Direction, HaltingSequencer, SensingWindowDressingSequencer, WindowDressingInstruction,
    WindowDressingSequencer,
};
use static_cell::StaticCell;

static REVERSALS: AtomicU16 = AtomicU16::new(0);
static STOPS: AtomicU16 = AtomicU16::new(0);
static SEQUENCERS: StaticCell<[HaltingSequencer<1024>; DRIVERS]> = StaticCell::new();

pub const DRIVERS: usize = get_driver_count();

const fn get_driver_count() -> usize {
    if cfg!(feature = "driver-qty-4") {
        4
    } else if cfg!(feature = "driver-qty-5") {
        5
    } else if cfg!(feature = "driver-qty-8") {
        8
    } else if cfg!(feature = "driver-qty-10") {
        10
    } else {
        #[cfg(not(any(
            feature = "driver-qty-4",
            feature = "driver-qty-5",
            feature = "driver-qty-8",
            feature = "driver-qty-10"
        )))]
        compile_error!("One driver-qty-{n} flag MUST be defined!");
        0 // Unreachable
    }
}

pub const FREQUENCY: u16 = 1000;

pub async fn run<B, S, const N: usize>(_spawner: Spawner, mut board: B)
where
    B: StepStickBoard + ControllableBoard + ConfigurableBoard<N> + StallGuard<S, N>,
{
    info!("Initializing controller...");
    let _ = board
        .get_host_rpc()
        .write(&OutgoingRpcPacket::Ready {})
        .await;

    let seq = SEQUENCERS.init([
        HaltingSequencer::new_roller(100_000),
        HaltingSequencer::new_roller(100_000),
        HaltingSequencer::new_roller(100_000),
        HaltingSequencer::new_roller(100_000),
        #[cfg(any(feature = "driver-qty-ge-5"))]
        HaltingSequencer::new_roller(100_000),
        #[cfg(any(feature = "driver-qty-ge-8"))]
        HaltingSequencer::new_roller(100_000),
        #[cfg(any(feature = "driver-qty-ge-8"))]
        HaltingSequencer::new_roller(100_000),
        #[cfg(any(feature = "driver-qty-ge-8"))]
        HaltingSequencer::new_roller(100_000),
        #[cfg(any(feature = "driver-qty-ge-10"))]
        HaltingSequencer::new_roller(100_000),
        #[cfg(any(feature = "driver-qty-ge-10"))]
        HaltingSequencer::new_roller(100_000),
    ]);
    let mut next_buf: [Option<WindowDressingInstruction>; DRIVERS] = [None; DRIVERS];
    let mut next_resume = [Instant::now(); DRIVERS];
    let mut last_reversal = [Instant::now(); DRIVERS];
    let mut cur_direction = [Direction::Hold; DRIVERS];

    info!("Ready to accept calls");
    loop {
        Timer::after_millis(250).await;

        match board.get_host_rpc().read().await {
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
                    sgthrs,
                } => {
                    seq[channel as usize] =
                        HaltingSequencer::new(full_cycle_steps, full_tilt_steps);

                    seq[channel as usize].load_state(&init);

                    if reverse.unwrap_or(false) {
                        REVERSALS.bit_set(channel as u32, Ordering::Relaxed);
                    } else {
                        REVERSALS.bit_clear(channel as u32, Ordering::Relaxed);
                    }

                    if let Some(sgthrs) = sgthrs {
                        board.set_sg_threshold(channel, sgthrs).await;
                    }

                    let _ = board
                        .get_host_rpc()
                        .write(&OutgoingRpcPacket::Position {
                            channel,
                            current: *seq[channel as usize].get_current_state(),
                            desired: *seq[channel as usize].get_desired_state(),
                        })
                        .await;
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
                    let out = OutgoingRpcPacket::Position {
                        channel,
                        current: *seq[channel as usize].get_current_state(),
                        desired: *seq[channel as usize].get_desired_state(),
                    };

                    if let Err(e) = board.get_host_rpc().write(&out).await {
                        error!("Failed to write Position: {:?}", e);
                    }
                }
                IncomingRpcPacket::GetStallGuardResult { channel } => {
                    let sg_result = board.get_sg_result(channel).await.unwrap_or(0);
                    let out = OutgoingRpcPacket::StallGuardResult { channel, sg_result };

                    if let Err(e) = board.get_host_rpc().write(&out).await {
                        error!("Failed to write StallGuardResult: {:?}", e);
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

        let stops = STOPS.swap(0, Ordering::AcqRel);

        for i in 0..DRIVERS {
            let now = Instant::now();

            if (stops >> i) & 0b1 == 1 && last_reversal[i] < now + Duration::from_millis(500) {
                defmt::info!("Endstop triggered");
                seq[i].trig_endstop();
                next_buf[i] = None;
                board.clear_steps(i);

                let _ = board
                    .get_host_rpc()
                    .write(&OutgoingRpcPacket::Position {
                        channel: i as u8,
                        current: *seq[i].get_current_state(),
                        desired: *seq[i].get_desired_state(),
                    })
                    .await;
            }

            if board.is_ready_for_steps(i) {
                if let Some(instr) = mem::replace(&mut next_buf[i], None) {
                    board.set_enabled(i, true);
                    if instr.quality == cur_direction[i] {
                        board.add_steps(i, instr.quantity);
                    } else if board.is_stopped(i) && next_resume[i] < now {
                        cur_direction[i] = instr.quality;
                        last_reversal[i] = now;

                        let _ = board
                            .get_host_rpc()
                            .write(&OutgoingRpcPacket::Position {
                                channel: i as u8,
                                current: *seq[i].get_current_state(),
                                desired: *seq[i].get_desired_state(),
                            })
                            .await;

                        match instr.quality {
                            Direction::Hold => {
                                let offset = Duration::from_micros(
                                    (instr.quantity as u64 * 1_000_000) / FREQUENCY as u64,
                                );
                                next_resume[i] = now + offset;

                                // Stop further commands on the PIO SMs & move on to the next channel
                                // Also stops the instruction being placed back into the buffer (as this block handles it)
                                continue;
                            }
                            Direction::Retract => board.set_direction(
                                i,
                                (REVERSALS.load(Ordering::Acquire) >> i) & 0b1 == 1,
                            ),
                            Direction::Extend => board.set_direction(
                                i,
                                (REVERSALS.load(Ordering::Acquire) >> i) & 0b1 == 0,
                            ),
                        }
                        board.add_steps(i, instr.quantity);
                    } else {
                        let _ = mem::replace(&mut next_buf[i], Some(instr));
                    }
                } else if let Some(next) = seq[i].get_next_instruction_grouped(FREQUENCY as u32) {
                    next_buf[i] = Some(next);
                } else if board.is_stopped(i) {
                    board.set_enabled(i, false);
                }
            }
        }
    }
}
