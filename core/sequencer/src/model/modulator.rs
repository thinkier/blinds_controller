use crate::{Direction, WindowDressingSequencer};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Ramping<T: WindowDressingSequencer> {
    pub(crate) inner: T,
    pub(crate) buffer: Option<T::Instruction>,
    pub(crate) last_direction: Direction,
    pub(crate) last_count: u16,
    pub(crate) ramp_exponent: u16,
    pub(crate) ramp_steps_exponent: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct RampingInstruction {
    pub direction: Direction,
    pub quantity: u32,
    pub ramping_denominator_exponent: u16,
}
