use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

use pciid_parser::PciDatabase;

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

#[derive(Debug, Eq, PartialEq)]
struct PciDeviceInfo {
    pub vendor_id: u16,
    pub device_id: u16,
    pub vendor_name: &'static str,
    pub device_name: &'static str,
}

fn main() {
    let db = PciDatabase::read().unwrap();

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("pci_device_map.rs");
    let mut filewriter = BufWriter::new(File::create(&path).unwrap());

    let mut devices = phf_codegen::Map::new();
    for (vendor_id, vendor) in db.vendors.iter() {
        eprintln!("{} {:?}", vendor_id, vendor);
        let vendor_name = string_to_static_str(vendor.name.clone());

        for (device_id, device) in vendor.devices.iter() {
            let vendor_id = u16::from_str_radix(vendor_id, 16).expect("Invalid vendor ID");
            let device_id = u16::from_str_radix(device_id, 16).expect("Invalid device ID");

            let key = (vendor_id as u32) << (u32::BITS / 2) | device_id as u32;
            let pci_dev_info = PciDeviceInfo {
                vendor_id,
                device_id,
                vendor_name,
                device_name: string_to_static_str(device.name.clone()),
            };

            devices.entry(key, string_to_static_str(format!("{:?}", pci_dev_info)));
            eprintln!("-- {} {:?}", device_id, device);
            //assert!(device.subdevices.is_empty(), "Don't deal with this atm.");
        }
    }

    writeln!(
        &mut filewriter,
        "pub static PCI_DEVICES: phf::Map<u32, PciDeviceInfo> = \n{};\n",
        devices.build()
    )
    .unwrap();
}
