pub mod mem;

use libc;
use libbarrelfish::*;
use libbarrelfish::pci::*;

/// Has to be > 0xffff
pub const PCI_DONT_CARE: u32 = 0x10000;

#[derive(Debug)]
pub struct PciDriver {
    bus: u32,
    dev: u32,
    fun: u32,
}

extern fn device_ready(bar_info: *mut device_mem, nr_mapped_bars: libc::c_int) {
    println!("PCI device is ready");
}

extern fn irq_handler(arg: *mut libc::c_void) {
    println!("Got interrupt");
}

impl PciDriver {

    pub fn new(bus: u32, dev: u32, fun: u32) -> PciDriver {
        unsafe {
            let err = pci_client_connect();
            if err_is_fail(err) {
                panic!("pci_client_connect");
            }

            let class = PCI_DONT_CARE;
            let subclass = PCI_DONT_CARE;
            let prog_if = PCI_DONT_CARE;
            let vendor = PCI_DONT_CARE;
            let device = PCI_DONT_CARE;

            let err = pci_register_driver_irq(device_ready, class, subclass, prog_if, vendor, device,
                                              bus, dev, fun, irq_handler, 0x0 as *mut libc::c_void);
            if err_is_fail(err) {
                panic!("pci_register_driver_irq");
            }

            let ws = get_default_waitset();
            loop {
                let err = event_dispatch(ws);
                assert!(err_is_ok(err), "dispatching events failed");
            }
        }

        PciDriver{ bus: bus, dev: dev, fun: fun }
    }
}
