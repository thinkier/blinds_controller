use crate::model::sequencer::{WindowDressingInstruction, WindowDressingState};
use crate::{Direction, SensingWindowDressingSequencer, WindowDressingSequencer};
type HaltingSequencer = crate::model::sequencer::HaltingSequencer<1024>;

#[test]
fn desired_state_updates() {
    let mut seq = HaltingSequencer::new_venetian(100_000, 180_0);
    seq.set_tilt(69);
    assert_eq!(seq.desired_state.tilt, 69);
}

#[test]
fn current_state_updates() {
    let mut seq = HaltingSequencer::new_venetian(100_000, 180_0);
    seq.current_state.tilt = 0;
    seq.set_tilt(69);
    for i in 1..=69 {
        seq.get_next_instruction();
        assert_eq!(seq.current_state.tilt, i);
    }
}

#[test]
fn noop_on_same_tilt() {
    let mut seq = HaltingSequencer::new_venetian(100_000, 180_0);
    seq.current_state.tilt = 69;
    seq.set_tilt(69);
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn close_full() {
    let mut seq = HaltingSequencer::new_venetian(100_000, 180_0);
    seq.current_state.tilt = -90;
    seq.set_tilt(90);
    for i in -89..=90 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Extend,
                quantity: 10,
                completed_state: WindowDressingState {
                    position: 0,
                    tilt: i
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
                tilt: 90
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn open_full() {
    let mut seq = HaltingSequencer::new_venetian(100_000, 180_0);
    seq.current_state.tilt = 90;
    seq.set_tilt(-90);
    for i in -89..=90 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Retract,
                quantity: 10,
                completed_state: WindowDressingState {
                    position: 0,
                    tilt: -i
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
                tilt: -90
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn close_trig_endstop() {
    let mut seq = HaltingSequencer::new_venetian(100_000, 180_0);
    seq.current_state.tilt = -90;
    seq.set_tilt(90);

    let _ = (0..90).map(|_| seq.get_next_instruction());
    seq.trig_endstop();
    assert_eq!(
        seq.current_state,
        WindowDressingState {
            position: 0,
            tilt: 90,
        }
    );
    assert_eq!(
        seq.get_next_instruction(),
        Some(WindowDressingInstruction {
            quality: Direction::Hold,
            quantity: 500,
            completed_state: WindowDressingState {
                position: 0,
                tilt: 90
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn open_trig_endstop() {
    let mut seq = HaltingSequencer::new_venetian(100_000, 180_0);
    seq.current_state.tilt = 90;
    seq.set_tilt(-90);

    let _ = (0..90).map(|_| seq.get_next_instruction());
    seq.trig_endstop();
    assert_eq!(
        seq.current_state,
        WindowDressingState {
            position: 100,
            tilt: 90,
        }
    );
    assert_eq!(
        seq.get_next_instruction(),
        Some(WindowDressingInstruction {
            quality: Direction::Hold,
            quantity: 500,
            completed_state: WindowDressingState {
                position: 100,
                tilt: 90
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}
