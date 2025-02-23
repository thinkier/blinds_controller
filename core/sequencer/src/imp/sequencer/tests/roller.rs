use crate::model::sequencer::{WindowDressingInstruction, WindowDressingState};
use crate::{Direction, SensingWindowDressingSequencer, WindowDressingSequencer};
type HaltingSequencer = crate::model::sequencer::HaltingSequencer<1024>;

#[test]
fn desired_state_updates() {
    let mut seq = HaltingSequencer::new_roller(100_000);
    seq.set_position(69);
    assert_eq!(
        seq.desired_state,
        WindowDressingState {
            position: 69,
            tilt: 0
        }
    );
}

#[test]
fn current_state_updates() {
    let mut seq = HaltingSequencer::new_roller(100_000);
    seq.current_state.position = 0;
    seq.set_position(69);
    for i in 1..=69 {
        seq.get_next_instruction();
        assert_eq!(
            seq.current_state,
            WindowDressingState {
                position: i,
                tilt: 0
            }
        );
    }
}

#[test]
fn noop_on_same_position() {
    let mut seq = HaltingSequencer::new_roller(100_000);
    seq.current_state.position = 69;
    seq.set_position(69);
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn open_fully() {
    let mut seq = HaltingSequencer::new_roller(100_000);
    seq.current_state.position = 0;
    seq.set_position(100);

    for i in 1..=100 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Retract,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: i,
                    tilt: 0
                },
            })
        );
    }
    assert_eq!(
        seq.get_next_instruction(),
        Some(WindowDressingInstruction {
            quality: Direction::Hold,
            quantity: 500,
            completed_state: WindowDressingState {
                position: 100,
                tilt: 0
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn open_partially() {
    let mut seq = HaltingSequencer::new_roller(100_000);
    seq.current_state.position = 25;
    seq.set_position(75);

    for i in 1..=50 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Retract,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: i + 25,
                    tilt: 0
                },
            })
        );
    }
    assert_eq!(
        seq.get_next_instruction(),
        Some(WindowDressingInstruction {
            quality: Direction::Hold,
            quantity: 500,
            completed_state: WindowDressingState {
                position: 75,
                tilt: 0
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn close_fully() {
    let mut seq = HaltingSequencer::new_roller(100_000);
    seq.current_state.position = 100;
    seq.set_position(0);

    for j in 1..=100 {
        let i = 100 - j;
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Extend,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: i,
                    tilt: 0
                },
            })
        );
    }
    assert_eq!(
        seq.get_next_instruction(),
        Some(WindowDressingInstruction {
            quality: Direction::Hold,
            quantity: 500,
            completed_state: WindowDressingState {
                position: 0,
                tilt: 0
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn close_partially() {
    let mut seq = HaltingSequencer::new_roller(100_000);
    seq.current_state.position = 75;
    seq.set_position(25);

    for j in 1..=50 {
        let i = 75 - j;
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Extend,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: i,
                    tilt: 0
                },
            })
        );
    }
    assert_eq!(
        seq.get_next_instruction(),
        Some(WindowDressingInstruction {
            quality: Direction::Hold,
            quantity: 500,
            completed_state: WindowDressingState {
                position: 25,
                tilt: 0
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn open_trig_endstop() {
    let mut seq = HaltingSequencer::new_roller(100_000);
    seq.current_state.position = 0;
    seq.set_position(100);

    for i in 1..=50 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Retract,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: i,
                    tilt: 0
                },
            })
        );
    }

    seq.trig_endstop();
    assert_eq!(
        seq.get_next_instruction(),
        Some(WindowDressingInstruction {
            quality: Direction::Hold,
            quantity: 500,
            completed_state: WindowDressingState {
                position: 100,
                tilt: 0
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn close_trig_endstop() {
    let mut seq = HaltingSequencer::new_roller(100_000);
    seq.current_state.position = 100;
    seq.set_position(0);

    for i in 1..=50 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Extend,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: 100 - i,
                    tilt: 0
                },
            })
        );
    }

    seq.trig_endstop();
    assert_eq!(
        seq.get_next_instruction(),
        Some(WindowDressingInstruction {
            quality: Direction::Hold,
            quantity: 500,
            completed_state: WindowDressingState {
                position: 0,
                tilt: 0
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}
