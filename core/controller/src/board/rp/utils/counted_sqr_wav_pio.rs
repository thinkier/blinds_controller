use crate::FREQUENCY;
use defmt::debug;
use embassy_rp::clocks::clk_sys_freq;
use embassy_rp::gpio::Level;
use embassy_rp::pio::{
    Common, Config, Direction, FifoJoin, Instance, LoadedProgram, PioPin, StateMachine,
};
use embassy_rp::Peri;
use fixed::traits::ToFixed;
use pio::pio_file;

/// This program is intended to run on a 24:1 ratio i.e. 24 PIO cycles per output cycle
///
/// $$
///     \text{Output Frequency} = \frac{
///         \text{System Frequency}
///     }{
///         \text{Divider} \times 24
///     }
/// $$
pub struct CountedSqrWavProgram<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> CountedSqrWavProgram<'a, PIO> {
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        // Code was evaluated on https://ice458.github.io/tools/pio_sim/index.html
        let prg = pio_file!("src/board/rp/utils/counted_sqr_wav.pio");

        let prg = common.load_program(&prg.program);

        debug!("System frequency is {}MHz", clk_sys_freq() / 1_000_000);

        Self { prg }
    }
}

pub struct CountedSqrWav<'a, PIO: Instance, const SM: usize> {
    sm: &'a mut StateMachine<'a, PIO, SM>,
}

impl<'a, PIO: Instance, const SM: usize> CountedSqrWav<'a, PIO, SM> {
    pub fn new(
        pio: &mut Common<'a, PIO>,
        sm: &'a mut StateMachine<'a, PIO, SM>,
        pin: Peri<'a, impl PioPin + 'a>,
        program: &'a CountedSqrWavProgram<'a, PIO>,
    ) -> Self {
        let pin = pio.make_pio_pin(pin);
        sm.set_pins(Level::Low, &[&pin]);
        sm.set_pin_dirs(Direction::Out, &[&pin]);

        let mut cfg = Config::default();

        cfg.fifo_join = FifoJoin::TxOnly;

        cfg.set_set_pins(&[&pin]);
        cfg.use_program(&program.prg, &[]);

        cfg.clock_divider = (clk_sys_freq() / (FREQUENCY as u32 * 24)).to_fixed();

        sm.set_config(&cfg);

        Self { sm }
    }

    pub fn clear(&mut self) {
        self.sm.set_enable(false);
        self.sm.clear_fifos();
        self.sm.restart();
    }

    pub fn stopped(&mut self) -> bool {
        self.sm.tx().stalled() || !self.sm.is_enabled()
    }

    pub fn ready(&mut self) -> bool {
        self.sm.tx().empty()
    }

    /// `delay_cycles` is the number of PIO cycles to stall the state machine in each phase.
    ///
    /// $$
    /// f = \frac{24}{24 + delay_cycles} \times 1000
    /// $$
    ///
    /// Example values:
    /// | Frequency | `delay_cycles` |
    /// |-----------|----------------|
    /// |  1000 Hz  | 0              |
    /// |   800 Hz  | 6              |
    /// |   750 Hz  | 8              |
    /// |   667 Hz  | 12             |
    /// |   600 Hz  | 16             |
    /// |   500 Hz  | 24             |
    /// |   400 Hz  | 36             |
    /// |   333 Hz  | 48             |
    /// |   250 Hz  | 72             |
    /// |   200 Hz  | 96             |
    /// |   125 Hz  | 168            |
    /// |   100 Hz  | 216            |
    /// |  62.5 Hz  | 360            |
    /// |    50 Hz  | 456            |
    /// |    25 Hz  | 936            |
    /// | 0.366 Hz  | 65535          |
    pub fn try_push(&mut self, value: u32) -> bool {
        self.sm.set_enable(true);
        self.sm.tx().try_push(value)
    }
}
