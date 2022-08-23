// x86 specific driver kit functionality

extern crate x86;

use crate::pci::PCIAddress;

pub trait MsrInterface {
    unsafe fn write(&mut self, msr: u32, value: u64) {
        x86::msr::wrmsr(msr, value);
    }

    unsafe fn read(&mut self, msr: u32) -> u64 {
        x86::msr::rdmsr(msr)
    }
}

pub trait PciInterface {
    const PCI_CONF_ADDR: u16 = 0xcf8;
    const PCI_CONF_DATA: u16 = 0xcfc;

    fn read(&self, addr: u32) -> u32 {
        unsafe {
            x86::io::outl(Self::PCI_CONF_ADDR, addr);
            x86::io::inl(Self::PCI_CONF_DATA)
        }
    }

    fn write(&mut self, addr: u32, value: u32) {
        unsafe {
            x86::io::outl(Self::PCI_CONF_ADDR, addr);
            x86::io::outl(Self::PCI_CONF_DATA, value);
        }
    }
}

impl PciInterface for PCIAddress {
    fn read(&self, offset: u32) -> u32 {
        let addr = self.addr() | offset;

        unsafe {
            x86::io::outl(<Self as PciInterface>::PCI_CONF_ADDR, addr);
            x86::io::inl(<Self as PciInterface>::PCI_CONF_DATA)
        }
    }

    fn write(&mut self, offset: u32, value: u32) {
        let addr = self.addr() | offset;

        unsafe {
            x86::io::outl(<Self as PciInterface>::PCI_CONF_ADDR, addr);
            x86::io::outl(<Self as PciInterface>::PCI_CONF_DATA, value);
        }
    }
}
