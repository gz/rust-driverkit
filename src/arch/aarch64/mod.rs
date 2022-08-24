// aarch64 specific driver kit functionality

pub use armv8::aarch64::vm::granule4k::{IOAddr, PAddr, VAddr};

pub trait MsrInterface {
    unsafe fn write(&mut self, msr: u32, value: u64) {
        panic!("NYI!");
    }
    unsafe fn read(&mut self, msr: u32) -> u64 {
        panic!("NYI!");
    }
}

pub trait PciInterface {
    const PCI_CONF_ADDR: u16 = 0xcf8;
    const PCI_CONF_DATA: u16 = 0xcfc;

    fn read(&self, addr: u32) -> u32 {
        panic!("NYI!");
    }
    fn write(&mut self, addr: u32, value: u32) {
        panic!("NYI!");
    }
}

impl PciInterface for PCIAddress {
    fn read(&self, offset: u32) -> u32 {
        panic!("NYI!");
    }

    fn write(&mut self, offset: u32, value: u32) {
        panic!("NYI!");
    }
}
