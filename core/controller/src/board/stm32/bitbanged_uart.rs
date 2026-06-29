use core::cell::RefCell;
use core::error::Error as CoreError;
use core::fmt::{Debug, Display, Formatter};
use embassy_stm32::gpio::{Flex, Level, Pull, Speed};
use embedded_io::{Error, ErrorKind, ErrorType, Read, Write};

pub struct BitbangedHalfDuplexUart<'a, T> {
    pub pin: Flex<'a>,
    pub tim: &'a RefCell<T>,
}

impl<'a, T> BitbangedHalfDuplexUart<'a, T> {
    pub fn new(mut pin: Flex<'a>, tim: &'a RefCell<T>) -> Self {
        // STM document DS13560 suggests that worst case $f_{max}$ is 1MHz in any case
        // which is ample for whatever we're doing here, aiming for 115200bd
        pin.set_as_input_output_pull(Speed::Low, Pull::Up);

        Self { pin, tim }
    }
}

impl<T> ErrorType for BitbangedHalfDuplexUart<'_, T> {
    type Error = BitbangedUartError;
}

impl<T> Read for BitbangedHalfDuplexUart<'_, T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, BitbangedUartError> {
        while self.pin.get_level() != Level::Low {
            // Busy wait for the other end to pull low
        }
        cortex_m::interrupt::free(||{
            let tim = self.tim.borrow_mut();

            
        });

        Ok(1)
    }
}

impl<T> Write for BitbangedHalfDuplexUart<'_, T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, BitbangedUartError> {
        let len = buf.len();

        Ok(len)
    }

    fn flush(&mut self) -> Result<(), BitbangedUartError> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum BitbangedUartError {}

impl Display for BitbangedUartError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl defmt::Format for BitbangedUartError {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "{:?}", self)
    }
}

impl CoreError for BitbangedUartError {}

impl Error for BitbangedUartError {
    fn kind(&self) -> ErrorKind {
        todo!()
    }
}
