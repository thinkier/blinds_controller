use core::cell::RefCell;
use critical_section::Mutex;

pub struct LookAheadBuffer<T, const N: usize> {
    inner: [Mutex<RefCell<Option<T>>>; N],
}

impl<T, const N: usize> LookAheadBuffer<T, N> {
    pub const fn new() -> Self {
        LookAheadBuffer {
            inner: [const { Mutex::new(RefCell::new(None)) }; N],
        }
    }

    pub fn has(&self, index: usize) -> bool {
        critical_section::with(|cs| self.inner[index].borrow_ref(cs).is_some())
    }

    pub fn put(&self, index: usize, value: T) {
        critical_section::with(|cs| self.inner[index].replace(cs, Some(value)));
    }

    pub fn take(&self, index: usize) -> Option<T> {
        critical_section::with(|cs| self.inner[index].replace(cs, None))
    }
}
