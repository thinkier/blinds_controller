use embassy_rp::pio::{Common, Instance, LoadedProgram};

/// This program is intended to run on a 20kHz clock, i.e. 20 instructions per cycle
///
/// The divider value should be 6250 on the 125MHz default clock if the desired frequency is 1KHz
pub struct CountedSqrWavProgram<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> CountedSqrWavProgram<'a, PIO> {
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        let prg = pio_proc::pio_asm!(
            ".side_set 1"
            ".wrap_target"
            "rst:"
                "pull noblock side 0" // Pull an updated counter from TX FIFO into osr
                "mov x, osr" // Move osr value into x register
                "jmp enter" // Jump into the main loop without the extra waiting introduced to sync up the reset

            "loop:"
                "jmp enter [2] side 0" // Synchronization with the reset code
            "enter:"
                "jmp x-- hi [6] side 0" // Entry to the loop, decrement x, jump to hi if we're to continue outputting square waves

            "lo:"
                "jmp loop [9] side 0" // Write lo to the pin for the rest of the phase and jump back to the loop
            "hi:"
                "jmp !x rst [8] side 1" // Write hi to the pin, if x is zero, jump to reset
                "jmp loop" // Continue the loop
            ".wrap"
        );

        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}
