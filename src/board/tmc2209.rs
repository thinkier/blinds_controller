use crate::board::Board;
use defmt::*;
use embedded_io::Write;
use tmc2209::reg::{CHOPCONF, GCONF, IHOLD_IRUN};
use tmc2209::send_write_request;

impl<'a, const N: usize, D, S> Board<'a, N, D, S>
where
    D: Write,
    D::Error: Format,
{
    pub fn configure_driver(&mut self) {
        let mut gconf = GCONF::default();
        gconf.set_mstep_reg_select(true); // Must be written prior to setting MRES in CHOPCONF
        let mut chop = CHOPCONF::default();
        chop.set_vsense(false); // Essential for using the 0R11 external sense resistors on the board, which will program the driver to run at approximately ~1.7A
        chop.set_mres(0b0111); // Half step mode (full-step has insane grinding problems)
        let mut current = IHOLD_IRUN::default();
        current.set_ihold(0b10000); // 1/2 of the max current

        for addr in 0..N as u8 {
            if let Err(e) = send_write_request(addr, gconf, &mut self.driver_serial) {
                warn!("Failed to program GCONF on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request(addr, chop, &mut self.driver_serial) {
                warn!("Failed to program CHOPCONF on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request(addr, current, &mut self.driver_serial) {
                warn!("Failed to program IHOLD_IRUN on addr {}: {:?}", addr, e);
            }
        }
    }
}
