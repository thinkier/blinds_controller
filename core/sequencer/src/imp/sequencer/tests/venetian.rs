use crate::model::sequencer::{WindowDressingInstruction, WindowDressingState};
use crate::{Direction, SensingWindowDressingSequencer, WindowDressingSequencer};
type HaltingSequencer = crate::model::sequencer::HaltingSequencer<1024>;

#[test]
fn open_full_sequence() {
    let mut seq = HaltingSequencer::new_venetian(100_000, 180_0);
    seq.current_state.position = 0;
    seq.current_state.tilt = 90;
    seq.set_position(100);

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

    for i in 1..=100 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Retract,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: i,
                    tilt: -90
                },
            })
        );
    }

    // Blinds should teleport back to previous tilt when fully opened
    assert_eq!(
        seq.get_next_instruction(),
        Some(WindowDressingInstruction {
            quality: Direction::Hold,
            quantity: 0,
            completed_state: WindowDressingState {
                position: 100,
                tilt: 90
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn open_full_tiltless_sequence() {
    let mut seq = HaltingSequencer::new_venetian(100_000, 180_0);
    seq.current_state.position = 0;
    seq.current_state.tilt = -90;
    seq.set_position(100);

    for i in 1..=100 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Retract,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: i,
                    tilt: -90
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
                tilt: -90
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn open_partial_sequence() {
    let mut seq = HaltingSequencer::new_venetian(100_000, 180_0);
    seq.current_state.position = 25;
    seq.current_state.tilt = 60;
    seq.set_position(75);

    for i in -59..=90 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Retract,
                quantity: 10,
                completed_state: WindowDressingState {
                    position: 25,
                    tilt: -i
                },
            })
        );
    }

    for i in 26..=75 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Retract,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: i,
                    tilt: -90
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
                tilt: -90
            },
        })
    );

    for i in -89..=60 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Extend,
                quantity: 10,
                completed_state: WindowDressingState {
                    position: 75,
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
                position: 75,
                tilt: 60
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn open_partial_tiltless_sequence() {
    let mut seq = HaltingSequencer::new_venetian(100_000, 180_0);
    seq.current_state.position = 25;
    seq.current_state.tilt = -90;
    seq.set_position(75);

    for i in 26..=75 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Retract,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: i,
                    tilt: -90
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
                tilt: -90
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn open_trig_endstop() {
    let mut seq = HaltingSequencer::new_venetian(100_000, 180_0);
    seq.current_state.position = 0;
    seq.current_state.tilt = -90;
    seq.set_position(90);

    for i in 1..=80 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Retract,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: i,
                    tilt: -90
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
                tilt: 90
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn close_full_sequence() {
    let mut seq = HaltingSequencer::new_venetian(100_000, 180_0);
    seq.current_state.position = 100;
    seq.current_state.tilt = -90;
    seq.set_position(0);

    // no tilt when position is 100
    assert_eq!(
        seq.get_next_instruction(),
        Some(WindowDressingInstruction {
            quality: Direction::Hold,
            quantity: 0,
            completed_state: WindowDressingState {
                position: 100,
                tilt: 90
            },
        })
    );

    for i in -99..=0 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Extend,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: -i as u8,
                    tilt: 90
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
fn close_full_tiltless_sequence() {
    let mut seq = HaltingSequencer::new_venetian(100_000, 180_0);
    seq.current_state.position = 100;
    seq.current_state.tilt = 90;
    seq.set_position(0);

    for i in -99..=0 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Extend,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: -i as u8,
                    tilt: 90
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
fn close_partial_sequence() {
    let mut seq = HaltingSequencer::new_venetian(100_000, 180_0);
    seq.current_state.position = 75;
    seq.current_state.tilt = -90;
    seq.set_position(25);

    for i in -89..=90 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Extend,
                quantity: 10,
                completed_state: WindowDressingState {
                    position: 75,
                    tilt: i
                },
            })
        );
    }

    for i in -74..=-25 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Extend,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: -i as u8,
                    tilt: 90
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
                tilt: 90
            },
        })
    );

    for i in -89..=90 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Retract,
                quantity: 10,
                completed_state: WindowDressingState {
                    position: 25,
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
                position: 25,
                tilt: -90
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn close_partial_tiltless_sequence() {
    let mut seq = HaltingSequencer::new_venetian(100_000, 180_0);
    seq.current_state.position = 75;
    seq.current_state.tilt = 90;
    seq.set_position(25);

    for i in -74..=-25 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Extend,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: -i as u8,
                    tilt: 90
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
                tilt: 90
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}

#[test]
fn close_trig_endstop() {
    let mut seq = HaltingSequencer::new_venetian(100_000, 180_0);
    seq.current_state.position = 100;
    seq.current_state.tilt = -90;
    seq.set_position(0);

    // no tilt when position is 100
    assert_eq!(
        seq.get_next_instruction(),
        Some(WindowDressingInstruction {
            quality: Direction::Hold,
            quantity: 0,
            completed_state: WindowDressingState {
                position: 100,
                tilt: 90
            },
        })
    );

    for i in -99..=-20 {
        assert_eq!(
            seq.get_next_instruction(),
            Some(WindowDressingInstruction {
                quality: Direction::Extend,
                quantity: 1000,
                completed_state: WindowDressingState {
                    position: -i as u8,
                    tilt: 90
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
                tilt: 90
            },
        })
    );
    assert_eq!(seq.get_next_instruction(), None);
}
