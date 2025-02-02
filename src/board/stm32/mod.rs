pub struct Board<const N: usize, D, H> {}

impl<const N: usize, D, H> Board<N, D, H> {
    pub fn init(serial_buffers: &'static mut crate::board::SerialBuffers) -> Self {
        defmt::unimplemented!();
    }
}
