use std::io;
use std::io::Seek;
use std::fs::File;

use mmap;
use libc;
use libc::{MAP_SHARED, MAP_ANON, MAP_HUGETLB, MAP_POPULATE};
use byteorder::{ReadBytesExt, LittleEndian};

/// Represents a consecutive region of physical memory pinned in memory.
pub struct DevMem {
    mapping: mmap::MemoryMap
}

const MAP_HUGE_SHIFT: usize = 26;
const MAP_HUGE_2MB: i32 = 21 << MAP_HUGE_SHIFT;
const MAP_HUGE_1GB: i32 = 30 << MAP_HUGE_SHIFT;

pub const FOUR_KIB: usize = 4*1024;
pub const TWO_MIB: usize = 2*1024*1024;
pub const ONE_GIB: usize = 1024*1024*1024;
const PAGESIZE: u64 = FOUR_KIB as u64;


/// Function to read the pagemap in Linux.
/// See also https://www.kernel.org/doc/Documentation/vm/pagemap.txt.
fn read_pagemap(virtual_page: u64) -> io::Result<u64> {
    assert!(virtual_page % PAGESIZE == 0);

    let mut f = try!(File::open("/proc/self/pagemap"));

    // The pagemap contains one 64-bit value for each virtual page:
    const PAGEMAP_ENTRY_SIZE: u64 = 8;
    let start = (virtual_page / PAGESIZE) * PAGEMAP_ENTRY_SIZE;
    try!(f.seek(io::SeekFrom::Start(start)));
    let value = try!(f.read_u64::<LittleEndian>());

    // Sanity check that the page is not swapped:
    let present_bit = 1 << 63;
    assert!(value & present_bit > 0);

    // Get the physical address by multiplying the PFN bits with the page size
    let pfn_mask: u64 = (1 << 55) - 1;
    Ok( (value & pfn_mask) * PAGESIZE)
}


#[derive(Debug)]
pub enum AllocError {
    Map,
    Pin
}

impl From<mmap::MapError> for AllocError {
    fn from(e: mmap::MapError) -> Self {
        AllocError::Map
    }
}

impl DevMem {

    /// Allocates a chunk of consecutive physical, pinned memory.
    /// This should be usable by devices that do DMA.
    pub fn alloc(size: usize) -> Result<DevMem, AllocError> {
        assert!(size == FOUR_KIB || size == TWO_MIB || size == ONE_GIB);

        let mut non_standard_flags = MAP_SHARED | MAP_ANON | MAP_POPULATE;
        match size {
            TWO_MIB =>
                non_standard_flags |= MAP_HUGETLB | MAP_HUGE_2MB,
            ONE_GIB =>
                non_standard_flags |= MAP_HUGETLB | MAP_HUGE_1GB,
            _ => (),
        }

        let flags = [ mmap::MapOption::MapNonStandardFlags(non_standard_flags),
                      mmap::MapOption::MapReadable,
                      mmap::MapOption::MapWritable ];
        let res = try!(mmap::MemoryMap::new(size, &flags));

        // Make sure memory is not swapped:
        let lock_ret = unsafe { libc::mlock(res.data() as *const libc::c_void, res.len()) };
        if lock_ret == -1 {
            return Err(AllocError::Pin);
        }
        assert!(lock_ret == 0);

        Ok( DevMem { mapping: res } )
    }

    /// Returns the physical address of the memory region.
    pub fn physical_address(&self) -> u64 {
        read_pagemap(self.virtual_address() as u64).unwrap()
    }

    /// Returns the virtual address of the memory region.
    pub fn virtual_address(&self) -> usize {
        self.data() as usize
    }

    /// Returns a pointer to the memory region.
    pub fn data(&self) -> *mut u8 { self.mapping.data() }

    /// Returns the size of the memory region.
    pub fn len(&self) -> usize { self.mapping.len() }
}


#[cfg(test)]
mod tests {
    use memops::*;

    #[test]
    fn alloc_1page() {
        let res = DevMem::alloc(FOUR_KIB);
        match res {
            Err(e) => { panic!("Can not allocate: {:?}", e); },
            Ok(f) => (),
        }
    }

    #[test]
    fn alloc_2mib() {
        let res = DevMem::alloc(TWO_MIB);
        match res {
            Err(e) => { panic!("Can not allocate: {:?}", e); },
            Ok(r) =>  {
                assert!(r.physical_address() % TWO_MIB as u64 == 0)
            }
        }
    }

    #[test]
    fn alloc_1gib() {
        let res = DevMem::alloc(ONE_GIB);
        match res {
            Err(e) => { panic!("Can not allocate: {:?}", e); },
            Ok(r) => {
                assert!( (r.physical_address() % ONE_GIB as u64) == 0)
            },
        }
    }
}
