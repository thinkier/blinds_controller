use crate::{
    Direction, RampedInstruction, Ramping, RampingInstruction, WindowDressingInstruction,
    WindowDressingSequencer, WindowDressingState,
};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Emitter(Direction, u32);

#[derive(Clone, Debug, PartialEq)]
struct Emitters<const N: usize>([Emitter; N], usize);

impl WindowDressingInstruction for Emitter {
    fn get_direction(&self) -> &Direction {
        &self.0
    }

    fn get_quantity(&self) -> &u32 {
        &self.1
    }
}

impl WindowDressingSequencer for Emitter {
    type Instruction = Self;

    fn get_next_instruction(&mut self) -> Option<Self::Instruction> {
        Some(self.clone())
    }

    fn get_next_instruction_grouped(&mut self, _threshold: u32) -> Option<Self::Instruction> {
        self.get_next_instruction()
    }

    fn get_current_state(&self) -> &WindowDressingState {
        unimplemented!()
    }

    fn get_desired_state(&self) -> &WindowDressingState {
        unimplemented!()
    }

    fn load_state(&mut self, _state: &WindowDressingState) {
        unimplemented!()
    }

    fn set_state(&mut self, _state: &WindowDressingState) {
        unimplemented!()
    }

    fn set_position(&mut self, _position: u8) {
        unimplemented!()
    }

    fn set_tilt(&mut self, _tilt: i8) {
        unimplemented!()
    }
}
impl<const N: usize> WindowDressingSequencer for Emitters<N> {
    type Instruction = Emitter;

    fn get_next_instruction(&mut self) -> Option<Self::Instruction> {
        if self.1 >= N {
            return None;
        }

        let buf = self.0[self.1];
        self.1 += 1;

        return Some(buf);
    }

    fn get_next_instruction_grouped(&mut self, _threshold: u32) -> Option<Self::Instruction> {
        self.get_next_instruction()
    }

    fn get_current_state(&self) -> &WindowDressingState {
        unimplemented!()
    }

    fn get_desired_state(&self) -> &WindowDressingState {
        unimplemented!()
    }

    fn load_state(&mut self, _state: &WindowDressingState) {
        unimplemented!()
    }

    fn set_state(&mut self, _state: &WindowDressingState) {
        unimplemented!()
    }

    fn set_position(&mut self, _position: u8) {
        unimplemented!()
    }

    fn set_tilt(&mut self, _tilt: i8) {
        unimplemented!()
    }
}

#[test]
fn no_hold_ramp() {
    let mut ramper = Ramping::new(Emitter(Direction::Hold, 3), 1, 2);

    match ramper.get_next_instruction() {
        Some(RampingInstruction::Ordinary(_)) => {}
        value => panic!("expected ordinary instruction, got {:?}", value),
    }
}

#[test]
fn no_hold_ramp_next() {
    let mut ramper = Ramping::new(
        Emitters(
            [Emitter(Direction::Extend, 3), Emitter(Direction::Hold, 3)],
            0,
        ),
        1,
        2,
    );

    let _ = ramper.get_next_instruction();

    match ramper.get_next_instruction() {
        Some(RampingInstruction::Ordinary(_)) => {}
        value => panic!("expected ordinary instruction, got {:?}", value),
    }
}

#[test]
fn low_value_one_step_incomplete() {
    let mut ramper = Ramping::new(Emitter(Direction::Extend, 3), 1, 2);

    let ramped =
        if let RampingInstruction::Ramped { ramped, .. } = ramper.get_next_instruction().unwrap() {
            ramped
        } else {
            panic!("not ramping instruction!")
        };

    assert_eq!(
        RampedInstruction {
            quantity: 3,
            ramping_denominator_exponent: 1,
        },
        ramped[0]
    );

    assert_eq!(1, ramped.len());
}

#[test]
fn low_value_two_step() {
    let mut ramper = Ramping::new(Emitter(Direction::Extend, 5), 1, 2);

    let ramped =
        if let RampingInstruction::Ramped { ramped, .. } = ramper.get_next_instruction().unwrap() {
            ramped
        } else {
            panic!("not ramping instruction!")
        };

    assert_eq!(
        RampedInstruction {
            quantity: 4,
            ramping_denominator_exponent: 1,
        },
        ramped[0]
    );
    assert_eq!(
        RampedInstruction {
            quantity: 1,
            ramping_denominator_exponent: 0,
        },
        ramped[1]
    );

    assert_eq!(2, ramped.len());
}

#[test]
fn high_value_two_step() {
    let mut ramper = Ramping::new(Emitter(Direction::Extend, 1_000_000), 1, 2);

    let ramped =
        if let RampingInstruction::Ramped { ramped, .. } = ramper.get_next_instruction().unwrap() {
            ramped
        } else {
            panic!("not ramping instruction!")
        };

    assert_eq!(
        RampedInstruction {
            quantity: 1 << 2,
            ramping_denominator_exponent: 1,
        },
        ramped[0]
    );
    assert_eq!(
        RampedInstruction {
            quantity: 1_000_000 - (1 << 2),
            ramping_denominator_exponent: 0,
        },
        ramped[1]
    );

    assert_eq!(2, ramped.len());
}

#[test]
fn low_value_three_step_incomplete() {
    let mut ramper = Ramping::new(Emitter(Direction::Extend, 5), 2, 2);

    let ramped =
        if let RampingInstruction::Ramped { ramped, .. } = ramper.get_next_instruction().unwrap() {
            ramped
        } else {
            panic!("not ramping instruction!")
        };

    assert_eq!(
        RampedInstruction {
            quantity: 4,
            ramping_denominator_exponent: 2,
        },
        ramped[0]
    );
    // 5 available at start, 5 used for ramping.
    assert_eq!(
        RampedInstruction {
            quantity: 1,
            ramping_denominator_exponent: 1,
        },
        ramped[1]
    );

    assert_eq!(2, ramped.len());

    // Subsequent instruction not interfered with
    assert_eq!(
        Some(RampingInstruction::Ordinary(Emitter(Direction::Extend, 5))),
        ramper.get_next_instruction()
    );
}

#[test]
fn high_value_three_step() {
    let mut ramper = Ramping::new(Emitter(Direction::Extend, 1_000_000), 2, 2);

    let ramped =
        if let RampingInstruction::Ramped { ramped, .. } = ramper.get_next_instruction().unwrap() {
            ramped
        } else {
            panic!("not ramping instruction!")
        };

    assert_eq!(
        RampedInstruction {
            quantity: 1 << 2,
            ramping_denominator_exponent: 2,
        },
        ramped[0]
    );
    assert_eq!(
        RampedInstruction {
            quantity: 1 << 1,
            ramping_denominator_exponent: 1,
        },
        ramped[1]
    );
    assert_eq!(
        RampedInstruction {
            quantity: 1_000_000 - 0b110,
            ramping_denominator_exponent: 0,
        },
        ramped[2]
    );

    assert_eq!(3, ramped.len());
}
