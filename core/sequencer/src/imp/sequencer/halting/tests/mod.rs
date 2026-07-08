use crate::{HaltingSequencer, WindowDressingState};

mod comparator;
mod roller;
mod roller_grouped;
mod roller_ramming;
mod tilt;
mod venetian;

#[test]
fn construct_roller() {
    let expect = HaltingSequencer::<1>::new(1000, None);
    let actual = HaltingSequencer::<1>::new_roller(1000);

    assert_eq!(expect.full_cycle_quantity, actual.full_cycle_quantity);
    assert_eq!(expect.full_tilt_quantity, actual.full_tilt_quantity);
    assert_eq!(expect.current_state, WindowDressingState::default());
    assert_eq!(expect.desired_state, WindowDressingState::default());
}

#[test]
fn construct_venetian() {
    let expect = HaltingSequencer::<1>::new(1000, Some(50));
    let actual = HaltingSequencer::<1>::new_venetian(1000, 50);

    assert_eq!(expect.full_cycle_quantity, actual.full_cycle_quantity);
    assert_eq!(expect.full_tilt_quantity, actual.full_tilt_quantity);
    assert_eq!(expect.current_state, WindowDressingState::default());
    assert_eq!(expect.desired_state, WindowDressingState::default());
}
