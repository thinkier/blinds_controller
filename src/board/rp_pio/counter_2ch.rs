use embassy_rp::pio::{Common, Instance, LoadedProgram};

/// This program is intended to run on a 250kHz clock, and is capable of controlling 2 pins.
/// So the divider value should be 500 on the 125MHz default clock
pub struct Counter2ChProgram<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> Counter2ChProgram<'a, PIO> {
    // pub fn new(common: &mut Common<'a, PIO>) -> Self {
    //     let prg = pio_proc::pio_asm!(
    //     );
    //
    //     let prg = common.load_program(&prg.program);
    //
    //     Self { prg }
    // }
}
