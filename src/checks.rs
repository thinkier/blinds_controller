use crate::board::SerialBuffers;
use blinds_sequencer::{HaltingSequencer};
use core::mem;
use portable_atomic::AtomicU8;
use static_cell::StaticCell;

pub fn all_checks() {
    memory_check();
}

fn memory_check() {
    let mut sum_bytes = 0;

    sum_bytes += mem::size_of::<AtomicU8>() * 2;
    sum_bytes += mem::size_of::<SerialBuffers>();
    sum_bytes += mem::size_of::<StaticCell<[HaltingSequencer<1024>; crate::DRIVERS]>>();

    defmt::info!(
        "Using {} bytes of SRAM in additional static variables",
        sum_bytes
    );
}
