use crate::board::stm32::Board;
use crate::board::SerialBufferPair;
use embassy_stm32::usart::BufferedUart;

impl Board<'static, 8, BufferedUart<'static>, BufferedUart<'static>> {
    pub fn init(serial_buffers: &'static mut SerialBufferPair) -> Self {
        unimplemented!()
    }
}
