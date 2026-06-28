#![no_std]

pub mod board;
pub mod rpc;

use crate::board::*;
#[cfg(any(feature = "host-uart", feature = "host-usb"))]
use crate::rpc::{AsyncRpc, AsyncRpcError, IncomingRpcPacket, OutgoingRpcPacket};
use core::mem;
use core::sync::atomic::Ordering;
#[allow(unused)]
use defmt::*;
#[allow(unused)]
use embassy_executor::Spawner;
#[allow(unused)]
use embassy_time::Ticker;
#[allow(unused)]
use embassy_time::{Duration, Instant, Timer};
#[cfg(any(feature = "host-uart", feature = "host-usb"))]
use heapless::Vec;
use portable_atomic::AtomicU16;
use sequencer::{Direction, WindowDressingInstruction, WindowDressingSequencer};
#[cfg(feature = "stallguard")]
use sequencer::{HaltingSequencer, SensingWindowDressingSequencer};
#[cfg(feature = "stallguard")]
use static_cell::StaticCell;

pub const DRIVERS: usize = get_driver_count();

#[cfg(feature = "brownout-protection")]
const BROWNOUT_PROTECTION: Duration = Duration::from_secs(2);
static REVERSALS: AtomicU16 = AtomicU16::new(0);
#[cfg(feature = "stallguard")]
static STOPS: AtomicU16 = AtomicU16::new(0);
#[allow(unused)]
#[cfg(feature = "stallguard")]
static SEQUENCERS: StaticCell<[Option<HaltingSequencer<1024>>; DRIVERS]> = StaticCell::new();

const fn get_driver_count() -> usize {
    cfg_select! {
        feature = "driver-qty-4" => 4,
        feature = "driver-qty-5" => 5,
        feature = "driver-qty-8" => 8,
        feature = "driver-qty-10" => 10,
        _ => compile_error!("One driver-qty-{n} flag MUST be defined!")
    }
}

pub const FREQUENCY: u16 = 1000;

struct RunState<const N: usize> {
    #[cfg(feature = "brownout-protection")]
    brownout_protection: Instant,
    next_buf: [Option<WindowDressingInstruction>; N],
    next_resume: [Instant; N],
    cur_direction: [Direction; N],
}

impl<const N: usize> Default for RunState<N> {
    fn default() -> Self {
        RunState {
            #[cfg(feature = "brownout-protection")]
            brownout_protection: Instant::MIN,
            next_buf: [None; N],
            next_resume: [Instant::now(); N],
            cur_direction: [Direction::Hold; N],
        }
    }
}

#[cfg(all(
    any(feature = "host-uart", feature = "host-usb"),
    any(
        feature = "uart_configurable_driver",
        feature = "uart_configurable_driver_async"
    ),
    feature = "stallguard",
))]
#[allow(unused)]
pub async fn run<B, S, const N: usize>(mut spawner: Spawner, mut board: B)
where
    B: StepStickBoard
        + ControllableBoard
        + ConfigurableBoard<N>
        + StallGuard<S, N>
        + ControlLoopInvoke,
{
    info!("Initializing controller...");

    let seqs = SEQUENCERS.init(
        [const { None }; cfg_select! {
            feature = "driver-qty-4" => 4,
            feature = "driver-qty-5" => 5,
            feature = "driver-qty-8" => 8,
            feature = "driver-qty-10" => 10,
        }],
    );
    let mut state = RunState::<DRIVERS>::default();

    loop {
        board.watchdog_feed();
        let incoming = board.get_host_rpc().peek().await.unwrap_or(None);

        match incoming {
            Some(IncomingRpcPacket::Setup { .. }) => {
                debug!("Received setup command. Continuing...");
                break;
            }
            Some(_) => {
                debug!("Received non-setup command. Draining...");
                let _ = board.get_host_rpc().read().await;
                Timer::after_millis(50).await; // Drain should be more eager than the less-intensive waiting for a new command
                continue;
            }
            None => {
                let _ = board
                    .get_host_rpc()
                    .write(&OutgoingRpcPacket::Ready {})
                    .await;
                debug!("Flagged ready state.");
                Timer::after_secs(1).await;
            }
        }
    }

    let mut ticker = Ticker::every(Duration::from_millis(250));
    loop {
        board.watchdog_feed();
        board.invoke(&mut spawner).await;
        let mut request_pos = 0u16;

        // Limit the consumption of commands so it's not in this loop without checking the PIO,
        // But also make it a bit greedy
        for _ in 0..N {
            match board.get_host_rpc().read().await {
                Ok(Some(packet)) => match packet {
                    IncomingRpcPacket::Home { channel } => {
                        if let Some(ref mut seq) = seqs[channel as usize] {
                            seq.home_fully_opened();
                        } else {
                            emit_absence(&mut board, channel).await;
                        }
                    }
                    IncomingRpcPacket::Setup {
                        channel,
                        init,
                        full_cycle_steps,
                        reverse,
                        full_tilt_steps,
                        sgthrs,
                    } => {
                        let mut seq = HaltingSequencer::new(full_cycle_steps, full_tilt_steps);

                        if let Some(init) = init {
                            seq.load_state(&init);
                        } else if let Some(ref old_seq) = seqs[channel as usize] {
                            seq.load_state(old_seq.get_current_state());
                        }

                        if reverse.unwrap_or(false) {
                            REVERSALS.bit_set(channel as u32, Ordering::Relaxed);
                        } else {
                            REVERSALS.bit_clear(channel as u32, Ordering::Relaxed);
                        }

                        if let Some(sgthrs) = sgthrs {
                            cfg_select! {
                                feature = "uart_configurable_driver" => {
                                    board.set_sg_threshold(channel, sgthrs);
                                },
                                feature = "uart_configurable_driver_async" => {
                                    board.set_sg_threshold(channel, sgthrs).await;
                                }
                            }
                        }

                        seqs[channel as usize] = Some(seq);
                        info!("Driver set up on channel {}", channel);
                    }
                    IncomingRpcPacket::Set {
                        channel,
                        position,
                        tilt,
                    } => {
                        if let Some(ref mut seq) = seqs[channel as usize] {
                            position.map(|p| seq.set_position(p));
                            tilt.map(|t| seq.set_tilt(t));
                        } else {
                            emit_absence(&mut board, channel).await;
                        }
                    }
                    IncomingRpcPacket::Get { channel } => {
                        request_pos |= 0b1 << channel;
                    }
                    IncomingRpcPacket::GetStallGuardResult { channel } => {
                        let sg_result = board.get_sg_result_halved(channel).await.unwrap_or(0);
                        let out = OutgoingRpcPacket::StallGuardResult { channel, sg_result };

                        if let Err(e) = board.get_host_rpc().write(&out).await {
                            error!("Failed to write StallGuardResult: {:?}", e);
                        }

                        break; // This is a heavy command, yield after running this
                    }
                    IncomingRpcPacket::Bootloader => {
                        board.enter_bootloader();
                    }
                },
                Ok(None) => {
                    break;
                }
                Err(e) => {
                    warn!("Failed to read from host: {:?}", e);
                    if e.is_broken_input() {
                        error!("Emitting state before rebooting...");
                        let _ = bulk_emit_state(&mut board, seqs, 0xFFFF, true).await;

                        Timer::after_secs(5).await;
                        board.reset();
                    }
                }
            }
        }

        let stopped = bulk_endstop_check(&mut board, seqs, &mut state);
        let finished = bulk_push_pull_state(&mut board, seqs, &mut state);

        // Emit state due to interruption or completion
        bulk_emit_state(&mut board, seqs, finished | stopped, true).await;
        bulk_emit_state(&mut board, seqs, request_pos & !(finished | stopped), false).await;

        if option_env!("LOG_SG_RESULT").is_some() {
            let _ =
                print_sg_result(
                    &mut board,
                    seqs.iter().enumerate().fold(0u16, |x, (i, opt)| {
                        if opt.is_some() {
                            x | (1 << i)
                        } else {
                            x
                        }
                    }),
                )
                .await;
        }

        ticker.next().await;
    }
}

#[cfg(feature = "stallguard")]
async fn print_sg_result<B, S, const N: usize>(board: &mut B, channels: u16)
where
    B: StepStickBoard + StallGuard<S, N>,
{
    let mut sgresult2 = [None; N];

    // I do incur a bit of performance penalty querying all channels (used or not)
    // over a single UART and waiting for a response for every single one.
    // But at least I'm not creating a race condition in async.
    //
    // Also, this class doesn't discriminate for the underlying write protocol.
    // This is actually the worst case assumption.
    // I know the Manta doesn't have a single shared serial bus, the octopus doesn't even use UART!
    //
    // According to my own measurements this function takes 200-300ms.
    // But I don't think it would be safe to offload to another task within the runtime.
    for i in 0..N {
        if (channels >> i) & 0b1 == 1 {
            sgresult2[i] = board.get_sg_result_halved(i as u8).await;
        }
    }

    defmt::debug!("SG_RESULT/2 = {}", sgresult2);
}

#[cfg(feature = "stallguard")]
fn bulk_endstop_check<B, Q, const N: usize>(
    board: &mut B,
    seqs: &mut [Option<Q>; N],
    state: &mut RunState<N>,
) -> u16
where
    B: StepStickBoard,
    Q: SensingWindowDressingSequencer,
{
    let mut flagged = 0u16;
    let stops = STOPS.swap(0, Ordering::AcqRel);

    for i in 0..DRIVERS {
        if (stops >> i) & 0b1 == 1 {
            let seq = if let Some(ref mut seq) = seqs[i] {
                seq
            } else {
                continue;
            };

            debug!(
                "Endstop trigger received for channel {} at {:?}",
                i,
                seq.get_current_state()
            );
            seq.trig_endstop();
            board.clear_steps(i);
            debug!("Channel {} is now at {:?}", i, seq.get_current_state());

            state.next_buf[i] = seq.get_next_instruction();

            flagged |= 1 << i;
        }
    }

    flagged
}

fn bulk_push_pull_state<const N: usize, B, Q>(
    board: &mut B,
    seqs: &mut [Option<Q>; N],
    state: &mut RunState<N>,
) -> u16
where
    B: StepStickBoard,
    Q: WindowDressingSequencer,
{
    let mut stopped = 0u16;

    let now = Instant::now();
    for i in 0..DRIVERS {
        let seq = if let Some(ref mut seq) = seqs[i] {
            seq
        } else {
            continue;
        };

        if !board.get_ready_for_steps(i) {
            continue;
        }

        if let Some(instr) = mem::replace(&mut state.next_buf[i], None) {
            if !board.get_enabled(i) {
                cfg_select! {
                    // Thinking of buying this: https://www.digikey.com.au/en/products/detail/tecate-group/SCAP-PBLS-1-0-27/9929729
                    feature = "brownout-protection" => {
                        if state.brownout_protection + BROWNOUT_PROTECTION <= now {
                            board.set_enabled(i, true);
                            state.brownout_protection = now;
                        } else {
                            // If we're at risk of brownout, undo popping the instruction and move on
                            //
                            // From my experience, 3x1.65A steppers starting up are enough to brown a laptop
                            // charger enough that the last stepper to start up will stall with StallGuard.
                            let _ = mem::replace(&mut state.next_buf[i], Some(instr));
                            continue;
                        }
                    },
                    _ => {
                        board.set_enabled(i, true);
                    }
                }
            }

            if instr.quality == state.cur_direction[i] {
                board.add_steps(i, instr.quantity);
            } else if board.get_stopped(i) && state.next_resume[i] < now {
                state.cur_direction[i] = instr.quality;

                stopped |= 1 << i;

                match instr.quality {
                    Direction::Hold => {
                        let offset = Duration::from_micros(
                            (instr.quantity as u64 * 1_000_000) / FREQUENCY as u64,
                        );
                        state.next_resume[i] = now + offset;

                        // Stop further commands on the PIO SMs & move on to the next channel
                        // Also stops the instruction being placed back into the buffer (as this block handles it)
                        continue;
                    }
                    Direction::Retract => {
                        board.set_direction(i, (REVERSALS.load(Ordering::Acquire) >> i) & 0b1 == 1)
                    }
                    Direction::Extend => {
                        board.set_direction(i, (REVERSALS.load(Ordering::Acquire) >> i) & 0b1 == 0)
                    }
                }
                board.add_steps(i, instr.quantity);
            } else {
                let _ = mem::replace(&mut state.next_buf[i], Some(instr));
            }
        } else if let Some(next) = seq.get_next_instruction_grouped(FREQUENCY as u32) {
            state.next_buf[i] = Some(next);
        } else if board.get_stopped(i) {
            board.set_enabled(i, false);
        }
    }

    stopped
}

#[cfg(any(feature = "host-uart", feature = "host-usb"))]
async fn bulk_emit_state<B, Q, const N: usize>(
    board: &mut B,
    seqs: &[Option<Q>; N],
    channels: u16,
    notify: bool,
) where
    B: ControllableBoard,
    Q: WindowDressingSequencer,
{
    let mut packets = Vec::<_, N>::new();

    for i in 0..DRIVERS {
        if (channels >> i) & 0b1 == 1 {
            let seq = if let Some(ref seq) = seqs[i] {
                seq
            } else {
                continue;
            };

            let _ = packets.push(OutgoingRpcPacket::Position {
                channel: i as u8,
                notify,
                current: *seq.get_current_state(),
                desired: *seq.get_desired_state(),
            });
        }
    }

    if let Err(e) = board.get_host_rpc().write_bulk(packets.iter()).await {
        error!("Failed to bulk write packet: {}", e);

        let _ = AsyncRpc::write_bulk(board.get_host_rpc(), packets.iter())
            .await
            .map_err(|e| error!("Failed to individually bulk write packet: {}", e));
    }
}

#[cfg(any(feature = "host-uart", feature = "host-usb"))]
async fn emit_absence<B>(board: &mut B, channel: u8)
where
    B: ControllableBoard,
{
    let _ = board
        .get_host_rpc()
        .write(&OutgoingRpcPacket::Absent { channel })
        .await;
}
