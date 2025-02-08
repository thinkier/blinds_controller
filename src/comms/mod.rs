mod serial_rpc;
#[cfg(feature = "host-usb")]
pub mod usb_cdc_acm_rpc;

#[cfg(feature = "host-uart")]
pub use serial_rpc::*;
