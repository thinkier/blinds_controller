use embassy_rp::pio::{Common, Instance, LoadedProgram};

/// This program is intended to run on a 20kHz clock, i.e. 50 instructions per cycle
/// So the divider value should be 6250 on the 125MHz default clock
pub struct CountedSqrWavProgram<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> CountedSqrWavProgram<'a, PIO> {
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        let prg = pio_proc::pio_asm!(
            ".side_set 1"
            ".wrap_target"
                "pull noblock side 0"
                // "add x, isr"
                "jmp x-- continue [9] side 0"
                "jmp end [7] side 0"
            "continue:"
                "set pins, 1 [5] side 1"
            "end:"
            ".wrap"
        );

        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}
