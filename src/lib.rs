#![feature(libc)]
#![feature(core_intrinsics)]
extern crate byteorder;
extern crate core;
extern crate libc;

#[cfg(target_os = "linux")]
extern crate mmap;

#[cfg(target_os = "barrelfish")]
extern crate libbarrelfish;

extern crate x86;

use core::fmt;
use core::intrinsics::{volatile_load, volatile_store};
use core::mem::uninitialized;
use core::ops::{BitAnd, BitOr, Not};

#[macro_use]
pub mod bitops;
pub mod timedops;

#[cfg(target_os = "barrelfish")]
mod barrelfish;

#[cfg(target_os = "linux")]
mod linux;

pub mod mem {
    #[cfg(target_os = "barrelfish")]
    pub use barrelfish::mem::*;

    #[cfg(target_os = "linux")]
    pub use linux::mem::*;
}

/// Driver life-cycle management trait
pub trait DriverControl: Sized {
    fn init(&mut self) {}

    fn attach(&mut self);

    fn detach(&mut self);

    fn set_sleep_level(&mut self, level: usize);

    fn destroy(mut self) {
        self.detach();
    }
}

#[repr(packed)]
pub struct Volatile<T> {
    value: T,
}

impl<T> Volatile<T>
where
    T: Copy + PartialEq + BitAnd<Output = T> + BitOr<Output = T> + Not<Output = T>,
{
    pub fn new() -> Self {
        Volatile {
            value: unsafe { uninitialized() },
        }
    }

    /// Create a volatile with an initial value.
    pub fn with_value(value: T) -> Volatile<T> {
        Volatile { value: value }
    }

    #[inline]
    pub fn get(&self) -> T {
        unsafe { volatile_load(&self.value) }
    }

    #[inline]
    pub fn set(&mut self, value: T) {
        unsafe { volatile_store(&self.value as *const T as *mut T, value) }
    }
}

impl<T: fmt::Debug> fmt::Debug for Volatile<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { write!(f, "{:?}", self.value) }
    }
}

impl<T: fmt::Display> fmt::Display for Volatile<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { write!(f, "{}", self.value) }
    }
}

pub trait MsrInterface {
    unsafe fn write(&mut self, msr: u32, value: u64) {
        x86::msr::wrmsr(msr, value);
    }

    unsafe fn read(&mut self, msr: u32) -> u64 {
        x86::msr::rdmsr(msr)
    }
}
