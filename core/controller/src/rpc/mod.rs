#[cfg(feature = "host-uart")]
mod serial;
#[cfg(feature = "host-usb")]
mod usb_cdc_acm;

use sequencer::WindowDressingState;
use serde::{Deserialize, Serialize};
#[cfg(feature = "host-uart")]
pub use serial::*;
#[cfg(feature = "host-usb")]
pub use usb_cdc_acm::*;

pub trait AsyncRpcError {
    fn is_broken_input(&self) -> bool;
}

#[allow(async_fn_in_trait)]
pub trait AsyncRpc {
    type Error: AsyncRpcError + defmt::Format;

    async fn peek(&mut self) -> Result<Option<IncomingRpcPacket>, Self::Error>;
    async fn read(&mut self) -> Result<Option<IncomingRpcPacket>, Self::Error>;
    async fn write(&mut self, packet: &OutgoingRpcPacket) -> Result<(), Self::Error>;
    /// The default implementation is to call write repeatedly,
    /// but implementers can choose to optimize the calls
    async fn write_bulk(
        &mut self,
        packets: impl Iterator<Item = &OutgoingRpcPacket>,
    ) -> Result<(), Self::Error> {
        for packet in packets {
            self.write(packet).await?;
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub enum IncomingRpcPacket {
    Home {
        channel: u8,
    },
    Setup {
        channel: u8,
        init: Option<WindowDressingState>,
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
    // This is not normally available to a generic Serial RPC caller,
    // it could be triggered by a side-channel flag like
    // - Lowering the baud rate below 1200Hz per Arduino / pico-sdk convention
    #[serde(skip)]
    Bootloader,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutgoingRpcPacket {
    Absent {
        channel: u8,
    },
    Ready {},
    Position {
        channel: u8,
        #[serde(skip_serializing_if = "is_false")]
        notify: bool,
        current: WindowDressingState,
        desired: WindowDressingState,
    },
    #[cfg(feature = "stallguard")]
    StallGuardResult {
        channel: u8,
        sg_result: u8,
    },
}

fn is_false(b: &bool) -> bool {
    !b
}
