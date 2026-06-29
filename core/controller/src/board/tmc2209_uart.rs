use crate::board::{ConfigurableStepStickDriver, ConfigurableStepStickHost};
use defmt::*;
use embedded_io_async::{ErrorType, Read, Write};
use tmc2209_async::data::MicroStepResolution;
use tmc2209_async::reg::{CHOPCONF, COOLCONF, GCONF, SLAVECONF, TCOOLTHRS, TPWMTHRS};
#[cfg(feature = "stallguard")]
use tmc2209_async::reg::{SGTHRS, SG_RESULT};
use tmc2209_async::{ReadableRegister, WritableRegister};

#[cfg(feature = "uart_soft_half_duplex")]
#[allow(unused)]
const DATAGRAM_SIZE_READ_REQ: usize = 4;
#[cfg(feature = "uart_soft_half_duplex")]
const DATAGRAM_SIZE_WRITE_REQ: usize = 8;

impl<B, S, const N: usize> ConfigurableStepStickDriver<S, N> for B
where
    B: ConfigurableStepStickHost<N, DriverSerial = S>,
    S: Read + Write,
    <S as ErrorType>::Error: Format,
{
    async fn configure_driver(&mut self) {
        let mut gconf = GCONF::default();
        gconf.set_mstep_reg_select(true); // Must be written prior to setting MRES in CHOPCONF
        let mut chop = CHOPCONF::default();
        chop.set_vsense(false); // Essential for using the 0R11 external sense resistors on the board, which will program the driver to run at approximately ~1.7A
        chop.set_mres(MicroStepResolution::new(0)); // Full-step mode (no grinding with PIO SqrWav Generator
        let tcoolthrs = TCOOLTHRS(0xFFFFF);
        let tpwmthrs = TPWMTHRS(0);
        let slaveconf = SLAVECONF(2 << 8); // Apply minimum SENDDELAY for a multi-driver system
        let coolconf = COOLCONF(0); // Disable CoolStep
        #[cfg(feature = "stallguard")]
        let sgthrs = SGTHRS(100);

        for addr in 0..N as u8 {
            let ser = self.driver_serial(addr);
            #[cfg(not(feature = "uart_driver_shared_bus"))]
            let addr = 0;

            if let Err(e) = send_write_request_safe(addr, gconf, &mut *ser).await {
                warn!("Failed to program GCONF on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request_safe(addr, chop, &mut *ser).await {
                warn!("Failed to program CHOPCONF on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request_safe(addr, tcoolthrs, &mut *ser).await {
                warn!("Failed to program TCOOLTHRS on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request_safe(addr, tpwmthrs, &mut *ser).await {
                warn!("Failed to program TPWMTHRS on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request_safe(addr, slaveconf, &mut *ser).await {
                warn!("Failed to program SLAVECONF on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request_safe(addr, coolconf, &mut *ser).await {
                warn!("Failed to program COOLCONF on addr {}: {:?}", addr, e);
            }
            #[cfg(feature = "stallguard")]
            if let Err(e) = send_write_request_safe(addr, sgthrs, &mut *ser).await {
                warn!("Failed to program SGTHRS on addr {}: {:?}", addr, e);
            }
        }
    }
}

#[cfg(feature = "uart_soft_half_duplex")]
#[allow(unused)]
async fn send_read_request_safe<R, U>(
    addr: u8,
    mut tx: U,
) -> Result<(), <U as ErrorType>::Error>
where
    R: ReadableRegister,
    U: crate::board::SoftHalfDuplex + Write,
{
    tmc2209_async::send_read_request::<R, _>(addr, &mut tx).await?;

    tx.flush_clear::<DATAGRAM_SIZE_READ_REQ>().await;

    Ok(())
}

#[cfg(feature = "uart_soft_half_duplex")]
async fn send_write_request_safe<R, U>(
    addr: u8,
    reg: R,
    mut tx: U,
) -> Result<(), <U as ErrorType>::Error>
where
    R: WritableRegister,
    U: crate::board::SoftHalfDuplex + Write,
{
    tmc2209_async::send_write_request(addr, reg, &mut tx).await?;

    tx.flush_clear::<DATAGRAM_SIZE_WRITE_REQ>().await;

    Ok(())
}

#[cfg(not(feature = "uart_soft_half_duplex"))]
#[allow(unused)]
use tmc2209_async::{
    send_read_request as send_read_request_safe, send_write_request as send_write_request_safe,
};

#[cfg(feature = "stallguard")]
impl<B, S, const N: usize> crate::board::StallGuard<S, N> for B
where
    B: ConfigurableStepStickHost<N, DriverSerial = S>,
    S: Read + Write,
    <S as ErrorType>::Error: Format,
{
    async fn set_sg_threshold(&mut self, addr: u8, sgthrs: u8) {
        let serial = self.driver_serial(addr);
        #[cfg(not(feature = "uart_driver_shared_bus"))]
        let addr = 0;

        let sgthrs = SGTHRS(sgthrs as u32);
        if let Err(e) = send_write_request_safe(addr, sgthrs, &mut *serial).await {
            warn!("Failed to program SGTHRS on addr {}: {:?}", addr, e);
        }
    }

    /// For API-compatibility with other StallGuard drivers, this function returns a halved SG_RESULT value
    async fn get_sg_result_halved(&mut self, addr: u8) -> Option<u8> {
        let serial = self.driver_serial(addr);
        #[cfg(not(feature = "uart_driver_shared_bus"))]
        let addr = 0;

        if let Err(e) = send_read_request_safe::<SG_RESULT, _>(addr, &mut *serial).await {
            defmt::warn!("Failed to request SG_RESULT on addr {}: {:?}", addr, e);
            return None;
        }
        match tmc2209_async::await_read::<SG_RESULT, _>(serial).await {
            Ok(sg_result) => Some((sg_result.get() / 2) as u8),
            Err(_) => {
                defmt::warn!("Failed to read SG_RESULT on addr {}", addr);
                None
            }
        }
    }
}
