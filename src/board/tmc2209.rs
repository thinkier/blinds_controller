use crate::board::Board;
use defmt::*;
use embassy_time::Timer;
use embedded_io::{Read, Write};
use tmc2209::reg::{CHOPCONF, COOLCONF, GCONF, IHOLD_IRUN, SGTHRS, SG_RESULT, SLAVECONF, TCOOLTHRS, TPWMTHRS, TSTEP};
use tmc2209::{await_read, send_read_request, send_write_request};

impl<'a, const N: usize, D, S> Board<'a, N, D, S>
where
    D: Write + Read,
    D::Error: Format,
{
    pub async fn configure_driver(&mut self) {
        let mut gconf = GCONF::default();
        gconf.set_mstep_reg_select(true); // Must be written prior to setting MRES in CHOPCONF
        let mut chop = CHOPCONF::default();
        chop.set_vsense(false); // Essential for using the 0R11 external sense resistors on the board, which will program the driver to run at approximately ~1.7A
        chop.set_mres(0b1000); // Full-step mode (no grinding with PIO SqrWav Generator
        let tcoolthrs = TCOOLTHRS(0);
        let tpwmthrs = TPWMTHRS(0xFFFFF);
        let slaveconf = SLAVECONF(2 << 8); // Apply minimum SENDDELAY for a multi-driver system
        let coolconf = COOLCONF(0); // Disable CoolStep

        for addr in 0..N as u8 {
            if let Err(e) = send_write_request(addr, gconf, &mut self.driver_serial) {
                warn!("Failed to program GCONF on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request(addr, chop, &mut self.driver_serial) {
                warn!("Failed to program CHOPCONF on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request(addr, tcoolthrs, &mut self.driver_serial) {
                warn!("Failed to program TCOOLTHRS on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request(addr, tpwmthrs, &mut self.driver_serial) {
                warn!("Failed to program TPWMTHRS on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request(addr, slaveconf, &mut self.driver_serial) {
                warn!("Failed to program SLAVECONF on addr {}: {:?}", addr, e);
            }
            if let Err(e) = send_write_request(addr, coolconf, &mut self.driver_serial) {
                warn!("Failed to program COOLCONF on addr {}: {:?}", addr, e);
            }

            for _ in 0..6 {
                let _ = self.driver_serial.sink_write_packet();
            }

            if let Err(e) = send_read_request::<GCONF, _>(addr, &mut self.driver_serial) {
                defmt::warn!("Failed to request GCONF on addr {}: {:?}", addr, e);
            }

            Timer::after_millis(50).await;
            self.driver_serial.sink_read_req_packet();
            match await_read::<GCONF, _>(&mut self.driver_serial) {
                Ok(gconf) => {
                    defmt::info!("GCONF on addr {}: {:b}", addr, gconf.0);
                }
                Err(_) => {
                    defmt::warn!("Failed to read GCONF on addr {}", addr);
                }
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
    fn sink_write_packet(&mut self);
    fn sink_read_req_packet(&mut self);
}

impl<S> SingleLineCommunication for S
where
    S: Read + Write,
    S::Error: defmt::Format,
{
    fn sink_write_packet(&mut self) {
        let _ = self.flush();
        let _ = self.read_exact(&mut [0u8; 8]);
    }
    fn sink_read_req_packet(&mut self) {
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
        if let Err(e) = send_read_request::<GCONF, _>(addr, self) {
            defmt::warn!("Failed to request GCONF on addr {}: {:?}", addr, e);
        }
        self.sink_read_req_packet();
        match await_read::<GCONF, _>(self) {
            Ok(gconf) => {
                defmt::info!("GCONF on addr {}: {:b}", addr, gconf.0);
            }
            Err(_) => {
                defmt::warn!("Failed to read GCONF on addr {}", addr);
            }
        }

        Timer::after_millis(5).await;

        if let Err(e) = send_read_request::<TSTEP, _>(addr, self) {
            defmt::warn!("Failed to request DRV_STATUS on addr {}: {:?}", addr, e);
        }
        self.sink_read_req_packet();
        match await_read::<TSTEP, _>(self) {
            Ok(tstep) => {
                defmt::info!("TSTEP on addr {}: {}", addr, tstep.get());
            }
            Err(_) => {
                defmt::warn!("Failed to read DRV_STATUS on addr {}", addr);
            }
        }

        Timer::after_millis(5).await;

        if let Err(e) = send_read_request::<SG_RESULT, _>(addr, self) {
            defmt::warn!("Failed to request SG_RESULT on addr {}: {:?}", addr, e);
        }
        self.sink_read_req_packet();

        match await_read::<SG_RESULT, _>(self) {
            Ok(sg_result) => {
                defmt::info!("SG_RESULT on addr {}: {}", addr, sg_result.get());
            }
            Err(_) => {
                defmt::warn!("Failed to read SG_RESULT on addr {}", addr);
            }
        }
    }
}
