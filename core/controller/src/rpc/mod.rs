mod serial;
#[cfg(feature = "host-usb")]
pub mod usb_cdc_acm;

use blinds_sequencer::WindowDressingState;
use serde::{Deserialize, Serialize};
#[cfg(feature = "host-uart")]
pub use serial::*;

#[allow(async_fn_in_trait)]
pub trait AsyncRpc {
    type Error: defmt::Format;

    async fn read(&mut self) -> Result<Option<IncomingRpcPacket>, Self::Error>;
    async fn write(&mut self, packet: &OutgoingRpcPacket) -> Result<(), Self::Error>;
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub enum IncomingRpcPacket {
    Home {
        channel: u8,
    },
    Setup {
        channel: u8,
        init: WindowDressingState,
        full_cycle_steps: u32,
        reverse: Option<bool>,
        full_tilt_steps: Option<u32>,
        #[cfg(feature = "stallguard")]
        sgthrs: Option<u8>,
    },
    Set {
        channel: u8,
        position: Option<u8>,
        tilt: Option<i8>,
    },
    Get {
        channel: u8,
    },
    #[cfg(feature = "stallguard")]
    GetStallGuardResult {
        channel: u8,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutgoingRpcPacket {
    Ready {},
    Position {
        channel: u8,
        current: WindowDressingState,
        desired: WindowDressingState,
    },
    #[cfg(feature = "stallguard")]
    StallGuardResult {
        channel: u8,
        sg_result: u8,
    },
}
