use crate::{
    Direction, Ramping, RampingInstruction, WindowDressingInstruction, WindowDressingSequencer,
    WindowDressingState,
};

#[derive(Clone)]
struct Emitter(u32);

impl WindowDressingInstruction for Emitter {
    fn get_direction(&self) -> &Direction {
        &Direction::Extend
    }

    fn get_quantity(&self) -> &u32 {
        &self.0
    }

    fn get_quantity_mut(&mut self) -> &mut u32 {
        &mut self.0
    }
}

impl WindowDressingSequencer for Emitter {
    type Instruction = Self;

    fn get_next_instruction(&mut self) -> Option<Self::Instruction> {
        Some(self.clone())
    }

    fn get_next_instruction_grouped(&mut self, _threshold: u32) -> Option<Self::Instruction> {
        Some(self.clone())
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
fn low_value_one_step_incomplete() {
    let mut ramper = Ramping::new(Emitter(3), 1, 2);

    let coarse = ramper.get_next_instruction();

    assert_eq!(
        RampingInstruction {
            direction: Direction::Extend,
            quantity: 3,
            ramping_denominator_exponent: 1,
        },
        coarse.unwrap()
    );
}

#[test]
fn low_value_two_step() {
    let mut ramper = Ramping::new(Emitter(5), 1, 2);

    let coarse = ramper.get_next_instruction();
    let normal = ramper.get_next_instruction();

    assert_eq!(
        RampingInstruction {
            direction: Direction::Extend,
            quantity: 4,
            ramping_denominator_exponent: 1,
        },
        coarse.unwrap()
    );
    assert_eq!(
        RampingInstruction {
            direction: Direction::Extend,
            quantity: 1,
            ramping_denominator_exponent: 0,
        },
        normal.unwrap()
    );
}

#[test]
fn high_value_two_step() {
    let mut ramper = Ramping::new(Emitter(1_000_000), 1, 2);

    let coarse = ramper.get_next_instruction();
    let normal = ramper.get_next_instruction();

    assert_eq!(
        RampingInstruction {
            direction: Direction::Extend,
            quantity: 1 << 2,
            ramping_denominator_exponent: 1,
        },
        coarse.unwrap()
    );
    assert_eq!(
        RampingInstruction {
            direction: Direction::Extend,
            quantity: 1_000_000 - (1 << 2),
            ramping_denominator_exponent: 0,
        },
        normal.unwrap()
    );
}

#[test]
fn low_value_three_step_incomplete() {
    let mut ramper = Ramping::new(Emitter(5), 2, 2);

    let coarse = ramper.get_next_instruction();
    let fine = ramper.get_next_instruction();
    let normal = ramper.get_next_instruction();

    assert_eq!(
        RampingInstruction {
            direction: Direction::Extend,
            quantity: 4,
            ramping_denominator_exponent: 2,
        },
        coarse.unwrap()
    );
    // 5 available at start, 5 used for ramping.
    assert_eq!(
        RampingInstruction {
            direction: Direction::Extend,
            quantity: 1,
            ramping_denominator_exponent: 1,
        },
        fine.unwrap()
    );
    // Subsequent instruction not interfered with
    assert_eq!(
        RampingInstruction {
            direction: Direction::Extend,
            quantity: 5,
            ramping_denominator_exponent: 0,
        },
        normal.unwrap()
    );
}

#[test]
fn high_value_three_step() {
    let mut ramper = Ramping::new(Emitter(1_000_000), 2, 2);

    let coarse = ramper.get_next_instruction();
    let fine = ramper.get_next_instruction();
    let normal = ramper.get_next_instruction();

    assert_eq!(
        RampingInstruction {
            direction: Direction::Extend,
            quantity: 1 << 2,
            ramping_denominator_exponent: 2,
        },
        coarse.unwrap()
    );
    assert_eq!(
        RampingInstruction {
            direction: Direction::Extend,
            quantity: 1 << 1,
            ramping_denominator_exponent: 1,
        },
        fine.unwrap()
    );
    assert_eq!(
        RampingInstruction {
            direction: Direction::Extend,
            quantity: 1_000_000 - 0b110,
            ramping_denominator_exponent: 0,
        },
        normal.unwrap()
    );
}
