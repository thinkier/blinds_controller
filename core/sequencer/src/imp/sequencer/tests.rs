use crate::{Direction, WindowDressingState};

#[test]
fn reverse_direction() {
    assert_eq!(Direction::Retract, Direction::Extend.reverse());
    assert_eq!(Direction::Extend, Direction::Retract.reverse());
    assert_eq!(Direction::Hold, Direction::Hold.reverse());
}

#[test]
fn defaults_closed() {
    assert_eq!(
        WindowDressingState {
            position: 0,
            tilt: 0,
        },
        WindowDressingState::default()
    );
}
