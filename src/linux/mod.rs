use std::prelude::v1::*;

use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::process::Command;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::MsrInterface;

pub mod mem;

pub struct MsrWriter {
    cpu: usize,
    msr_file: File,
}

impl MsrWriter {
    pub fn new(cpuid: usize) -> MsrWriter {
        // /dev/cpu/CPUNUM/msr provides an interface to read and write the
        // model-specific registers (MSRs) of an x86 CPU.  CPUNUM is the number
        // of the CPU to access as listed in /proc/cpuinfo.
        let load_module = Command::new("modprobe")
            .args(&["msr"])
            .output()
            .expect("failed to execute process");
        assert!(load_module.status.success());

        let msr_path = format!("/dev/cpu/{}/msr", cpuid);
        let msr_file = OpenOptions::new()
            .write(true)
            .read(true)
            .open(msr_path)
            .expect("Can't open file");
        MsrWriter {
            cpu: cpuid,
            msr_file: msr_file,
        }
    }
}

impl fmt::Debug for MsrWriter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "MsrWriter @ core {}", self.cpu)
    }
}

impl MsrInterface for MsrWriter {
    // The register access is done by opening the file and seeking to the
    // MSR number as offset in the file, and then reading or writing in
    // chunks of 8 bytes.  An I/O transfer of more than 8 bytes means
    // multiple reads or writes of the same register.

    unsafe fn write(&mut self, msr: u32, value: u64) {
        let pos = self
            .msr_file
            .seek(SeekFrom::Start(msr as u64))
            .expect("Can't seek");
        assert!(pos == msr.into());

        let mut contents: Vec<u8> = vec![];
        contents
            .write_u64::<LittleEndian>(value)
            .expect("Can't serialize MSR value");
        assert_eq!(contents.len(), 8, "Write exactly 8 bytes");
        self.msr_file
            .write(&contents)
            .expect(format!("Can't write MSR 0x{:x} with 0x{:x}", msr, value).as_str());

        debug!("wrmsr(0x{:x}, 0x{:x})", msr, value);
    }

    unsafe fn read(&mut self, msr: u32) -> u64 {
        let pos = self
            .msr_file
            .seek(SeekFrom::Start(msr as u64))
            .expect("Can't seek");
        assert!(pos == msr.into());
        let mut raw_value: Vec<u8> = vec![0; 8];
        self.msr_file
            .read(&mut raw_value)
            .expect("Can't read MSR value");
        let value = raw_value
            .as_slice()
            .read_u64::<LittleEndian>()
            .expect("Can't parse msr value");
        debug!("rdmsr(0x{:x}) -> 0x{:x}", msr, value);
        value
    }
}
