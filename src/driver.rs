use crate::board::DriverPins;
use blinds_sequencer::{Direction, WindowDressingInstruction};
use core::mem;

pub fn dir_hold(pins: &mut DriverPins, direction: Option<Direction>) {
    if let Some(direction) = direction {
        pins.enable.set_low();
        match direction {
            Direction::Extend => {
                pins.dir.set_high();
            }
            Direction::Retract => {
                pins.dir.set_low();
            }
            Direction::Hold => {}
        }
    } else {
        pins.enable.set_high();
    }
}

pub fn stp_rise(pins: &mut DriverPins, instr: &mut Option<WindowDressingInstruction>) {
    if instr.map(|i| i.quantity).unwrap_or(0) == 0 {
        let _ = mem::replace(instr, None);
        return;
    }

    if let Some(instr) = instr {
        instr.quantity -= 1;

        match instr.quality {
            Direction::Extend => {
                pins.step.set_high();
            }
            Direction::Retract => {
                pins.step.set_high();
            }
            Direction::Hold => {}
        }
    }
}

pub fn stp_fall(pins: &mut DriverPins) {
    pins.step.set_low();
}
