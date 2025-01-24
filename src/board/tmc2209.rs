use crate::board::{ConfigurableBoard, ConfigurableDriver};
use defmt::*;
use embassy_time::Timer;
use embedded_io::{ErrorType, Read, Write};
use tmc2209::reg::{CHOPCONF, COOLCONF, GCONF, SGTHRS, SG_RESULT, SLAVECONF, TCOOLTHRS, TPWMTHRS};
use tmc2209::{await_read, send_read_request, send_write_request};

impl<B, S, const N: usize> ConfigurableDriver<S, N> for B
where
    B: ConfigurableBoard<N, DriverSerial = S>,
    S: Read + Write,
    <S as ErrorType>::Error: Format,
{
    async fn configure_driver(&mut self) {
        let ser = self.driver_serial();

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

            for _ in 0..7 {
                ser.sink_write_packet().await;
            }
            Timer::after_millis(50).await;
        }
    }
}

pub trait SetSgthrs {
    fn set_sgthrs(&mut self, addr: u8, sgthrs: u8);
}

impl<W> SetSgthrs for W
where
    W: Write + Read,
    W::Error: defmt::Format,
{
    fn set_sgthrs(&mut self, addr: u8, sgthrs: u8) {
        let sgthrs = SGTHRS(sgthrs as u32);
        if let Err(e) = send_write_request(addr, sgthrs, self) {
            warn!("Failed to program SGTHRS on addr {}: {:?}", addr, e);
        }
        let _ = self.sink_write_packet();
    }
}

pub trait SingleLineCommunication {
    async fn sink_write_packet(&mut self);
    async fn sink_read_req_packet(&mut self);
}

impl<S> SingleLineCommunication for S
where
    S: Read + Write,
    S::Error: defmt::Format,
{
    async fn sink_write_packet(&mut self) {
        Timer::after_millis(50).await;
        let _ = self.flush();
        let _ = self.read_exact(&mut [0u8; 8]);
    }
    async fn sink_read_req_packet(&mut self) {
        Timer::after_millis(50).await;
        let _ = self.flush();
        let _ = self.read_exact(&mut [0u8; 4]);
    }
}

pub trait ReadSgDiagnostics {
    async fn read_sg_diagnostics(&mut self, addr: u8);
}

impl<S> ReadSgDiagnostics for S
where
    S: Write + Read,
    S::Error: defmt::Format,
{
    async fn read_sg_diagnostics(&mut self, addr: u8) {
        if let Err(e) = send_read_request::<SG_RESULT, _>(addr, self) {
            defmt::warn!("Failed to request SG_RESULT on addr {}: {:?}", addr, e);
        }
        self.sink_read_req_packet().await;

        match await_read::<SG_RESULT, _>(self) {
            Ok(sg_result) => {
                defmt::info!("SG_RESULT/2 on addr {}: {}", addr, sg_result.get() / 2);
            }
            Err(_) => {
                defmt::warn!("Failed to read SG_RESULT on addr {}", addr);
            }
        }
    }
}
