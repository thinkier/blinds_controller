use crate::{WindowDressingInstruction, WindowDressingSequencer};
use core::time::Duration;

pub struct FixedFrequencyStepperModulator<S: WindowDressingSequencer> {
    pub(crate) period: Duration,
    pub(crate) sequencer: S,
    pub(crate) cur_instruction: Option<WindowDressingInstruction>,
}

// pub struct LinearRampingModulator<S: WindowDressingSequencer> {
//     pub(crate) main_period: Duration,
//     pub(crate) base_period: Duration,
//     pub(crate) ramping_duration: Duration,
//     pub(crate) sequencer: S,
//     pub(crate) cur_instruction: Option<WindowDressingInstruction>,
//     pub(crate) next_instruction: Option<WindowDressingInstruction>,
// }
