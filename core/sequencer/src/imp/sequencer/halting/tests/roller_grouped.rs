use crate::model::sequencer::{HaltingWindowDressingInstruction, WindowDressingState};
use crate::{Direction, WindowDressingSequencer};
type HaltingSequencer = crate::model::sequencer::HaltingSequencer<1024>;

#[test]
fn close_grouped() {
    let mut seq = HaltingSequencer::new_roller(100_000);
    seq.current_state.position = 100;
    seq.set_position(0);

    assert_eq!(
        seq.get_next_instruction_grouped(u32::MAX),
        Some(HaltingWindowDressingInstruction {
            direction: Direction::Extend,
            quantity: 100_000,
            completed_state: WindowDressingState {
                position: 0,
                tilt: 0
            },
        })
    );

    assert_eq!(
        seq.get_next_instruction_grouped(u32::MAX),
        Some(HaltingWindowDressingInstruction {
            direction: Direction::Hold,
            quantity: 500,
            completed_state: WindowDressingState {
                position: 0,
                tilt: 0
            }
        })
    );
}
