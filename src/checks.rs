use crate::board::SerialBuffers;
use crate::comms::InstructionBuffer;
use blinds_sequencer::{HaltingSequencer, WindowDressingInstruction};
use core::mem;
use embassy_executor::Executor;
use embassy_rp::multicore::Stack;
use embassy_rp::Peripherals;
use portable_atomic::AtomicU8;
use static_cell::StaticCell;

pub fn all_checks() {
    memory_check();
}

fn memory_check() {
    let mut sum_bytes = 0;

    sum_bytes += mem::size_of::<StaticCell<Executor>>();
    sum_bytes += mem::size_of::<Stack<8192>>();
    sum_bytes += mem::size_of::<AtomicU8>();
    sum_bytes += mem::size_of::<StaticCell<Peripherals>>();
    sum_bytes += mem::size_of::<InstructionBuffer<WindowDressingInstruction, { crate::DRIVERS }>>();
    sum_bytes += mem::size_of::<SerialBuffers>();
    sum_bytes += mem::size_of::<StaticCell<[HaltingSequencer<1024>; { crate::DRIVERS }]>>();

    defmt::info!(
        "Using {} bytes of SRAM in static variables (including core1 stack)",
        sum_bytes
    );
}
