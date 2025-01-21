use embassy_rp::pio::{Common, Instance, LoadedProgram};

/// This program is intended to run on a 20:1 ratio i.e. 20 PIO cycles per output cycle
///
/// $$
///     \text{Output Frequency} = \frac{
///         \text{System Frequency}
///     }{
///         \text{Divider} \times 20
///     }
/// $$
///
/// ## Sample PIO clock divider values for the default 125MHz clock
/// | Divider | Output Frequency |
/// |---------|------------------|
/// | 12_500  | 500 Hz           |
/// | 6250    | 1 kHz            |
/// | 3125    | 2 kHz            |
pub struct CountedSqrWavProgram<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> CountedSqrWavProgram<'a, PIO> {
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        let prg = pio_proc::pio_asm!(
            ".side_set 1"
            ".wrap_target"
            "reset:"
                "pull block side 0" // Pull an updated counter from TX FIFO into osr
                "mov x, osr side 0" // Move osr value into x register
                "jmp enter side 0" // Jump into the main loop without the extra waiting introduced to sync up the reset

            "next:"
                "jmp enter [1] side 0"
            "enter:" // Reset + Enter is 11 cycles whereas Next + Enter is 10 cycles
                "jmp x-- hi [7] side 0" // Entry to the loop, decrement x, jump to hi if we're to continue outputting square waves

            "lo:" // Normatively 10 cycles
                "jmp next [9] side 0" // Write lo to the pin for the rest of the phase and jump back to the loop
            "hi:" // Normatively 10 cycles, resetting is 9 cycles
                "jmp !x reset [8] side 1" // Write hi to the pin, if x is zero, jump to reset
                "jmp next side 1" // Continue the loop
            ".wrap"
        );

        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}
