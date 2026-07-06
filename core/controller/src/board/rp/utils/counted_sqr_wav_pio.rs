use embassy_rp::clocks::clk_sys_freq;
use embassy_rp::gpio::Level;
use embassy_rp::pio::{Common, Config, Direction, Instance, LoadedProgram, PioPin, StateMachine};
use embassy_rp::Peri;
use fixed::traits::ToFixed;

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
        let prg = pio::pio_asm!(
        ".wrap_target"

            "reset:"
                "pull block"
                "set pins 1 [8]" // Fixed bug where it outputs 1 less step than provided
                "out x, 16" // 16 MSB for delays, 16 LSB for steps, zero steps is invalid
                "jmp enter" // Reset + enter = 13 as reset acts as a pull-high now

            "pull_high:" // Totals 12
                "set pins 1"
                "nop [9]"
            "enter:"
                "jmp x-- pull_low"

            "pull_low:" // Totals 12 in all code paths except when jumping to reset, then it's 11
                "set pins 0 [7]"
                // Dead time insertion
                "mov y, osr"
                "stall:"
                    "jmp y-- stall"

                // do ... x-- ... while x > 0
                "jmp !x reset"
                "jmp pull_high"
        ".wrap"
        );

        let prg = common.load_program(&prg.program);

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
        frequency: u16,
    ) -> Self {
        let pin = pio.make_pio_pin(pin);
        sm.set_pins(Level::Low, &[&pin]);
        sm.set_pin_dirs(Direction::Out, &[&pin]);

        let mut cfg = Config::default();
        cfg.use_program(&program.prg, &[&pin]);
        cfg.clock_divider = (clk_sys_freq() / (frequency as u32 * 24)).to_fixed();

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

    pub fn try_push(&mut self, count: u16) -> bool {
        self.sm.set_enable(true);
        self.sm.tx().try_push(count as u32)
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
    /// | 1000 Hz   | 0              |
    /// |  800 Hz   | 6              |
    /// |  750 Hz   | 8              |
    /// |  667 Hz   | 12             |
    /// |  600 Hz   | 16             |
    /// |  500 Hz   | 24             |
    /// |  400 Hz   | 36             |
    /// |  333 Hz   | 48             |
    /// |  250 Hz   | 72             |
    /// |  200 Hz   | 96             |
    /// |  125 Hz   | 168            |
    /// |  100 Hz   | 216            |
    /// |   50 Hz   | 456            |
    pub fn try_push_modulated(&mut self, count: u16, delay_cycles: u16) -> bool {
        self.sm.set_enable(true);
        self.sm
            .tx()
            .try_push(((delay_cycles as u32) << 16) | (count as u32))
    }
}
