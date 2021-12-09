//! Static library of PCI device, vendor IDs to `PciDeviceInfo`
//!
//! A statically generated hashtable maps a key (a u32 constructed as `vendor id
//! << 16 | device id`) -> to `PciDeviceInfo`.
//!
//! # Note on license
//!
//! The PCI id database (parsed and included through pci_device_map.rs) is a
//! compilation of factual data, and as such the copyright only covers the
//! aggregation and formatting. The copyright of the DB files is held by Martin
//! Mares and Albert Pool. Licensed as 3-clause BSD.
//!
//! See also <https://github.com/pciutils/pciids> and
//! <https://github.com/ilyazzz/pci-id-parser> for more details.

use phf;

/// Information about a PCI device.
#[derive(Debug, Eq, PartialEq)]
pub struct PciDeviceInfo {
    pub vendor_id: u16,
    pub device_id: u16,
    pub vendor_name: &'static str,
    pub device_name: &'static str,
}

pub fn make_key(vendor: u16, device: u16) -> u32 {
    (vendor as u32) << (u32::BITS / 2) | device as u32
}

include!(concat!(env!("OUT_DIR"), "/pci_device_map.rs"));

#[cfg(test)]
mod tests {

    #[test]
    fn test_device_map() {
        let key = make_key(0xfffe, 0x0710);
        let dev = super::PCI_DEVICES.get(&key).unwrap();
        assert_eq!(dev.vendor_name, "VMWare Inc (temporary ID)");
        assert_eq!(dev.device_name, "Virtual SVGA");
        assert_eq!(dev.vendor_id, 0xfffe);
        assert_eq!(dev.device_id, 0x0710);
    }
}
