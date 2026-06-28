use core::error::Error as CoreError;
use core::fmt::{Debug, Display, Formatter};
use core::marker::PhantomData;
use embedded_io::{Error, ErrorKind, ErrorType, Read, Write};

pub struct BitbangedHalfDuplexUart<'a, T> {
    pub pin: PhantomData<&'a mut T>, // TODO
}

impl<T> ErrorType for BitbangedHalfDuplexUart<'_, T> {
    type Error = BitbangedUartError;
}

impl<T> Read for BitbangedHalfDuplexUart<'_, T>
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, BitbangedUartError> {
        todo!()
    }
}

impl<T> Write for BitbangedHalfDuplexUart<'_, T>
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, BitbangedUartError> {
        todo!()
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
