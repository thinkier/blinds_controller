use crate::model::sequencer::{WindowDressingInstruction, WindowDressingState};
use crate::{Direction, WindowDressingSequencer};
type HaltingSequencer = crate::model::sequencer::HaltingSequencer<1024>;

#[test]
fn close_rams() {
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

    seq.set_position(100);
    assert_eq!(
        seq.get_next_instruction(),
        Some(WindowDressingInstruction {
            quality: Direction::Hold,
            quantity: 500,
            completed_state: WindowDressingState {
                position: 50,
                tilt: 0
            },
        })
    );

    for i in 1..=50 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Retract,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: 50 + i,
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
fn open_rams() {
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

    seq.set_position(0);
    assert_eq!(
        seq.get_next_instruction(),
        Some(WindowDressingInstruction {
            quality: Direction::Hold,
            quantity: 500,
            completed_state: WindowDressingState {
                position: 50,
                tilt: 0
            },
        })
    );

    for i in 1..=50 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Extend,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: 50 - i,
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
