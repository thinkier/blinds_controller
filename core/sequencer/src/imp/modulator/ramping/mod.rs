use crate::{
    Direction, RampedInstruction, Ramping, RampingInstruction, SensingWindowDressingSequencer,
    WindowDressingInstruction, WindowDressingSequencer, WindowDressingState,
};
use core::ops::Deref;
use heapless::Vec;

#[cfg(test)]
mod tests;

impl<T: WindowDressingSequencer> Ramping<T> {
    pub fn new(inner: T, ramp_exponent: u16, ramp_steps_exponent: u16) -> Self {
        assert!(ramp_exponent < 7);  // pico simplex fifo depth is 8, minus 1 for tail
        assert!(ramp_steps_exponent >= ramp_exponent);

        Self {
            inner,
            last_direction: Direction::Hold,
            ramp_exponent,
            ramp_steps_exponent,
        }
    }
}

impl<T: SensingWindowDressingSequencer> SensingWindowDressingSequencer for Ramping<T> {
    fn trig_endstop(&mut self) {
        self.inner.trig_endstop()
    }

    fn home_fully_opened(&mut self) {
        self.inner.home_fully_opened()
    }

    fn home_fully_closed(&mut self) {
        self.inner.home_fully_closed()
    }
}

impl<T: WindowDressingSequencer> WindowDressingSequencer for Ramping<T> {
    type Instruction = RampingInstruction<T::Instruction>;

    fn get_next_instruction(&mut self) -> Option<Self::Instruction> {
        self.get_next_instruction_grouped(0)
    }

    fn get_next_instruction_grouped(&mut self, threshold: u32) -> Option<Self::Instruction> {
        let take = (2 << self.ramp_steps_exponent) - 1;
        let inner = self.inner.get_next_instruction_grouped(threshold + take)?;
        let mut quantity = *inner.get_quantity();

        if self.last_direction == *inner.get_direction() {
            return Some(RampingInstruction::Ordinary(inner));
        } else {
            self.last_direction = *inner.get_direction();

            if self.last_direction == Direction::Hold {
                return Some(RampingInstruction::Ordinary(inner));
            }
        }

        let mut ramped = Vec::new();

        for i in 0..self.ramp_exponent {
            let d = self.ramp_exponent - i;

            let mut inter_quantity = 1 << (self.ramp_steps_exponent - i);

            if inter_quantity > quantity {
                inter_quantity = quantity;
            }

            quantity -= inter_quantity;

            let _ = ramped.push(RampedInstruction {
                quantity: inter_quantity,
                ramping_denominator_exponent: d,
            });

            if quantity == 0 {
                return Some(RampingInstruction::Ramped { inner, ramped });
            }
        }

        let _ = ramped.push(RampedInstruction {
            quantity,
            ramping_denominator_exponent: 0,
        });

        Some(RampingInstruction::Ramped { inner, ramped })
    }

    fn get_current_state(&self) -> &WindowDressingState {
        self.inner.get_current_state()
    }

    fn get_desired_state(&self) -> &WindowDressingState {
        self.inner.get_desired_state()
    }

    fn load_state(&mut self, state: &WindowDressingState) {
        self.inner.load_state(state)
    }

    fn set_state(&mut self, state: &WindowDressingState) {
        self.inner.set_state(state)
    }

    fn set_position(&mut self, position: u8) {
        self.inner.set_position(position)
    }

    fn set_tilt(&mut self, tilt: i8) {
        self.inner.set_tilt(tilt)
    }
}

impl<T> Deref for RampingInstruction<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            RampingInstruction::Ordinary(data) => data,
            RampingInstruction::Ramped { inner, .. } => &inner,
        }
    }
}

impl<T> WindowDressingInstruction for RampingInstruction<T>
where
    T: WindowDressingInstruction,
{
    fn get_direction(&self) -> &Direction {
        self.deref().get_direction()
    }

    fn get_quantity(&self) -> &u32 {
        self.deref().get_quantity()
    }
}
