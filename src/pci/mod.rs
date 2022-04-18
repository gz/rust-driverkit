use core::fmt;

use bit_field::BitField;
use x86::io;

pub mod device_db;

use crate::PciInterface;

pub type VendorId = u16;
pub type DeviceId = u16;
pub type DeviceRevision = u8;
pub type BaseClass = u8;
pub type SubClass = u8;
pub type Interface = u8;
pub type HeaderType = u8;

#[derive(Debug)]
pub enum PciDeviceType {
    Endpoint = 0x00,
    PciBridge = 0x01,
    Unknown = 0xff,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PCIAddress {
    bus: u8,
    dev: u8,
    fun: u8,
}

impl PCIAddress {
    fn new(bus: u8, dev: u8, fun: u8) -> Self {
        assert!(dev <= 31);
        assert!(fun <= 7);

        //trace!("address ({:2}:{:2}.{:1})", bus, dev, fun);
        PCIAddress { bus, dev, fun }
    }

    fn addr(&self) -> u32 {
        (1 << 31) | ((self.bus as u32) << 16) | ((self.dev as u32) << 11) | ((self.fun as u32) << 8)
    }
}

impl PciInterface for PCIAddress {
    fn read(&self, offset: u32) -> u32 {
        let addr = self.addr() | offset;

        unsafe {
            io::outl(<Self as PciInterface>::PCI_CONF_ADDR, addr);
            io::inl(<Self as PciInterface>::PCI_CONF_DATA)
        }
    }

    fn write(&mut self, offset: u32, value: u32) {
        let addr = self.addr() | offset;

        unsafe {
            io::outl(<Self as PciInterface>::PCI_CONF_ADDR, addr);
            io::outl(<Self as PciInterface>::PCI_CONF_DATA, value);
        }
    }
}

impl fmt::Debug for PCIAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02}:{:02}.{}", self.bus, self.dev, self.fun)
    }
}

#[derive(Debug)]
pub struct PCIHeader(PCIAddress);

impl PCIHeader {
    pub fn new(bus: u8, device: u8, function: u8) -> Option<Self> {
        let addr = PCIAddress::new(bus, device, function);
        if PCIHeader::is_valid(addr) {
            Some(PCIHeader(addr))
        } else {
            None
        }
    }

    pub fn is_valid(addr: PCIAddress) -> bool {
        addr.read(0) != u32::MAX
    }
}

/// # See also
/// <https://wiki.osdev.org/PCI#Class_Codes>
#[derive(Debug)]
pub enum ClassCode {
    IDEController = 0x0101,
    SATAController = 0x0106,
    EthernetController = 0x0200,
    VGACompatibleController = 0x0300,
    RAMController = 0x0500,
    HostBridge = 0x0600,
    ISABridge = 0x0601,
    OtherBridge = 0x0680,
    Unknown = 0xffff,
}

impl From<u16> for ClassCode {
    fn from(value: u16) -> ClassCode {
        match value {
            0x0101 => ClassCode::IDEController,
            0x0106 => ClassCode::SATAController,
            0x0200 => ClassCode::EthernetController,
            0x0300 => ClassCode::VGACompatibleController,
            0x0500 => ClassCode::RAMController,
            0x0600 => ClassCode::HostBridge,
            0x0601 => ClassCode::ISABridge,
            0x0680 => ClassCode::OtherBridge,
            _ => ClassCode::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BarType {
    IO,
    Mem,
}

impl From<bool> for BarType {
    fn from(value: bool) -> BarType {
        match value {
            true => BarType::IO,
            false => BarType::Mem,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub region_type: BarType,
    pub prefetchable: bool,
    pub address: u64,
    pub size: u64,
}

#[derive(Debug)]
pub struct PciDevice {
    header: PCIHeader,
}

impl PciDevice {
    pub fn new(bus: u8, device: u8, function: u8) -> Option<Self> {
        let header = PCIHeader::new(bus, device, function);
        if let Some(header) = header {
            Some(PciDevice { header })
        } else {
            None
        }
    }

    pub fn device_type(&self) -> PciDeviceType {
        let header = self.header.0.read(0x0c);

        match header.get_bits(16..23) as u8 {
            0x00 => PciDeviceType::Endpoint,
            0x01 => PciDeviceType::PciBridge,
            _ => PciDeviceType::Unknown,
        }
    }

    pub fn vendor_id(&self) -> VendorId {
        self.header.0.read(0x00) as VendorId
    }

    pub fn device_id(&self) -> DeviceId {
        self.header.0.read(0x02) as DeviceId
    }

    pub fn is_bus_master(&self) -> bool {
        self.header.0.read(0x04).get_bit(2)
    }

    pub fn enable_bus_mastering(&mut self) {
        let mut command = self.header.0.read(0x04);
        command.set_bit(2, true);
        self.header.0.write(0x04, command);
    }

    pub fn bar(&mut self, index: u8) -> Option<Bar> {
        match self.device_type() {
            PciDeviceType::Endpoint => assert!(index < 6),
            PciDeviceType::PciBridge => assert!(index < 2),
            PciDeviceType::Unknown => return None,
        }

        let offset = 0x10 + (index as u32) * 4;
        let base = self.header.0.read(offset);
        let bartype_is_io = base.get_bit(0);

        if !bartype_is_io {
            let locatable = base.get_bits(1..3);
            let prefetchable = base.get_bit(3);

            self.header.0.write(offset, u32::MAX);
            let size_encoded = self.header.0.read(offset);
            self.header.0.write(offset, base);

            if size_encoded == 0x0 {
                return None;
            }

            // To get the region size using BARs:
            // - Clear lower 4 bits
            // - Invert all all-bits
            // - Add 1 to the result
            // Ref: https://wiki.osdev.org/PCI#Base_Address_Registers
            let (address, size) = {
                match locatable {
                    // 32-bit address
                    0 => {
                        let size = !(size_encoded & !0xF) + 1;
                        ((base & 0xFFFF_FFF0) as u64, size as u64)
                    }
                    // 64-bit address
                    2 => {
                        let next_offset = offset + 4;
                        let next_bar = self.header.0.read(next_offset);
                        let address = (base & 0xFFFF_FFF0) as u64
                            | (next_bar as u64 & (u32::MAX as u64)) << 32;

                        // Size for 64-bit Memory Space BARs:
                        self.header.0.write(next_offset, u32::MAX);
                        let msb_size_encoded = self.header.0.read(next_offset);
                        self.header.0.write(next_offset, next_bar);
                        let size = (msb_size_encoded as u64) << 32 | size_encoded as u64;

                        (address, (!(size & !0xF) + 1))
                    }
                    _ => unimplemented!("Unsupported locatable: {}", locatable),
                }
            };

            Some(Bar {
                region_type: bartype_is_io.into(),
                prefetchable,
                address,
                size,
            })
        } else {
            unimplemented!("Unable to handle IO BARs")
        }
    }

    pub fn revision_and_class(&self) -> (DeviceRevision, BaseClass, SubClass, Interface) {
        let field = { self.header.0.read(0x08) };
        (
            field.get_bits(0..8) as DeviceRevision,
            field.get_bits(24..32) as BaseClass,
            field.get_bits(16..24) as SubClass,
            field.get_bits(8..16) as Interface,
        )
    }

    pub fn device_class(&self) -> ClassCode {
        let (_revision, base_class, sub_class, _interface) = self.revision_and_class();
        let class = (base_class as u16) << 8 | (sub_class as u16);
        class.into()
    }

    pub fn info(&self) -> Option<&'static device_db::PciDeviceInfo> {
        let key = device_db::make_key(self.vendor_id(), self.device_id());
        crate::pci::device_db::PCI_DEVICES.get(&key)
    }
}

impl fmt::Display for PciDevice {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: ", self.header.0)?;
        if let Some(dev_info) = self.info() {
            write!(f, "{} {}", dev_info.vendor_name, dev_info.device_name)
        } else {
            write!(
                f,
                "Unknown[{:#x}] Unknown[{:#x}]",
                self.vendor_id(),
                self.device_id()
            )
        }
    }
}

pub struct PciDeviceIterator {
    bus: u8,
    device: u8,
    function: u8,
}

// Implement `Iterator` for `PciDeviceIterator`.
// The `Iterator` trait only requires a method to be defined for the `next` element.
impl Iterator for PciDeviceIterator {
    type Item = PciDevice;

    fn next(&mut self) -> Option<Self::Item> {
        for bus in self.bus..=255 {
            for device in self.device..=31 {
                for function in self.function..=7 {
                    if let Some(pci_device) = PciDevice::new(bus, device, function) {
                        self.bus = bus;
                        self.device = device;
                        // Start with next function on next iteration
                        self.function = function + 1;

                        return Some(pci_device);
                    }
                }
            }
        }

        None
    }
}

/// Scans the PCI bus addresses, returns vector of all
pub fn scan_bus() -> PciDeviceIterator {
    PciDeviceIterator {
        bus: 0x0,
        device: 0x0,
        function: 0x0,
    }
}
