#![feature(core_intrinsics)]
extern crate core;
extern crate libc;
extern crate mmap;
extern crate byteorder;

use core::mem::uninitialized;
use core::ops::{BitAnd, BitOr, Not};
use core::intrinsics::{volatile_load, volatile_store};
use core::fmt;

#[macro_use]
pub mod bitops;
pub mod timedops;
pub mod mem;

#[repr(packed)]
pub struct Volatile<T> {
    value: T,
}

impl<T> Volatile<T>
    where T: Copy + PartialEq + BitAnd<Output = T> + BitOr<Output = T> + Not<Output = T>
{
    pub fn new() -> Self {
        Volatile { value: unsafe { uninitialized() } }
    }

    /// Create a volatile with an initial value.
    pub fn with_value(value: T) -> Volatile<T> {
        Volatile { value: value }
    }

    #[inline]
    pub fn get(&self) -> T {
        unsafe { volatile_load(&self.value) }
    }

    pub fn set(&self, value: T) {
        unsafe { volatile_store(&self.value as *const T as *mut T, value) }
    }
}

impl<T: fmt::Debug> fmt::Debug for Volatile<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

impl<T: fmt::Display> fmt::Display for Volatile<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
