use crate::model::sequencer::WindowDressingState;

#[test]
fn more_open_is_greater() {
    let lesser = WindowDressingState {
        position: 0,
        tilt: 0,
    };
    let greater = WindowDressingState {
        position: 100,
        tilt: 0,
    };

    assert!(greater > lesser);
}

#[test]
fn same_position_tiebreaks() {
    let a = WindowDressingState {
        position: 0,
        tilt: 0,
    };
    let b = WindowDressingState {
        position: 0,
        tilt: 1,
    };

    assert_ne!(a, b);
}

#[test]
fn identical_equals() {
    let a = WindowDressingState {
        position: 0,
        tilt: 0,
    };
    let b = WindowDressingState {
        position: 0,
        tilt: 0,
    };

    assert_eq!(a, b);
}

#[test]
fn max_extend_is_less_than_everything() {
    let max_extend = WindowDressingState {
        position: 0,
        tilt: 90,
    };
    let other = WindowDressingState {
        position: 0,
        tilt: 0,
    };

    assert!(other > max_extend);
}
