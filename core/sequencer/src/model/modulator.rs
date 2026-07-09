use heapless::Vec;
use crate::Direction;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Ramping<T> {
    pub(crate) inner: T,
    pub(crate) last_direction: Direction,
    pub(crate) ramp_exponent: u16,
    pub(crate) ramp_steps_exponent: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RampingInstruction<T> {
    Ordinary(T),
    Ramped {
        inner: T,
        ramped: Vec<RampedInstruction, 8>, // pico simplex fifo depth is 8
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct RampedInstruction {
    pub quantity: u32,
    pub ramping_denominator_exponent: u16
}