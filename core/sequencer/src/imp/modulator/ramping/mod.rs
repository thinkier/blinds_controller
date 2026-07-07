use crate::{
    Direction, Ramping, RampingInstruction, SensingWindowDressingSequencer,
    WindowDressingInstruction, WindowDressingSequencer, WindowDressingState,
};
use core::mem;

#[cfg(test)]
mod tests;

impl<T: WindowDressingSequencer> Ramping<T> {
    pub fn new(inner: T, ramp_exponent: u16, ramp_steps_exponent: u16) -> Self {
        Self {
            inner,
            buffer: None,
            last_direction: Direction::Hold,
            last_count: ramp_exponent,
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
    type Instruction = RampingInstruction;

    fn get_next_instruction(&mut self) -> Option<Self::Instruction> {
        self.get_next_instruction_grouped(0)
    }

    fn get_next_instruction_grouped(&mut self, threshold: u32) -> Option<Self::Instruction> {
        let mut buf = mem::replace(&mut self.buffer, None);

        if buf.is_none() {
            buf = self.inner.get_next_instruction_grouped(threshold);
        }

        let mut buf = buf?;

        if self.last_direction != *buf.get_direction() {
            self.last_count = self.ramp_exponent;
            self.last_direction = *buf.get_direction();
        }

        let direction = *buf.get_direction();
        let mut quantity = *buf.get_quantity();
        let mut ramping_denominator_exponent = 0;

        if self.last_count > 0 {
            ramping_denominator_exponent = self.last_count;
            let take_exp = self.ramp_steps_exponent - (self.ramp_exponent - self.last_count);

            if quantity > 1 << take_exp {
                quantity = 1 << take_exp;
                *buf.get_quantity_mut() -= quantity;
                let _ = mem::replace(&mut self.buffer, Some(buf));
            }

            self.last_count -= 1;
        }

        Some(Self::Instruction {
            direction,
            quantity,
            ramping_denominator_exponent,
        })
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

impl WindowDressingInstruction for RampingInstruction {
    fn get_direction(&self) -> &Direction {
        &self.direction
    }

    fn get_quantity(&self) -> &u32 {
        &self.quantity
    }

    fn get_quantity_mut(&mut self) -> &mut u32 {
        &mut self.quantity
    }
}
