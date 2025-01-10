use core::mem;

pub struct LookAheadBuffer<T, const N: usize> {
    inner: [Option<T>; N],
}

impl<T, const N: usize> LookAheadBuffer<T, N> {
    pub const fn new() -> Self {
        LookAheadBuffer {
            inner: [const { None }; N],
        }
    }

    pub fn put(&mut self, index: usize, value: T) {
        self.inner[index] = Some(value);
    }

    pub fn take(&mut self, index: usize) -> Option<T> {
        mem::replace(&mut self.inner[index], None)
    }
}
