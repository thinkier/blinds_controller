use crate::board::{ConfigurableBoard, ConfigurableDriver, StallGuard};
use defmt::*;
use embassy_time::Timer;
use embedded_io::{ErrorType, Read, Write};
use tmc2209::reg::{CHOPCONF, COOLCONF, GCONF, SGTHRS, SG_RESULT, SLAVECONF, TCOOLTHRS, TPWMTHRS};
use tmc2209::{await_read, send_read_request, send_write_request};

#[cfg(feature = "uart_soft_half_duplex")]
const DATAGRAM_SIZE_READ_REQ: usize = 4;
#[cfg(feature = "uart_soft_half_duplex")]
const DATAGRAM_SIZE_WRITE_REQ: usize = 8;

impl<B, S, const N: usize> ConfigurableDriver<S, N> for B
where
    B: ConfigurableBoard<N, DriverSerial = S>,
    S: Read + Write,
    <S as ErrorType>::Error: Format,
{
    async fn configure_driver(&mut self) {
        let mut gconf = GCONF::default();
        gconf.set_mstep_reg_select(true); // Must be written prior to setting MRES in CHOPCONF
        let mut chop = CHOPCONF::default();
        chop.set_vsense(false); // Essential for using the 0R11 external sense resistors on the board, which will program the driver to run at approximately ~1.7A
        chop.set_mres(0b1000); // Full-step mode (no grinding with PIO SqrWav Generator
        let tcoolthrs = TCOOLTHRS(0xFFFFF);
        let tpwmthrs = TPWMTHRS(0);
        let slaveconf = SLAVECONF(2 << 8); // Apply minimum SENDDELAY for a multi-driver system
        let coolconf = COOLCONF(0); // Disable CoolStep
        let sgthrs = SGTHRS(100);

        for addr in 0..N as u8 {
            let ser = self.driver_serial(addr);
            #[cfg(not(feature = "uart_driver_shared_bus"))]
            let addr = 0;

            if let Err(e) = send_write_request(addr, gconf, ser) {
                warn!("Failed to program GCONF on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request(addr, chop, ser) {
                warn!("Failed to program CHOPCONF on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request(addr, tcoolthrs, ser) {
                warn!("Failed to program TCOOLTHRS on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request(addr, tpwmthrs, ser) {
                warn!("Failed to program TPWMTHRS on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request(addr, slaveconf, ser) {
                warn!("Failed to program SLAVECONF on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request(addr, coolconf, ser) {
                warn!("Failed to program COOLCONF on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request(addr, sgthrs, ser) {
                warn!("Failed to program SGTHRS on addr {}: {:?}", addr, e);
            }

            #[cfg(feature = "uart_soft_half_duplex")]
            for _ in 0..7 {
                use crate::board::SoftHalfDuplex;
                ser.flush_clear::<DATAGRAM_SIZE_WRITE_REQ>().await;
            }
            Timer::after_millis(50).await;
        }
    }
}

impl<B, S, const N: usize> StallGuard<S, N> for B
where
    B: ConfigurableBoard<N, DriverSerial = S>,
    S: Read + Write,
    <S as ErrorType>::Error: Format,
{
    async fn set_sg_threshold(&mut self, addr: u8, sgthrs: u8) {
        let serial = self.driver_serial(addr);
        #[cfg(not(feature = "uart_driver_shared_bus"))]
        let addr = 0;

        let sgthrs = SGTHRS(sgthrs as u32);
        if let Err(e) = send_write_request(addr, sgthrs, serial) {
            warn!("Failed to program SGTHRS on addr {}: {:?}", addr, e);
        }
        #[cfg(feature = "uart_soft_half_duplex")]
        {
            use crate::board::SoftHalfDuplex;
            let _ = serial.flush_clear::<DATAGRAM_SIZE_WRITE_REQ>().await;
        }
    }

    /// For API-compatibility with other StallGuard drivers, this function returns a halved SG_RESULT value
    async fn get_sg_result(&mut self, addr: u8) -> Option<u8> {
        let serial = self.driver_serial(addr);
        #[cfg(not(feature = "uart_driver_shared_bus"))]
        let addr = 0;

        if let Err(e) = send_read_request::<SG_RESULT, _>(addr, serial) {
            defmt::warn!("Failed to request SG_RESULT on addr {}: {:?}", addr, e);
            return None;
        }
        #[cfg(feature = "uart_soft_half_duplex")]
        {
            use crate::board::SoftHalfDuplex;
            let _ = serial.flush_clear::<DATAGRAM_SIZE_READ_REQ>().await;
        }
        match await_read::<SG_RESULT, _>(serial) {
            Ok(sg_result) => {
                defmt::info!("SG_RESULT/2 on addr {}: {}", addr, sg_result.get() / 2);
                Some((sg_result.get() / 2) as u8)
            }
            Err(_) => {
                defmt::warn!("Failed to read SG_RESULT on addr {}", addr);
                None
            }
        }
    }
}
