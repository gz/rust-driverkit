use std::alloc::{Allocator, Layout};
use std::alloc::AllocError;
use std::vec::Vec;
use std::ptr::NonNull;
use custom_error_core::custom_error;

use std::cmp;



// custom error for the IOMemory
custom_error! {pub IOMemError
    OutOfMemory = "reached out of memory",
    NotYetImplemented = "feature not yet implemented"
}

/*
 * =================================================================================================
 * IOMemAllocator
 * =================================================================================================
 */

/// represents an IOMemAllocator that backs IOBufs
pub struct IOMemAllocator {
    /// the layout established for this allocator holding the alignment and
    layout: Layout
}

/// IOMemAllocator Implementation
impl IOMemAllocator {
    fn new(blocksize: usize, alignment: usize) -> Result<IOMemAllocator,IOMemError> {
        let layout = Layout::from_size_align(blocksize, alignment).expect("Layout was invalid.");

        Ok(IOMemAllocator {
            layout: layout
        })
    }
}

/// implements the allocator trait for the IOMemAllocator
unsafe impl Allocator for IOMemAllocator {
    /// allocates memory
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        // get the size to be allocated as a multiple of the initialized layout of the allocator
        let sz = ((layout.size() + layout.size() - 1) / self.layout.size()) * self.layout.size();
        let align = cmp::max(self.layout.align(), layout.align());

        // construct the new layout for allocation
        let alloc_layout = Layout::from_size_align(sz, align).expect("Layout was invalid.");

        unsafe {
            // do the actual allocation
            // TODO: refer to the OS allocator
            let ptr : *mut u8 = std::alloc::alloc_zeroed(alloc_layout);
            if ptr.is_null() {
                return Err(AllocError)
            }

            // wrap in in NonNull, remove option type
            let ptr_nonnull = NonNull::new(ptr).unwrap();

            // construct the NonNull slice for the return
            Ok(NonNull::slice_from_raw_parts(ptr_nonnull, sz))
        }
    }

    /// deallocates the previously allocated memory
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // XXX: check the layout matches the allocator here?
        let buf = ptr.as_ptr();
        // TODO: refer to the OS allocator
        std::alloc::dealloc(buf, layout);
    }
}


/*
 * =================================================================================================
 * IOBuf
 * =================================================================================================
 */

/// TODO: move this somewhere else
const KERNEL_BASE : u64 = 0xffff000000000000;

/// represents an IO buffer
pub struct IOBuf {
    /// the address to be used by the device/hardware. TODO make this dependent whether its mapped
    ioaddr: u64,
    ///
    buf : Vec<u8, IOMemAllocator>
}

impl IOBuf {
    fn new(len: usize, align: usize) -> Result<IOBuf,IOMemError> {
        // get the aligned buffer length

        // get the layouf for the allocation
        let allocator = IOMemAllocator::new(len, align);

        let buf : Vec<u8, IOMemAllocator> = Vec::with_capacity_in(len, allocator.unwrap());

        // for now just using the phys addr... TODO: setup with
        let ioaddr = buf.as_ptr() as u64 - KERNEL_BASE;

        Ok(IOBuf{
            ioaddr: ioaddr,
            buf: buf,
        })
    }

    /// clears the buffer contents
    fn clear(&mut self) {
        self.buf.clear();
    }

    /// copy data in at a given offset
    fn copy_in_at(&mut self, offset: usize, src: &[u8]) -> Result<usize, IOMemError> {

        // currently we do not allow extending the buffer here
        let remaining_capacity = self.buf.capacity() - offset;
        let cnt = cmp::min(remaining_capacity, src.len());

        // copy the slice
        self.buf[offset..offset+cnt].copy_from_slice(&src[0..cnt]);

        Ok(cnt)
    }

    /// copy data raw data of size `len` into the buffer at offset `offset`
    fn copy_in(&mut self, src: &[u8]) -> Result<usize, IOMemError>{
        self.copy_in_at(self.buf.len(), src)
    }

    /// copy data out of the buffer starting at a given offset upto a reuqested length
    fn copy_out_at(&self, offset : usize, dst: & mut [u8]) -> Result<usize, IOMemError> {
        // of the offset is outside of the length of the vector then we
        if offset >= self.buf.len() {
            return Ok(0)
        }

        let cnt = cmp::min(self.buf.len() - offset, dst.len());
        // copy the slice
        dst[0..cnt].copy_from_slice(&self.buf[offset..offset + cnt]);
        Ok(cnt)
    }

    /// copy the data from 0 to len into the dst vector
    fn copy_out(&self, dst: &mut [u8]) -> Result<usize, IOMemError> {
        self.copy_out_at(0, dst)
    }

    /// get a u8 array from the backing buffer
    fn as_slice(&self) -> Result<&[u8], IOMemError> {
        Ok(self.buf.as_slice())
    }

    /// get a mutable u8 array from the backing buffer
    fn as_mut_slice(&mut self) -> Result<&mut [u8], IOMemError> {
        Ok(self.buf.as_mut_slice())
    }
}

// impl DmaObject for IOBuf {
//     /// address of the IOBuf in main memory
//     fn paddr(&self) -> PAddr {
//         PAddr::from(self.buf.as_ptr() as u64 - pci::KERNEL_BASE)
//     }

//     /// virtual address this buffer can be access by software
//     fn vaddr(&self) -> VAddr {
//         VAddr::from(self.buf.as_ptr() as u64)
//     }

//     /// io address this buffer can be accessed by the device
//     // fn ioaddr(&self) -> Option<IOAddr> {
//     //     IOAddr::from(self.ioaddr)
//     // }
// }
