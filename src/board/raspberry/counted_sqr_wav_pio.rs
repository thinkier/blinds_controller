use embassy_rp::clocks::clk_sys_freq;
use embassy_rp::gpio::Level;
use embassy_rp::pio::{Common, Config, Direction, Instance, LoadedProgram, PioPin, StateMachine};
use fixed::traits::ToFixed;

/// This program is intended to run on a 10:1 ratio i.e. 10 PIO cycles per output cycle
///
/// $$
///     \text{Output Frequency} = \frac{
///         \text{System Frequency}
///     }{
///         \text{Divider} \times 10
///     }
/// $$
pub struct CountedSqrWavProgram<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> CountedSqrWavProgram<'a, PIO> {
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        let prg = pio_proc::pio_asm!(
            ".side_set 1 opt"
            ".wrap_target"

            "reset:"
                "pull block" // Pull an updated counter from TX FIFO into osr
                "mov x, osr side 0" // Move osr value into x register, also beginning of the falling edge
                "jmp enter" // Jump into the main loop without the extra waiting introduced to sync up the reset

            "pull_low:"
                "jmp enter [1] side 0"
            "enter:" // Reset + Enter is 6 cycles whereas Pull-Low + Enter is 5 cycles
                "jmp x-- pull_high [2]" // Entry to the loop, decrement x. jmp is the only instruction that's capable of updating the counter

            "pull_high:" // Normatively 5 cycles, resetting is 4 cycles
                "jmp !x reset [3] side 1" // Write hi to the pin, if x is zero, jump to reset
                "jmp pull_low" // Continue the loop
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
        pin: &'a mut impl PioPin,
        program: &'a CountedSqrWavProgram<'a, PIO>,
        frequency: u16,
    ) -> Self {
        let pin = pio.make_pio_pin(pin);
        sm.set_pins(Level::Low, &[&pin]);
        sm.set_pin_dirs(Direction::Out, &[&pin]);

        let mut cfg = Config::default();
        cfg.use_program(&program.prg, &[&pin]);
        cfg.clock_divider = (clk_sys_freq() / (frequency as u32 * 10)).to_fixed();

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

    pub fn try_push(&mut self, count: u32) -> bool {
        self.sm.set_enable(true);
        self.sm.tx().try_push(count)
    }
}
