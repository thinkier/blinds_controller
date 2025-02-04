use crate::board::stm32::Board;
use crate::board::SerialBuffers;
use embassy_stm32::usart::BufferedUart;

impl Board<'static, 5, BufferedUart<'static>, BufferedUart<'static>> {
    pub fn init(serial_buffers: &'static mut SerialBuffers) -> Self {
        unimplemented!()
    }
}
